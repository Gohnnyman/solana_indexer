use crate::actors::collector::CollectorHandle;
use crate::actors::erroneous_transactions_collector::ErroneousTransactionsCollectorHandle;
use crate::actors::prometheus_exporter::PrometheusExporterHandle;
use crate::actors::transaction_parser::TransactionParserHandle;
use crate::{actors::queue_manager::QueueManagerHandle, register::Register};
use crate::{metrics_update, repeat_until_ok};
use anyhow::Result;
use log::error;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tokio::time::{sleep, Duration};

pub struct TransactionsParsingCtx;

impl TransactionsParsingCtx {
    pub async fn setup_and_run(register: &Register) -> Result<Self> {
        let transaction_queue_manager = QueueManagerHandle::new(register).await?;
        let collector = CollectorHandle::new(register).await?;
        let erroneous_transactions_collector =
            ErroneousTransactionsCollectorHandle::new(register).await?;
        PrometheusExporterHandle::new(register).await?;

        let transaction_parser = TransactionParserHandle::new().await;

        // Transaction thread
        tokio::spawn(TransactionsParsingCtx::transaction_worker(
            transaction_queue_manager,
            transaction_parser,
            collector,
            erroneous_transactions_collector,
        ));

        Ok(Self {})
    }

    async fn transaction_worker(
        mut queue_manager: QueueManagerHandle,
        mut transaction_parser: TransactionParserHandle,
        mut collector: CollectorHandle,
        mut erroneous_transactions_collector: ErroneousTransactionsCollectorHandle,
    ) {
        metrics_update!(inc total ACTIVE_WORKERS_COUNT, &["transaction"]);

        let transaction_join_handle = tokio::spawn(async move {
            loop {
                let loop_timer = metrics_update!(timer LOOP_TIME, &["transaction"]);
                let encoded_transaction_res = queue_manager
                    .get_transactions()
                    .await
                    .unwrap_or_else(|err| {
                        panic!("Transaction queue manager has been killed: {:#?}", err);
                    });

                if encoded_transaction_res.is_empty() {
                    sleep(Duration::from_millis(5000)).await;
                    continue;
                }

                for encoded_transaction in encoded_transaction_res {
                    // ToDo: mark transaction as parsed (2) after instructions and balances will be stored

                    // EncodedConfirmedTransactionWithStatusMeta doesn't implement Copy trait
                    let cloned_encoded_transaction = EncodedConfirmedTransactionWithStatusMeta {
                        slot: encoded_transaction.slot,
                        transaction: encoded_transaction.transaction.clone(),
                        block_time: encoded_transaction.block_time,
                    };

                    let parsing_timer = metrics_update!(timer TRANSACTION_PARSING_TIME);
                    let parsing_result = transaction_parser
                        .parse_transaction(cloned_encoded_transaction)
                        .await;
                    metrics_update!(timer observe parsing_timer);

                    match parsing_result {
                        Ok(parsing_result) => {
                            let (instructions, balances, instruction_arguments) = parsing_result;

                            let (delegations, undelegations) = repeat_until_ok!(
                                transaction_parser
                                    .parse_delegations(
                                        queue_manager.clone(),
                                        instructions.clone(),
                                        balances
                                            .iter()
                                            .map(|balance| {
                                                (
                                                    balance.account.clone(),
                                                    balance.pre_balance.unwrap(),
                                                )
                                            })
                                            .collect(),
                                    )
                                    .await,
                                5
                            );

                            let tx_signature = instructions[0].tx_signature.clone();

                            for instruction in instructions {
                                collector.save_instruction(instruction).await;
                            }

                            for instruction_argument in instruction_arguments {
                                collector
                                    .save_instruction_argument(instruction_argument)
                                    .await;
                            }

                            for balance in balances {
                                collector.save_balance(balance).await;
                            }

                            for delegation in delegations {
                                collector.save_delegation(delegation).await;
                            }

                            for undelegation in undelegations {
                                collector.save_undelegation(undelegation).await;
                            }

                            repeat_until_ok!(
                                queue_manager
                                    .mark_transaction_as_parsed(tx_signature.clone())
                                    .await,
                                5
                            );
                        }
                        Err(parsing_err) => {
                            if let Err(err) = erroneous_transactions_collector
                                .handle_error(encoded_transaction, parsing_err)
                                .await
                            {
                                log::error!(
                                    "Problem occurred when processing erroneous transaction: {:#?}",
                                    err
                                );
                            } else {
                                metrics_update!(inc ERRONEOUS_TRANSACTIONS_COUNT);
                            }
                        }
                    }
                }
                metrics_update!(timer observe loop_timer);
            }
        });

        if transaction_join_handle.await.is_err() {
            metrics_update!(dec total ACTIVE_WORKERS_COUNT, &["transaction"]);
            error!("Transaction worker has been killed");
        }
    }
}
