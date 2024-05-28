use std::collections::HashMap;

use crate::errors::ParseInstructionError;
use crate::metrics_update;
use crate::storages::main_storage::{Balance, Delegation, Instruction, InstructionArgument};

use anyhow::Result;
use log::debug;
use macros::{ActorInstance, HandleInstance};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tokio::sync::{mpsc, oneshot};

pub type Delegations = Vec<Delegation>;
pub type Undelegations = Vec<Delegation>;

use super::queue_manager::QueueManagerHandle;

mod parse_delegations;
mod parse_instructions;

const STAKE_ACC_RENT_EXEMPTION: u64 = 2_282_880;

#[derive(ActorInstance)]
struct TransactionParser {
    receiver: mpsc::Receiver<TransactionParserMessage>,
}

type TransactionParsingResult = (Vec<Instruction>, Vec<Balance>, Vec<InstructionArgument>);

enum TransactionParserMessage {
    GetInstructions {
        respond_to: oneshot::Sender<Result<TransactionParsingResult, ParseInstructionError>>,
        encoded_confirmed_transaction: EncodedConfirmedTransactionWithStatusMeta,
    },
    GetDelegations {
        respond_to: oneshot::Sender<Result<(Delegations, Undelegations)>>,
        queue_manager: QueueManagerHandle,
        instructions: Vec<Instruction>,
        pre_balances: HashMap<String, u64>,
    },
}

impl TransactionParser {
    async fn new(receiver: mpsc::Receiver<TransactionParserMessage>) -> Self {
        metrics_update!(inc total ACTIVE_ACTOR_INSTANCES_COUNT, &["transaction_parser"]);
        TransactionParser { receiver }
    }

    async fn handle_message(&mut self, msg: TransactionParserMessage) {
        match msg {
            TransactionParserMessage::GetInstructions {
                respond_to,
                encoded_confirmed_transaction,
            } => {
                debug!(
                    "TransactionParser::handle_message: {:#?}",
                    encoded_confirmed_transaction
                );
                let parsing_result = Self::parse_transactions(encoded_confirmed_transaction);
                let _ = respond_to.send(parsing_result);
            }

            TransactionParserMessage::GetDelegations {
                respond_to,
                queue_manager,
                instructions,
                pre_balances,
            } => {
                let parsing_result =
                    Self::parse_delegations(queue_manager, instructions, pre_balances).await;
                let _ = respond_to.send(parsing_result);
            }
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }
}

#[derive(HandleInstance)]
pub struct TransactionParserHandle {
    sender: mpsc::Sender<TransactionParserMessage>,
}

impl TransactionParserHandle {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let mut parser_manager = TransactionParser::new(receiver).await;
        tokio::spawn(async move { parser_manager.run().await });

        metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &["transaction_parser_handle"]);

