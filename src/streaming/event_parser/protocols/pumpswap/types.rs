use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use sol_common::protocols::pumpswap::{GlobalConfig, Pool};
use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::pumpswap::{PumpSwapGlobalConfigAccountEvent, PumpSwapPoolAccountEvent},
        DexEvent,
    },
    grpc::AccountPretty,
};

pub const GLOBAL_CONFIG_SIZE: usize = 32 + 8 + 8 + 1 + 32 * 8 + 8 + 32 + 32 + 32 + 1 + 32 * 7;

pub fn global_config_decode(data: &[u8]) -> Option<GlobalConfig> {
    if data.len() < GLOBAL_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<GlobalConfig>(&data[..GLOBAL_CONFIG_SIZE]).ok()
}

pub fn global_config_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountPumpSwapGlobalConfig;

    if account.data.len() < GLOBAL_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(config) = global_config_decode(&account.data[8..GLOBAL_CONFIG_SIZE + 8]) {
        Some(DexEvent::PumpSwapGlobalConfigAccountEvent(PumpSwapGlobalConfigAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            global_config: config,
        }))
    } else {
        None
    }
}

pub const POOL_SIZE: usize = 1 + 2 + 32 * 6 + 8 + 32 + 1;

pub fn pool_decode(data: &[u8]) -> Option<Pool> {
    if data.len() < POOL_SIZE {
        return None;
    }
    borsh::from_slice::<Pool>(&data[..POOL_SIZE]).ok()
}

pub fn direct_pool_decode(data: &[u8]) -> Option<Pool> {
    if data.len() < POOL_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<Pool>(&data[8..POOL_SIZE + 8]).ok()
}

pub fn pool_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountPumpSwapPool;

    if account.data.len() < POOL_SIZE + 8 {
        return None;
    }
    if let Some(pool) = pool_decode(&account.data[8..POOL_SIZE + 8]) {
        Some(DexEvent::PumpSwapPoolAccountEvent(PumpSwapPoolAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool: pool,
        }))
    } else {
        None
    }
}
