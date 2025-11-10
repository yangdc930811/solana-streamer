use borsh_derive::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::orca::types::Whirlpool;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct OrcaSwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub whirlpool: Pubkey,
    #[borsh(skip)]
    pub token_owner_account_a: Pubkey,
    #[borsh(skip)]
    pub token_vault_a: Pubkey,
    #[borsh(skip)]
    pub token_owner_account_b: Pubkey,
    #[borsh(skip)]
    pub token_vault_b: Pubkey,
    #[borsh(skip)]
    pub oracle: Pubkey,
}

// 池
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct OrcaPoolAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool: Whirlpool,
}

pub mod discriminators {
    // Instruction discriminators
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];

    pub const POOL_ACCOUNT: &[u8] = &[63, 149, 209, 12, 225, 128, 99, 9];
}