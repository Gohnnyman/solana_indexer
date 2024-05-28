use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct WithdrawFromFee {
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct WithdrawFromTreasury {
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct UpdateAuctionHouse {
    seller_fee_basis_points: Option<u16>,
    requires_sign_off: Option<bool>,
    can_change_sale_price: Option<bool>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct CreateAuctionHouse {
    _bump: u8,
    fee_payer_bump: u8,
    treasury_bump: u8,
    seller_fee_basis_points: u16,
    requires_sign_off: bool,
    can_change_sale_price: bool,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Withdraw {
    escrow_payment_bump: u8,
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Deposit {
    escrow_payment_bump: u8,
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Cancel {
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ExecuteSale {
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Sell {
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct PublicBuy {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct CloseEscrowAccount {
    escrow_payment_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct PrintListingReceipt {
    receipt_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct PrintBidReceipt {
    receipt_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct PrintPurchaseReceipt {
    purchase_receipt_bump: u8,
}
#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Buy {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerBuy {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerPublicBuy {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerCancel {
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerExecuteSale {
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerDeposit {
    escrow_payment_bump: u8,
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerExecutePartialSale {
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
    partial_order_size: Option<u64>,
    partial_order_price: Option<u64>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ExecutePartialSale {
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
    partial_order_size: Option<u64>,
    partial_order_price: Option<u64>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct DelegateAuctioneer {
    scopes: Vec<AuthorityScope>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct UpdateAuctioneer {
    scopes: Vec<AuthorityScope>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerWithdraw {
    escrow_payment_bump: u8,
    amount: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct AuctioneerSell {
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    token_size: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse(InstrRoot)]
pub enum AuctionHouseInstruction {
    WithdrawFromFee {
        amount: u64,
    },
    WithdrawFromTreasury {
        amount: u64,
    },
    UpdateAuctionHouse {
        seller_fee_basis_points: Option<u16>,
        requires_sign_off: Option<bool>,
        can_change_sale_price: Option<bool>,
    },
    CreateAuctionHouse {
        _bump: u8,
        fee_payer_bump: u8,
        treasury_bump: u8,
        seller_fee_basis_points: u16,
        requires_sign_off: bool,
        can_change_sale_price: bool,
    },
    Buy {
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    AuctioneerBuy {
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    PublicBuy {
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    AuctioneerPublicBuy {
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    Cancel {
        buyer_price: u64,
        token_size: u64,
    },
    AuctioneerCancel {
        buyer_price: u64,
        token_size: u64,
    },
    Deposit {
        escrow_payment_bump: u8,
        amount: u64,
    },
    AuctioneerDeposit {
        escrow_payment_bump: u8,
        amount: u64,
    },
    ExecuteSale {
        escrow_payment_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    ExecutePartialSale {
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
        partial_order_size: Option<u64>,
        partial_order_price: Option<u64>,
    },
    AuctioneerExecuteSale {
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    AuctioneerExecutePartialSale {
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
        partial_order_size: Option<u64>,
        partial_order_price: Option<u64>,
    },
    Sell {
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    },
    AuctioneerSell {
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        token_size: u64,
    },
    Withdraw {
        escrow_payment_bump: u8,
        amount: u64,
    },
    AuctioneerWithdraw {
        escrow_payment_bump: u8,
        amount: u64,
    },
    CloseEscrowAccount {
        escrow_payment_bump: u8,
    },
    DelegateAuctioneer {
        scopes: Vec<AuthorityScope>,
    },
    UpdateAuctioneer {
        scopes: Vec<AuthorityScope>,
    },
    PrintListingReceipt {
        receipt_bump: u8,
    },
    CancelListingReceipt,
    PrintBidReceipt {
        receipt_bump: u8,
    },
    CancelBidReceipt,
    PrintPurchaseReceipt {
        purchase_receipt_bump: u8,
    },
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub enum AuthorityScope {
    Deposit = 0,
    Buy = 1,
    PublicBuy = 2,
    ExecuteSale = 3,
    Sell = 4,
    Cancel = 5,
    Withdraw = 6,
}

#[cfg(test)]
pub mod tests {
    use solana_sdk::bs58;

    use super::*;

    #[test]
    fn test_aboba() {
        let bytes = "23s1EunhJwrywvGXMGTeqdk5KNRtPKxKAJm6jWr";
        let data = bs58::decode(bytes).into_vec().unwrap();
        println!("DATA: {:?}", data);

        let gues_instr = Buy {
            trade_state_bump: 255,
            escrow_payment_bump: 253,
            // buyer_price: u64::from_le_bytes([0, 233, 164, 53, 0, 0, 0, 0]),
            // token_size: u64::from_le_bytes([1, 0, 0, 0, 0, 0, 0, 0]),
            buyer_price: 900000000,
            token_size: 1,
        };

        let mut sighash: Vec<u8> = [102, 6, 61, 18, 1, 218, 235, 234].to_vec();
        sighash.append(&mut gues_instr.try_to_vec().unwrap());

        println!("GUES: {:?}", sighash);

        let instr: Buy = Buy::try_from_slice(&data).unwrap();
    }
}

impl AuctionHouseInstruction {
    pub fn match_sighash(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<AuctionHouseInstruction, ParseInstructionError> {
        match sighash {
            [179, 208, 190, 154, 32, 179, 19, 59] => {
                let withdraw_from_fee = WithdrawFromFee::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::WithdrawFromFee {
                    amount: withdraw_from_fee.amount,
                })
            }
            [0, 164, 86, 76, 56, 72, 12, 170] => {
                let withdraw_from_treasury = WithdrawFromTreasury::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::WithdrawFromTreasury {
                    amount: withdraw_from_treasury.amount,
                })
            }
            [84, 215, 2, 172, 241, 0, 245, 219] => {
                let update_auction_house = UpdateAuctionHouse::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::UpdateAuctionHouse {
                    seller_fee_basis_points: update_auction_house.seller_fee_basis_points,
                    requires_sign_off: update_auction_house.requires_sign_off,
                    can_change_sale_price: update_auction_house.can_change_sale_price,
                })
            }
            [221, 66, 242, 159, 249, 206, 134, 241] => {
                let create_auction_house = CreateAuctionHouse::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::CreateAuctionHouse {
                    _bump: create_auction_house._bump,
                    fee_payer_bump: create_auction_house.fee_payer_bump,
                    treasury_bump: create_auction_house.treasury_bump,
                    seller_fee_basis_points: create_auction_house.seller_fee_basis_points,
                    requires_sign_off: create_auction_house.requires_sign_off,
                    can_change_sale_price: create_auction_house.can_change_sale_price,
                })
            }
            [102, 6, 61, 18, 1, 218, 235, 234] => {
                let data = &data[0..18];

                let buy = Buy::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::Buy {
                    trade_state_bump: buy.trade_state_bump,
                    escrow_payment_bump: buy.escrow_payment_bump,
                    buyer_price: buy.buyer_price,
                    token_size: buy.token_size,
                })
            }
            [17, 106, 133, 46, 229, 48, 45, 208] => {
                let auctioneer_buy = AuctioneerBuy::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerBuy {
                    trade_state_bump: auctioneer_buy.trade_state_bump,
                    escrow_payment_bump: auctioneer_buy.escrow_payment_bump,
                    buyer_price: auctioneer_buy.buyer_price,
                    token_size: auctioneer_buy.token_size,
                })
            }
            [169, 84, 218, 35, 42, 206, 16, 171] => {
                let public_buy = PublicBuy::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::PublicBuy {
                    trade_state_bump: public_buy.trade_state_bump,
                    escrow_payment_bump: public_buy.escrow_payment_bump,
                    buyer_price: public_buy.buyer_price,
                    token_size: public_buy.token_size,
                })
            }
            [221, 239, 99, 240, 86, 46, 213, 126] => {
                let auctioneer_public_buy = AuctioneerPublicBuy::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerPublicBuy {
                    trade_state_bump: auctioneer_public_buy.trade_state_bump,
                    escrow_payment_bump: auctioneer_public_buy.escrow_payment_bump,
                    buyer_price: auctioneer_public_buy.buyer_price,
                    token_size: auctioneer_public_buy.token_size,
                })
            }
            [232, 219, 223, 41, 219, 236, 220, 190] => {
                let cancel = Cancel::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::Cancel {
                    buyer_price: cancel.buyer_price,
                    token_size: cancel.token_size,
                })
            }
            [197, 97, 152, 196, 115, 204, 64, 215] => {
                let auctioneer_cancel = AuctioneerCancel::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerCancel {
                    buyer_price: auctioneer_cancel.buyer_price,
                    token_size: auctioneer_cancel.token_size,
                })
            }
            [242, 35, 198, 137, 82, 225, 242, 182] => {
                let deposit = Deposit::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::Deposit {
                    escrow_payment_bump: deposit.escrow_payment_bump,
                    amount: deposit.amount,
                })
            }
            [79, 122, 37, 162, 120, 173, 57, 127] => {
                let auctioneer_deposit = AuctioneerDeposit::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerDeposit {
                    escrow_payment_bump: auctioneer_deposit.escrow_payment_bump,
                    amount: auctioneer_deposit.amount,
                })
            }
            [37, 74, 217, 157, 79, 49, 35, 6] => {
                let execute_sale = ExecuteSale::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::ExecuteSale {
                    escrow_payment_bump: execute_sale.escrow_payment_bump,
                    free_trade_state_bump: execute_sale._free_trade_state_bump,
                    program_as_signer_bump: execute_sale.program_as_signer_bump,
                    buyer_price: execute_sale.buyer_price,
                    token_size: execute_sale.token_size,
                })
            }
            [163, 18, 35, 157, 49, 164, 203, 133] => {
                let execute_partial_sale = ExecutePartialSale::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::ExecutePartialSale {
                    escrow_payment_bump: execute_partial_sale.escrow_payment_bump,
                    _free_trade_state_bump: execute_partial_sale._free_trade_state_bump,
                    program_as_signer_bump: execute_partial_sale.program_as_signer_bump,
                    buyer_price: execute_partial_sale.buyer_price,
                    token_size: execute_partial_sale.token_size,
                    partial_order_size: execute_partial_sale.partial_order_size,
                    partial_order_price: execute_partial_sale.partial_order_price,
                })
            }
            [68, 125, 32, 65, 251, 43, 35, 53] => {
                let auctioneer_execute_sale = AuctioneerExecuteSale::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerExecuteSale {
                    escrow_payment_bump: auctioneer_execute_sale.escrow_payment_bump,
                    _free_trade_state_bump: auctioneer_execute_sale._free_trade_state_bump,
                    program_as_signer_bump: auctioneer_execute_sale.program_as_signer_bump,
                    buyer_price: auctioneer_execute_sale.buyer_price,
                    token_size: auctioneer_execute_sale.token_size,
                })
            }
            [9, 44, 46, 15, 161, 143, 21, 54] => {
                let auctioneer_execute_partial_sale =
                    AuctioneerExecutePartialSale::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerExecutePartialSale {
                    escrow_payment_bump: auctioneer_execute_partial_sale.escrow_payment_bump,
                    _free_trade_state_bump: auctioneer_execute_partial_sale._free_trade_state_bump,
                    program_as_signer_bump: auctioneer_execute_partial_sale.program_as_signer_bump,
                    buyer_price: auctioneer_execute_partial_sale.buyer_price,
                    token_size: auctioneer_execute_partial_sale.token_size,
                    partial_order_size: auctioneer_execute_partial_sale.partial_order_size,
                    partial_order_price: auctioneer_execute_partial_sale.partial_order_price,
                })
            }
            [51, 230, 133, 164, 1, 127, 131, 173] => {
                let sell = Sell::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::Sell {
                    trade_state_bump: sell.trade_state_bump,
                    free_trade_state_bump: sell.free_trade_state_bump,
                    program_as_signer_bump: sell.program_as_signer_bump,
                    buyer_price: sell.buyer_price,
                    token_size: sell.token_size,
                })
            }
            [251, 60, 142, 195, 121, 203, 26, 183] => {
                let auctioneer_sell = AuctioneerSell::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerSell {
                    trade_state_bump: auctioneer_sell.trade_state_bump,
                    free_trade_state_bump: auctioneer_sell.free_trade_state_bump,
                    program_as_signer_bump: auctioneer_sell.program_as_signer_bump,
                    token_size: auctioneer_sell.token_size,
                })
            }
            [183, 18, 70, 156, 148, 109, 161, 34] => {
                let withdraw = Withdraw::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::Withdraw {
                    escrow_payment_bump: withdraw.escrow_payment_bump,
                    amount: withdraw.amount,
                })
            }
            [85, 166, 219, 110, 168, 143, 180, 236] => {
                let auctioneer_withdraw = AuctioneerWithdraw::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::AuctioneerWithdraw {
                    escrow_payment_bump: auctioneer_withdraw.escrow_payment_bump,
                    amount: auctioneer_withdraw.amount,
                })
            }
            [209, 42, 208, 179, 140, 78, 18, 43] => {
                let close_escrow_account = CloseEscrowAccount::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::CloseEscrowAccount {
                    escrow_payment_bump: close_escrow_account.escrow_payment_bump,
                })
            }
            [106, 178, 12, 122, 74, 173, 251, 222] => {
                let delegate_auctioneer = DelegateAuctioneer::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::DelegateAuctioneer {
                    scopes: delegate_auctioneer.scopes,
                })
            }
            [103, 255, 80, 234, 94, 56, 168, 208] => {
                let update_auctioneer = UpdateAuctioneer::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::UpdateAuctioneer {
                    scopes: update_auctioneer.scopes,
                })
            }
            [207, 107, 44, 160, 75, 222, 195, 27] => {
                let print_listing_receipt = PrintListingReceipt::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::PrintListingReceipt {
                    receipt_bump: print_listing_receipt.receipt_bump,
                })
            }
            [171, 59, 138, 126, 246, 189, 91, 11] => {
                Ok(AuctionHouseInstruction::CancelListingReceipt)
            }
            [94, 249, 90, 230, 239, 64, 68, 218] => {
                let print_bid_receipt = PrintBidReceipt::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::PrintBidReceipt {
                    receipt_bump: print_bid_receipt.receipt_bump,
                })
            }
            [246, 108, 27, 229, 220, 42, 176, 43] => Ok(AuctionHouseInstruction::CancelBidReceipt),
            [227, 154, 251, 7, 180, 56, 100, 143] => {
                let print_purchase_receipt = PrintPurchaseReceipt::try_from_slice(data)?;
                Ok(AuctionHouseInstruction::PrintPurchaseReceipt {
                    purchase_receipt_bump: print_purchase_receipt.purchase_receipt_bump,
                })
            }
            _ => Err(ParseInstructionError::SighashMatchError(
                "Auction House".to_string(),
            )),
        }
    }

    pub fn parse_instruction(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = Self::match_sighash(sighash, data);

        let instruction = match instruction {
            Err(ParseInstructionError::DeserializeError(err)) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Auction House".to_string(),
                    err,
                });
            }
            _ => instruction,
        }?;

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }
}