        Self { sender }
    }

    pub async fn parse_delegations(
        &mut self,
        queue_manager: QueueManagerHandle,
        instructions: Vec<Instruction>,
        pre_balances: HashMap<String, u64>,
    ) -> Result<(Delegations, Undelegations)> {
        let (sender, receiver) = oneshot::channel();
        let msg = TransactionParserMessage::GetDelegations {
            respond_to: sender,
            queue_manager,
            instructions,
            pre_balances,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("TransactionParser task has been killed")
    }

    pub async fn parse_transaction(
        &mut self,
        encoded_confirmed_transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<TransactionParsingResult, ParseInstructionError> {
        let (sender, receiver) = oneshot::channel();
        let msg = TransactionParserMessage::GetInstructions {
            respond_to: sender,
            encoded_confirmed_transaction,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("TransactionParser task has been killed")
    }
}

#[tokio::test]
async fn parse_instruction() -> Result<(), String> {
    let encoded_transaction = "
    {
        \"transaction\":{
            \"signatures\":[
                \"3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU\",
                \"2jSM9Z45j51ifbKCH1kLe2jSfcoh1x5XYSWfzZHpvJLQpNw1HSm6kykFUsN1JLCjaMLcbdpbkEK1hTQBL7jYfJj6\"
            ],
            \"message\":{
                \"header\":{
                    \"numRequiredSignatures\":2,
                    \"numReadonlySignedAccounts\":0,
                    \"numReadonlyUnsignedAccounts\":9
                },
                \"accountKeys\":[
                    \"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\",
                    \"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                    \"JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy\",
                    \"Aurdw9mjPnBMQCiczdN4H7qfSoHF8K915GfSi364SZgA\",
                    \"DV2rLHZsXZLTJzfQ3iUQoKxqX8phM8hR4qjgxtqRV81W\",
                    \"6DnkBtW5UmsWRFCZBkihS1yZzUWWKpUZiHUwMPDx6c9C\",
                    \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                    \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                    \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                    \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                    \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                    \"SysvarRent111111111111111111111111111111111\",
                    \"11111111111111111111111111111111\",
                    \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                    \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                    \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                    \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                    \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                    \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                    \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\"
                ],
                \"recentBlockhash\":\"2JpSV2YKxT9dhMtHCcEVPFQi4WMVNDSL8QW9Xqb4Jrd4\",
                \"instructions\":[
                    {
                        \"programIdIndex\":13,
                        \"accounts\":[0,1],
                        \"data\":\"11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,12
                        ],
                        \"data\":\"11MNMwXYvKPccpzacm55yfoDVN9UBrpnqpeCRxJSuWFC5uaDNTXr8DpxhhsDPuGmTbrgcrR8mSvmsSTqVSGitFWsSmM\"
                    },{
                        \"programIdIndex\":19,
                        \"accounts\":[
                            0,2,0,1,13,14,12
                        ],
                        \"data\":\"\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,2,0
                        ],
                        \"data\":\"6AuM4xMCPFhR\"
                    },{
                        \"programIdIndex\":20,
                        \"accounts\":[
                            15,3,0,16,4,5,6,7,8,1,0,9,10,11,12,17,18,14,13
                        ],
                        \"data\":\"guFfuH\"
                    }
                ]
            }
        },
        \"meta\":{
            \"err\":null,
            \"status\":{
                \"Ok\":null
            },
            \"fee\":10000,
            \"preBalances\":[
                501683013,0,0,7168800,1900080,2039280,0,0,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
            ],
            \"postBalances\":[
                489987173,1461600,2039280,7168800,1900080,2039280,5616720,2568240,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
            ],
            \"innerInstructions\":[
                {
                    \"index\":2,
                    \"instructions\":[
                        {
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,2
                            ],
                            \"data\":\"3Bxs4h24hBtQy9rw\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                2
                            ],
                            \"data\":\"9krTDU2LzCSUJuVZ\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                2
                            ],
                            \"data\":\"SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                2,1,0,12
                            ],
                            \"data\":\"2\"
                        }
                    ]
                },{
                    \"index\":4,
                    \"instructions\":[
                        {
                            \"programIdIndex\":18,
                            \"accounts\":[
                                6,7,8,1,11,0,0,16,5,0,9,14,13,12
                            ],
                            \"data\":\"9D2mNcMSmYR5\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,6
                            ],
                            \"data\":\"3Bxs4EMbRQoDyoj5\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                6
                            ],
                            \"data\":\"9krTDUMpjBo4wxLP\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                6
                            ],
                            \"data\":\"SYXsBkG6yKW2wWDcW8EDHR6D3P82bKxJGPpM65DD8nHqBfMP\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,7
                            ],
                            \"data\":\"3Bxs48v9NdVhakdd\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                7
                            ],
                            \"data\":\"9krTDgje7Fnho7ps\"
                        },{
                            \"programIdIndex\":13,
                            \"accounts\":[
                                7
                            ],
                            \"data\":\"SYXsBkG6yKW2wWDcW8EDHR6D3P82bKxJGPpM65DD8nHqBfMP\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"biy3SZviff8JK2ske48JhXBfLVA8SeCDLcf1rQfY8uouBdD\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"bkH6Deonc6hYPobmkX4Tcy5Bqpg6sNvvcgrptbusxEJ72dq\"
                        }
                    ]
                }
            ],
            \"logMessages\":[
                \"Program 11111111111111111111111111111111 invoke [1]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]\",
                \"Program log: Instruction: InitializeMint\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2457 of 200000 compute units\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success\",
                \"Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]\",
                \"Program log: Transfer 2039280 lamports to the associated token account\",
                \"Program 11111111111111111111111111111111 invoke [2]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Allocate space for the associated token account\",
                \"Program 11111111111111111111111111111111 invoke [2]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Assign the associated token account to the SPL Token program\",
                \"Program 11111111111111111111111111111111 invoke [2]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Initialize the associated token account\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]\",
                \"Program log: Instruction: InitializeAccount\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3297 of 179576 compute units\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success\",
                \"Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 24370 of 200000 compute units\",
                \"Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]\",
                \"Program log: Instruction: MintTo\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2611 of 200000 compute units\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success\",
                \"Program packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu invoke [1]\",
                \"Program log: Instruction: ClaimPack\",
                \"Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s invoke [2]\",
                \"Program log: Instruction: Mint New Edition from Master Edition Via Token\",
                \"Program log: Transfer 5616720 lamports to the new account\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Allocate space for the account\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Assign the account to the owning program\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Transfer 2568240 lamports to the new account\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Allocate space for the account\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Assign the account to the owning program\",
                \"Program 11111111111111111111111111111111 invoke [3]\",
                \"Program 11111111111111111111111111111111 success\",
                \"Program log: Setting mint authority\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]\",
                \"Program log: Instruction: SetAuthority\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1929 of 120161 compute units\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success\",
                \"Program log: Setting freeze authority\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]\",
                \"Program log: Instruction: SetAuthority\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1928 of 115676 compute units\",
                \"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success\",
                \"Program log: Finished setting freeze authority\",
                \"Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s consumed 60432 of 173045 compute units\",
                \"Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s success\",
                \"Program packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu consumed 91571 of 200000 compute units\",
                \"Program packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu success\"
            ],
            \"preTokenBalances\":[
                {
                    \"accountIndex\":5,
                    \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"uiTokenAmount\":
                    {
                        \"uiAmount\":1.0,
                        \"decimals\":0,
                        \"amount\":\"1\",
                        \"uiAmountString\":\"1\"
                    },
                    \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                }
            ],
            \"postTokenBalances\":[
                {
                    \"accountIndex\":2,
                    \"mint\":\"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                    \"uiTokenAmount\":
                    {
                        \"uiAmount\":1.0,
                        \"decimals\":0,
                        \"amount\":\"1\",
                        \"uiAmountString\":\"1\"
                    },
                    \"owner\":\"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\"
                },{
                    \"accountIndex\":5,
                    \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"uiTokenAmount\":
                    {
                        \"uiAmount\":1.0,
                        \"decimals\":0,
                        \"amount\":\"1\",
                        \"uiAmountString\":\"1\"
                    },
                    \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                }
            ],
            \"rewards\":[]
        }
    }";

    let encoded_confirmed_transaction = EncodedConfirmedTransactionWithStatusMeta {
        slot: 117946133_u64,
        transaction: serde_json::from_str(encoded_transaction).unwrap(),
        block_time: Some(1643213404_i64),
    };

    let mut transaction_parser = TransactionParserHandle::new().await;
    let parsed_transaction = transaction_parser
        .parse_transaction(encoded_confirmed_transaction)
        .await
        .unwrap();

    assert_eq!(parsed_transaction.0.len(), 18);

    assert_eq!(
        parsed_transaction.0[0].tx_signature,
        "3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU"
            .to_string()
    );

    let mut accs: [Option<String>; crate::storages::main_storage::ACCOUNTS_ARRAY_SIZE] = [0;
        crate::storages::main_storage::ACCOUNTS_ARRAY_SIZE]
        .iter()
        .map(|_| -> Option<String> { None })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(); // Will never fail because of the same size

    accs[0] = Some("E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8".to_string());
    accs[1] = Some("JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy".to_string());
    accs[2] = Some("GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm".to_string());

    assert_eq!(parsed_transaction.0[3].accounts, accs);

    assert_eq!(parsed_transaction.0[4].instruction_name, "ClaimPack");

    Ok(())
}

