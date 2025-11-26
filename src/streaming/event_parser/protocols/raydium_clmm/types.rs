use sol_common::protocols::raydium_clmm::states::config::AmmConfig;
use sol_common::protocols::raydium_clmm::states::pool::PoolState;
use sol_common::protocols::raydium_clmm::states::tick_array::TickArrayState;
use sol_common::protocols::raydium_clmm::states::tickarray_bitmap_extension::TickArrayBitmapExtension;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::raydium_clmm::{RaydiumClmmAmmConfigAccountEvent, RaydiumClmmPoolStateAccountEvent, RaydiumClmmTickArrayBitmapExtensionAccountEvent, RaydiumClmmTickArrayStateAccountEvent};
use crate::streaming::grpc::AccountPretty;

pub const POOL_STATE_SIZE: usize = 1536;

pub fn pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[..POOL_STATE_SIZE]).ok()
}

pub fn direct_pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[8..POOL_STATE_SIZE + 8]).ok()
}

pub fn pool_state_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmPoolState;

    if account.data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    if let Some(pool_state) = pool_state_decode(&account.data[8..POOL_STATE_SIZE + 8]) {
        Some(DexEvent::RaydiumClmmPoolStateAccountEvent(RaydiumClmmPoolStateAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool_state: pool_state,
        }))
    } else {
        None
    }
}

pub const AMM_CONFIG_SIZE: usize = 1 + 2 + 32 + 4 * 2 + 2 + 4 * 2 + 32 + 8 * 3;

pub fn amm_config_decode(data: &[u8]) -> Option<AmmConfig> {
    if data.len() < AMM_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<AmmConfig>(&data[..AMM_CONFIG_SIZE]).ok()
}

pub fn amm_config_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmAmmConfig;

    if account.data.len() < AMM_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(amm_config) = amm_config_decode(&account.data[8..AMM_CONFIG_SIZE + 8]) {
        Some(DexEvent::RaydiumClmmAmmConfigAccountEvent(RaydiumClmmAmmConfigAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            amm_config: amm_config,
        }))
    } else {
        None
    }
}

pub const TICK_ARRAY_BITMAP_EXTENSION_SIZE: usize = 1824;

pub fn tick_array_bitmap_extension_decode(data: &[u8]) -> Option<TickArrayBitmapExtension> {
    if data.len() < TICK_ARRAY_BITMAP_EXTENSION_SIZE {
        return None;
    }
    borsh::from_slice::<TickArrayBitmapExtension>(&data[..TICK_ARRAY_BITMAP_EXTENSION_SIZE]).ok()
}

pub fn direct_tick_array_bitmap_extension_decode(data: &[u8]) -> Option<TickArrayBitmapExtension> {
    if data.len() < TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<TickArrayBitmapExtension>(&data[8..TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8]).ok()
}

pub fn tick_array_bitmap_extension_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmTickArrayBitmapExtension;

    if account.data.len() < TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8 {
        return None;
    }
    if let Some(tick_array_bitmap_extension) =
        tick_array_bitmap_extension_decode(&account.data[8..TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8])
    {
        Some(DexEvent::RaydiumClmmTickArrayBitmapExtensionAccountEvent(
            RaydiumClmmTickArrayBitmapExtensionAccountEvent {
                metadata,
                pubkey: account.pubkey,
                executable: account.executable,
                lamports: account.lamports,
                owner: account.owner,
                rent_epoch: account.rent_epoch,
                tick_array_bitmap_extension,
            },
        ))
    } else {
        None
    }
}

pub const TICK_ARRAY_STATE_SIZE: usize = 10232;

pub fn tick_array_state_decode(data: &[u8]) -> Option<TickArrayState> {
    if data.len() < TICK_ARRAY_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<TickArrayState>(&data[..TICK_ARRAY_STATE_SIZE]).ok()
}

pub fn tick_array_state_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmTickArrayState;

    if account.data.len() < TICK_ARRAY_STATE_SIZE + 8 {
        return None;
    }
    if let Some(tick_array_state) =
        tick_array_state_decode(&account.data[8..TICK_ARRAY_STATE_SIZE + 8])
    {
        Some(DexEvent::RaydiumClmmTickArrayStateAccountEvent(
            RaydiumClmmTickArrayStateAccountEvent {
                metadata,
                pubkey: account.pubkey,
                executable: account.executable,
                lamports: account.lamports,
                owner: account.owner,
                rent_epoch: account.rent_epoch,
                tick_array_state,
            },
        ))
    } else {
        None
    }
}