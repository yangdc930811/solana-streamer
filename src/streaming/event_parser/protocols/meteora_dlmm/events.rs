use borsh_derive::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use sol_common::protocols::meteora_dlmm::extensions::{BinArray, BinArrayBitmapExtension};
use sol_common::protocols::meteora_dlmm::types::LbPair;
use crate::streaming::event_parser::common::EventMetadata;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDlmmSwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub lb_pair: Pubkey,
    pub from: Pubkey,
    pub start_bin_id: i32,
    pub end_bin_id: i32,
    pub amount_in: u64,
    pub amount_out: u64,
    pub swap_for_y: bool,
    pub fee: u64,
    pub protocol_fee: u64,
    pub fee_bps: u128,
    pub host_fee: u64,
    #[borsh(skip)]
    pub reserve_x: Pubkey,
    #[borsh(skip)]
    pub reserve_y: Pubkey,
    #[borsh(skip)]
    pub token_x_mint: Pubkey,
    #[borsh(skip)]
    pub token_y_mint: Pubkey,
    #[borsh(skip)]
    pub oracle: Pubkey,
    #[borsh(skip)]
    pub token_x_program: Pubkey,
    #[borsh(skip)]
    pub token_y_program: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
}

// 池
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDlmmPoolAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool: LbPair,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmBinArrayBitmapExtensionAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub bin_array_bitmap_extension: BinArrayBitmapExtension,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmBinArrayAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub bin_array: BinArray,
}

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    pub const SWAP_EVENT: &[u8] = &[228, 69, 165, 46, 81, 203, 154, 29, 81, 108, 227, 190, 205, 208, 10, 196];

    // Instruction discriminators
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];

    pub const POOL_ACCOUNT: &[u8] = &[33, 11, 49, 98, 181, 101, 177, 13];

    pub const BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT: &[u8] = &[80, 111, 124, 113, 55, 237, 18, 5];

    pub const BIN_ARRAY_ACCOUNT: &[u8] = &[92, 142, 92, 220, 5, 148, 70, 181];
}

pub const METEORA_DLMM_SWAP_EVENT_LOG_SIZE: usize = 129;

pub fn meteora_dlmm_swap_event_log_decode(data: &[u8]) -> Option<MeteoraDlmmSwapEvent> {
    if data.len() < METEORA_DLMM_SWAP_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<MeteoraDlmmSwapEvent>(&data[..METEORA_DLMM_SWAP_EVENT_LOG_SIZE]).ok()
}