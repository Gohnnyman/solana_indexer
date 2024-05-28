use super::{main_storage::Metadata, QueueStorage};
use anyhow::Result;
use async_trait::async_trait;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, Consumer};
use log::{error, info};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

use serialization::{deserialize_metadata, deserialize_transaction};

mod serialization;

pub struct RabbitStorage {
    consumer: Consumer,
}

impl RabbitStorage {
    pub async fn new(addr: &str, queue_type: QueueType) -> Result<Self> {
        let connection = Connection::connect(addr, ConnectionProperties::default()).await?;

        info!("Connection to RabbitMQ has been established");

        let channel = connection.create_channel().await?;

        info!("Channel created");

        let consumer = match queue_type {
            QueueType::Transaction => {
                let transaction_consumer = channel
                    .basic_consume(
                        "Transactions",
                        "DataAnalyzer_TransactionConsumer",
                        BasicConsumeOptions::default(),
                        FieldTable::default(),
                    )
                    .await?;

                info!("Transaction Consumer created");
                transaction_consumer
            }

            QueueType::Metadata => {
                let metadata_consumer = channel
                    .basic_consume(
                        "Metadata",
                        "DataAnalyzer_MetadataConsumer",
                        BasicConsumeOptions::default(),
                        FieldTable::default(),
                    )
                    .await?;

                info!("Metadata Consumer created");
                metadata_consumer
            }
        };

        Ok(Self { consumer })
    }
}

#[async_trait]
impl QueueStorage for RabbitStorage {
    async fn get_transactions(&mut self) -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
        todo!()
        // if let Some(delivery) = self.consumer.next().await {
        //     if delivery.is_err() {
        //         error!(
        //             "Cannot consume message from RabbitMQ: {:#?}",
        //             delivery.err().unwrap()
        //         );
        //         return None;
        //     }

        //     let delivery = delivery.unwrap();

        //     let transaction = deserialize_transaction(delivery.data.as_slice());

        //     if transaction.is_err() {
        //         error!(
        //             "Cannot deserialize delivered data from RabbitMQ: {:#?}",
        //             transaction.err().unwrap()
        //         );
        //         return None;
        //     }

        //     let ack_result = delivery.ack(BasicAckOptions::default()).await;

        //     if ack_result.is_err() {
        //         error!("Cannot ack message: {:#?}", ack_result.err().unwrap());
        //         return None;
        //     }

        //     Some(transaction.unwrap())
        // } else {
        //     None
        // }
    }

    async fn get_metadata(&mut self) -> Option<Metadata> {
        if let Some(delivery) = self.consumer.next().await {
            if delivery.is_err() {
                error!(
                    "Cannot consume message from RabbitMQ: {:#?}",
                    delivery.err().unwrap()
                );
                return None;
            }

            let delivery = delivery.unwrap();
            let metadata = deserialize_metadata(delivery.data.as_slice());

            if metadata.is_err() {
                error!(
                    "Cannot deserialize delivered data from RabbitMQ: {:#?}",
                    metadata.err().unwrap()
                );
                return None;
            }

            let ack_result = delivery.ack(BasicAckOptions::default()).await;

            if ack_result.is_err() {
                error!("Cannot ack message: {:#?}", ack_result.err().unwrap());
                return None;
            }

            Some(metadata.unwrap())
        } else {
            None
        }
    }
}