#[cfg(test)]
mod parse_erroneous_transaction_tests {
    use super::*;

    #[tokio::test]
    async fn invalid_index_test() {
        let encoded_transaction = "
        {
            \"transaction\":{
                \"signatures\":[
                \"3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU\",
                \"2jSM9Z45j51ifbKCH1kLe2jSfcoh1x5XYSWfzZHpvJLQpNw1HSm6kykFUsN1JLCjaMLcbdpbkEK1hTQBL7jYfJj6\"
                ],
                \"message\":{
                    \"header\":{
                        \"numRequiredSignatures\":2,
                        \"numReadonlySignedAccounts\":0,
                    \"numReadonlyUnsignedAccounts\":9
                },
                \"accountKeys\":[
                    \"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\",
                    \"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                    \"JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy\",
                    \"Aurdw9mjPnBMQCiczdN4H7qfSoHF8K915GfSi364SZgA\",
                    \"DV2rLHZsXZLTJzfQ3iUQoKxqX8phM8hR4qjgxtqRV81W\",
                    \"6DnkBtW5UmsWRFCZBkihS1yZzUWWKpUZiHUwMPDx6c9C\",
                    \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                    \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                    \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                    \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                    \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                    \"SysvarRent111111111111111111111111111111111\",
                    \"11111111111111111111111111111111\",
                    \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                    \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                    \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                    \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                    \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                    \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                    \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\"
                ],
                \"recentBlockhash\":\"2JpSV2YKxT9dhMtHCcEVPFQi4WMVNDSL8QW9Xqb4Jrd4\",
                \"instructions\":[
                    {
                        \"programIdIndex\":13,
                        \"accounts\":[0,1],
                        \"data\":\"11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,12
                        ],
                        \"data\":\"11MNMwXYvKPccpzacm55yfoDVN9UBrpnqpeCRxJSuWFC5uaDNTXr8DpxhhsDPuGmTbrgcrR8mSvmsSTqVSGitFWsSmM\"
                    },{
                        \"programIdIndex\":19,
                        \"accounts\":[
                            0,2,0,1,13,14,12
                        ],
                        \"data\":\"\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,2,0
                        ],
                        \"data\":\"6AuM4xMCPFhR\"
                    },{
                        \"programIdIndex\":20,
                        \"accounts\":[
                            15,3,0,16,4,5,6,7,8,1,0,9,10,11,12,17,18,14,13
                        ],
                        \"data\":\"guFfuH\"
                    }
                ]
            }
        },
        \"meta\":{
            \"err\":null,
            \"status\":{
                \"Ok\":null
            },
            \"fee\":10000,
            \"preBalances\":[
                501683013,0,0,7168800,1900080,2039280,0,0,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"postBalances\":[
                489987173,1461600,2039280,7168800,1900080,2039280,5616720,2568240,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"innerInstructions\":[
                    {
                    \"index\":2,
                    \"instructions\":[
                        {
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,2
                            ],
                            \"data\":\"3Bxs4h24hBtQy9rw\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                2,1,0,12
                            ],
                            \"data\":\"2\"
                        }
                    ]
                },{
                    \"index\":4,
                    \"instructions\":[
                        {
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"biy3SZviff8JK2ske48JhXBfLVA8SeCDLcf1rQfY8uouBdD\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"bkH6Deonc6hYPobmkX4Tcy5Bqpg6sNvvcgrptbusxEJ72dq\"
                        }
                    ]
                }
            ],
            \"logMessages\":[
            ],
            \"preTokenBalances\":[
                {
                    \"accountIndex\":5,
                    \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"uiTokenAmount\":
                    {
                        \"uiAmount\":1.0,
                        \"decimals\":0,
                        \"amount\":\"1\",
                        \"uiAmountString\":\"1\"
                    },
                    \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                }
            ],
            \"postTokenBalances\":[
                {
                    \"accountIndex\":37,
                    \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"uiTokenAmount\":
                    {
                        \"uiAmount\":1.0,
                        \"decimals\":0,
                        \"amount\":\"1\",
                        \"uiAmountString\":\"1\"
                    },
                    \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                }
            ],
            \"rewards\":[]
        }
        }";

        let encoded_confirmed_transaction = EncodedConfirmedTransactionWithStatusMeta {
            slot: 117946133_u64,
            transaction: serde_json::from_str(encoded_transaction).unwrap(),
            block_time: Some(1643213404_i64),
        };

        let mut transaction_parser = TransactionParserHandle::new().await;
        let result = transaction_parser
            .parse_transaction(encoded_confirmed_transaction)
            .await;

        if let Err(ParseInstructionError::InvalidIndex {
            site,
            index,
            max_len,
        }) = result
        {
            assert_eq!(site, "post_token_balance".to_string());
            assert_eq!(index, 37);
            assert_eq!(max_len, crate::storages::main_storage::ACCOUNTS_ARRAY_SIZE);
        } else {
            panic!("Value is not \"ParseInstructionError::InvalidIndex\"");
        }
    }

    #[tokio::test]
    async fn invalid_length_test() {
        let encoded_transaction = "
        {
            \"transaction\":{
                \"signatures\":[
                \"3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU\",
                \"2jSM9Z45j51ifbKCH1kLe2jSfcoh1x5XYSWfzZHpvJLQpNw1HSm6kykFUsN1JLCjaMLcbdpbkEK1hTQBL7jYfJj6\"
                ],
                \"message\":{
                    \"header\":{
                        \"numRequiredSignatures\":2,
                        \"numReadonlySignedAccounts\":0,
                    \"numReadonlyUnsignedAccounts\":9
                },
                \"accountKeys\":[
                    \"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\",
                    \"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                    \"JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy\",
                    \"Aurdw9mjPnBMQCiczdN4H7qfSoHF8K915GfSi364SZgA\",
                    \"DV2rLHZsXZLTJzfQ3iUQoKxqX8phM8hR4qjgxtqRV81W\",
                    \"6DnkBtW5UmsWRFCZBkihS1yZzUWWKpUZiHUwMPDx6c9C\",
                    \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                    \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                    \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                    \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                    \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                    \"SysvarRent111111111111111111111111111111111\",
                    \"11111111111111111111111111111111\",
                    \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                    \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                    \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                    \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                    \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                    \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                    \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\",

                    \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                    \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                    \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                    \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                    \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                    \"SysvarRent111111111111111111111111111111111\",
                    \"11111111111111111111111111111111\",
                    \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                    \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                    \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                    \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                    \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                    \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                    \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\"
                ],
                \"recentBlockhash\":\"2JpSV2YKxT9dhMtHCcEVPFQi4WMVNDSL8QW9Xqb4Jrd4\",
                \"instructions\":[
                    {
                        \"programIdIndex\":13,
                        \"accounts\":[0,1],
                        \"data\":\"11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,12
                        ],
                        \"data\":\"11MNMwXYvKPccpzacm55yfoDVN9UBrpnqpeCRxJSuWFC5uaDNTXr8DpxhhsDPuGmTbrgcrR8mSvmsSTqVSGitFWsSmM\"
                    },{
                        \"programIdIndex\":19,
                        \"accounts\":[
                            0,2,0,1,13,14,12
                        ],
                        \"data\":\"\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,2,0
                        ],
                        \"data\":\"6AuM4xMCPFhR\"
                    },{
                        \"programIdIndex\":20,
                        \"accounts\":[
                            15,3,0,16,4,5,6,7,8,1,0,9,10,11,12,17,18,14,13
                        ],
                        \"data\":\"guFfuH\"
                    }
                ]
            }
        },
        \"meta\":{
            \"err\":null,
            \"status\":{
                \"Ok\":null
            },
            \"fee\":10000,
            \"preBalances\":[
                501683013,0,0,7168800,1900080,2039280,0,0,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"postBalances\":[
                489987173,1461600,2039280,7168800,1900080,2039280,5616720,2568240,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"innerInstructions\":[
                    {
                    \"index\":2,
                    \"instructions\":[
                        {
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,2
                            ],
                            \"data\":\"3Bxs4h24hBtQy9rw\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                2,1,0,12
                            ],
                            \"data\":\"2\"
                        }
                    ]
                },{
                    \"index\":4,
                    \"instructions\":[
                        {
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"biy3SZviff8JK2ske48JhXBfLVA8SeCDLcf1rQfY8uouBdD\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"bkH6Deonc6hYPobmkX4Tcy5Bqpg6sNvvcgrptbusxEJ72dq\"
                        }
                    ]
                }
            ],
            \"logMessages\":[
            ],
            \"preTokenBalances\":[
            ],
            \"postTokenBalances\":[
            ],
            \"rewards\":[]
        }
        }";

        let encoded_confirmed_transaction = EncodedConfirmedTransactionWithStatusMeta {
            slot: 117946133_u64,
            transaction: serde_json::from_str(encoded_transaction).unwrap(),
            block_time: Some(1643213404_i64),
        };

        let mut transaction_parser = TransactionParserHandle::new().await;
        let result = transaction_parser
            .parse_transaction(encoded_confirmed_transaction)
            .await;

        if let Err(ParseInstructionError::InvalidLength {
            site,
            len,
            expected_len,
        }) = result
        {
            assert_eq!(site, "accounts".to_string());
            assert_eq!(len, 36);
            assert_eq!(
                expected_len,
                crate::storages::main_storage::ACCOUNTS_ARRAY_SIZE
            );
        } else {
            panic!("Value is not \"ParseInstructionError::InvalidLength\"");
        }
    }

    #[tokio::test]
    async fn deserialize_from_base58_error_test() {
        let encoded_transaction = "
        {
            \"transaction\":{
                \"signatures\":[
                \"3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU\",
                \"2jSM9Z45j51ifbKCH1kLe2jSfcoh1x5XYSWfzZHpvJLQpNw1HSm6kykFUsN1JLCjaMLcbdpbkEK1hTQBL7jYfJj6\"
                ],
                \"message\":{
                    \"header\":{
                        \"numRequiredSignatures\":2,
                        \"numReadonlySignedAccounts\":0,
                    \"numReadonlyUnsignedAccounts\":9
                },
                \"accountKeys\":[
                    \"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\",
                    \"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                    \"JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy\",
                    \"Aurdw9mjPnBMQCiczdN4H7qfSoHF8K915GfSi364SZgA\",
                    \"DV2rLHZsXZLTJzfQ3iUQoKxqX8phM8hR4qjgxtqRV81W\",
                    \"6DnkBtW5UmsWRFCZBkihS1yZzUWWKpUZiHUwMPDx6c9C\",
                    \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                    \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                    \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                    \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                    \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                    \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                    \"SysvarRent111111111111111111111111111111111\",
                    \"11111111111111111111111111111111\",
                    \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                    \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                    \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                    \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                    \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                    \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                    \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\"
                ],
                \"recentBlockhash\":\"2JpSV2YKxT9dhMtHCcEVPFQi4WMVNDSL8QW9Xqb4Jrd4\",
                \"instructions\":[
                    {
                        \"programIdIndex\":13,
                        \"accounts\":[0,1],
                        \"data\":\"11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,12
                        ],
                        \"data\":\"11MNMwXYvKPccpzacm55yfoDVN9UBrpnqpeCRxJSuWFC5uaDNTXr8DpxhhsDPuGmTbrgcrR8mSvmsSTqVSGitFWsSmM\"
                    },{
                        \"programIdIndex\":19,
                        \"accounts\":[
                            0,2,0,1,13,14,12
                        ],
                        \"data\":\"ERROR IS HERE\"
                    },{
                        \"programIdIndex\":14,
                        \"accounts\":[
                            1,2,0
                        ],
                        \"data\":\"6AuM4xMCPFhR\"
                    },{
                        \"programIdIndex\":20,
                        \"accounts\":[
                            15,3,0,16,4,5,6,7,8,1,0,9,10,11,12,17,18,14,13
                        ],
                        \"data\":\"guFfuH\"
                    }
                ]
            }
        },
        \"meta\":{
            \"err\":null,
            \"status\":{
                \"Ok\":null
            },
            \"fee\":10000,
            \"preBalances\":[
                501683013,0,0,7168800,1900080,2039280,0,0,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"postBalances\":[
                489987173,1461600,2039280,7168800,1900080,2039280,5616720,2568240,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"innerInstructions\":[
                    {
                    \"index\":2,
                    \"instructions\":[
                        {
                            \"programIdIndex\":13,
                            \"accounts\":[
                                0,2
                            ],
                            \"data\":\"3Bxs4h24hBtQy9rw\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                2,1,0,12
                            ],
                            \"data\":\"2\"
                        }
                    ]
                },{
                    \"index\":4,
                    \"instructions\":[
                        {
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"biy3SZviff8JK2ske48JhXBfLVA8SeCDLcf1rQfY8uouBdD\"
                        },{
                            \"programIdIndex\":14,
                            \"accounts\":[
                                1,0,0
                            ],
                            \"data\":\"bkH6Deonc6hYPobmkX4Tcy5Bqpg6sNvvcgrptbusxEJ72dq\"
                        }
                    ]
                }
            ],
            \"logMessages\":[
            ],
            \"preTokenBalances\":[
            ],
            \"postTokenBalances\":[
            ],
            \"rewards\":[]
        }
        }";

        let encoded_confirmed_transaction = EncodedConfirmedTransactionWithStatusMeta {
            slot: 117946133_u64,
            transaction: serde_json::from_str(encoded_transaction).unwrap(),
            block_time: Some(1643213404_i64),
        };

        let mut transaction_parser = TransactionParserHandle::new().await;
        let result = transaction_parser
            .parse_transaction(encoded_confirmed_transaction)
            .await;

        if let Err(ParseInstructionError::DeserializeFromBase58Error) = result {
        } else {
            panic!("Value is not \"ParseInstructionError::DeserializeFromBase58Error\"");
        }
    }

    #[tokio::test]
    async fn program_address_match_test() {
        let encoded_transaction = "
        {
            \"transaction\":{
                \"signatures\":[
                    \"3gDkTVuedWyYiqaZMhZE7axGZMnWS6Jaha62SJuf67HY6D3hgZZ2qmUwwh4qEZZhCCYETHjFXDMzayJGqwHW1ChU\",
                    \"2jSM9Z45j51ifbKCH1kLe2jSfcoh1x5XYSWfzZHpvJLQpNw1HSm6kykFUsN1JLCjaMLcbdpbkEK1hTQBL7jYfJj6\"
                ],
                \"message\":{
                    \"header\":{
                        \"numRequiredSignatures\":2,
                        \"numReadonlySignedAccounts\":0,
                        \"numReadonlyUnsignedAccounts\":9
                    },
                    \"accountKeys\":[
                        \"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\",
                        \"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                        \"JB4vdpYFSG4xCqeZbMC8r96H81nB7oi2xBdMmVBGWWyy\",
                        \"Aurdw9mjPnBMQCiczdN4H7qfSoHF8K915GfSi364SZgA\",
                        \"DV2rLHZsXZLTJzfQ3iUQoKxqX8phM8hR4qjgxtqRV81W\",
                        \"6DnkBtW5UmsWRFCZBkihS1yZzUWWKpUZiHUwMPDx6c9C\",
                        \"Eozy2f2NoxvuRJcFdif8ma3rAuWvHJte937NEWH3Fhwr\",
                        \"CG18v8fAZusKkMzZp7kLbCpsYrDkLVDmqhbXu5v7hHwZ\",
                        \"FwGMDsTRbf6fNTb9YSN6HorTPEPhcLCG7H9zFEicm61u\",
                        \"8mkxhojbDFkzofuPjesqaakcGZvfA72GaSVEXXFsEemq\",
                        \"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                        \"9XQJeiCUAN4oZyBrG8x6kAHi4cszz6L4kjnGZGR2fsWs\",
                        \"SysvarRent111111111111111111111111111111111\",
                        \"11111111111111111111111111111111\",
                        \"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",
                        \"H6FEUafrGDeQsGnCerFomtzG3B3TctUaue8yM7heLi8W\",
                        \"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\",
                        \"rndshKFf48HhGaPbaCd3WQYtgCNKzRgVQ3U2we4Cvf9\",
                        \"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s\",
                        \"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL\",
                        \"packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu\"
                    ],
                    \"recentBlockhash\":\"2JpSV2YKxT9dhMtHCcEVPFQi4WMVNDSL8QW9Xqb4Jrd4\",
                    \"instructions\":[
                        {
                            \"programIdIndex\":11,
                            \"accounts\":[0,1],
                            \"data\":\"11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC\"
                        }                    
                    ]
                }
            },
            \"meta\":{
                \"err\":null,
                \"status\":{
                    \"Ok\":null
                },
                \"fee\":10000,
                \"preBalances\":[
                    501683013,0,0,7168800,1900080,2039280,0,0,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"postBalances\":[
                    489987173,1461600,2039280,7168800,1900080,2039280,5616720,2568240,2853600,5616720,1461600,1113600,1009200,1,953185920,7050480,0,1398960,1141440,898174080,1141440
                ],
                \"innerInstructions\":[
                    {
                        \"index\":2,
                        \"instructions\":[
                            {
                                \"programIdIndex\":2,
                                \"accounts\":[
                                    0,3
                                ],
                                \"data\":\"3Bxs4h24hBtQy9rw\"
                            }                       
                        ]
                    }               
                ],
                \"logMessages\":[
                ],
                \"preTokenBalances\":[
                    {
                        \"accountIndex\":5,
                        \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                        \"uiTokenAmount\":
                        {
                            \"uiAmount\":1.0,
                            \"decimals\":0,
                            \"amount\":\"1\",
                            \"uiAmountString\":\"1\"
                        },
                        \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                    }
                ],
                \"postTokenBalances\":[
                    {
                        \"accountIndex\":2,
                        \"mint\":\"E29Nen991Z4Gin11wxNV3Nq8xJh5a1nYbGAYBgZDLCB8\",
                        \"uiTokenAmount\":
                        {
                            \"uiAmount\":1.0,
                            \"decimals\":0,
                            \"amount\":\"1\",
                            \"uiAmountString\":\"1\"
                        },
                        \"owner\":\"GXzqybrSAbDmALLJQFKZMMdib7QPBTavyGatoAGtEmPm\"
                    },{
                        \"accountIndex\":5,
                        \"mint\":\"BNFSDxJuDPM6EYKKZGs5pcR9HYu8t2UjSe18ZUTaBkgM\",
                        \"uiTokenAmount\":
                        {
                            \"uiAmount\":1.0,
                            \"decimals\":0,
                            \"amount\":\"1\",
                            \"uiAmountString\":\"1\"
                        },
                        \"owner\":\"4wawb6MxhWmANe4nDYB7Hy5tdFY3A5s1MyNSJHShnjz\"
                    }
                ],
                \"rewards\":[]
            }
        }";

        let encoded_confirmed_transaction = EncodedConfirmedTransactionWithStatusMeta {
            slot: 117946133_u64,
            transaction: serde_json::from_str(encoded_transaction).unwrap(),
            block_time: Some(1643213404_i64),
        };

        let mut transaction_parser = TransactionParserHandle::new().await;
        let parsed_transaction = transaction_parser
            .parse_transaction(encoded_confirmed_transaction)
            .await
            .unwrap();

        println!("PREKOL: {:#?}", parsed_transaction.0[0]);

        assert_eq!(parsed_transaction.0.len(), 2);
        assert_eq!(parsed_transaction.0[0].instruction_name, "".to_string());
        assert_eq!(
            parsed_transaction.0[0].data,
            "11114XtYk9gGfZoo968fyjNUYQJKf9gdmkGoaoBpzFv4vyaSMBn3VKxZdv7mZLzoyX5YNC".to_string()
        );

        assert_eq!(parsed_transaction.0[1].instruction_name, "".to_string());
        assert_eq!(parsed_transaction.0[1].data, "3Bxs4h24hBtQy9rw".to_string());
    }
}
