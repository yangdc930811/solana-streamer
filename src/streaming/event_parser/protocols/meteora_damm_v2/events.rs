use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::common::EventMetadata;
use borsh::BorshDeserialize;
use crate::streaming::event_parser::protocols::meteora_damm_v2::types::Pool;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SwapParameters {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SwapParameters2 {
    /// When it's exact in, partial fill, this will be amount_in. When it's exact out, this will be amount_out
    pub amount_0: u64,
    /// When it's exact in, partial fill, this will be minimum_amount_out. When it's exact out, this will be maximum_amount_in
    pub amount_1: u64,
    /// Swap mode, refer [SwapMode]
    pub swap_mode: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SwapResult {
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SwapResult2 {
    // This is excluded_transfer_fee_amount_in
    pub included_fee_input_amount: u64,
    pub excluded_fee_input_amount: u64,
    pub amount_left: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2SwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub trade_direction: u8,
    pub has_referral: bool,
    pub params: SwapParameters,
    pub swap_result: SwapResult,
    pub actual_amount_in: u64,
    pub current_timestamp: u64,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub token_a_mint: Pubkey,
    #[borsh(skip)]
    pub token_b_mint: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2Swap2Event {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub trade_direction: u8,
    pub collect_fee_mode: u8,
    pub has_referral: bool,
    pub params: SwapParameters2,
    // excluded_transfer_fee_amount_in is swap_result.included_fee_amount_in
    pub swap_result: SwapResult2,
    pub included_transfer_fee_amount_in: u64,
    pub included_transfer_fee_amount_out: u64,
    pub excluded_transfer_fee_amount_out: u64,
    pub current_timestamp: u64,
    pub reserve_a_amount: u64,
    pub reserve_b_amount: u64,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub token_a_mint: Pubkey,
    #[borsh(skip)]
    pub token_b_mint: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
}

// 池
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2PoolAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool: Pool,
}

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    // pub const SWAP_EVENT: &[u8] = &[27, 60, 21, 213, 138, 170, 187, 147];
    // pub const SWAP2_EVENT: &[u8] = &[189, 66, 51, 168, 38, 80, 117, 153];

    // Instruction discriminators
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];
    pub const SWAP2: &[u8] = &[65, 75, 63, 76, 235, 91, 91, 136];

    // 账户鉴别器
    pub const POOL_ACCOUNT: &[u8] = &[241, 154, 109, 4, 17, 177, 109, 188];
}