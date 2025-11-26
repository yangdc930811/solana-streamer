use sol_common::protocols::meteora_dlmm::extensions::BinArrayBitmapExtension;
use sol_common::protocols::meteora_dlmm::types::LbPair;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::meteora_dlmm::events::{MeteoraDlmmBinArrayBitmapExtensionAccountEvent, MeteoraDlmmPoolAccountEvent};
use crate::streaming::grpc::AccountPretty;

pub const POOL_SIZE: usize = 896;

pub fn pool_decode(data: &[u8]) -> Option<LbPair> {
    if data.len() < POOL_SIZE {
        return None;
    }
    borsh::from_slice::<LbPair>(&data[..POOL_SIZE]).ok()
}

pub fn direct_pool_decode(data: &[u8]) -> Option<LbPair> {
    if data.len() < POOL_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<LbPair>(&data[8..POOL_SIZE + 8]).ok()
}

pub fn pool_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountMeteoraDlmmPool;

    if account.data.len() < POOL_SIZE + 8 {
        return None;
    }
    if let Some(pool) = pool_decode(&account.data[8..POOL_SIZE + 8]) {
        Some(DexEvent::MeteoraDlmmPoolAccountEvent(MeteoraDlmmPoolAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool,
        }))
    } else {
        None
    }
}

pub const BIN_ARRAY_BITMAP_EXTENSION_SIZE: usize = 1568;

pub fn direct_bin_array_bitmap_extension_decode(data: &[u8]) -> Option<BinArrayBitmapExtension> {
    if data.len() < BIN_ARRAY_BITMAP_EXTENSION_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<BinArrayBitmapExtension>(&data[8..BIN_ARRAY_BITMAP_EXTENSION_SIZE + 8]).ok()
}

pub fn bin_array_bitmap_extension_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountMeteoraDlmmBinArrayBitmapExtension;

    if let Some(bin_array_bitmap_extension) = direct_bin_array_bitmap_extension_decode(&account.data) {
        Some(DexEvent::MeteoraDlmmBinArrayBitmapExtensionAccountEvent(MeteoraDlmmBinArrayBitmapExtensionAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            bin_array_bitmap_extension,
        }))
    } else {
        None
    }
}