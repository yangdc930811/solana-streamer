use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use sol_common::protocols::raydium_cpmm::{AmmConfig, PoolState};
use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::raydium_cpmm::{
            RaydiumCpmmAmmConfigAccountEvent, RaydiumCpmmPoolStateAccountEvent,
        },
        DexEvent,
    },
    grpc::AccountPretty,
};

pub const AMM_CONFIG_SIZE: usize = 228;

pub fn amm_config_decode(data: &[u8]) -> Option<AmmConfig> {
    if data.len() < AMM_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<AmmConfig>(&data[..AMM_CONFIG_SIZE]).ok()
}

pub fn amm_config_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumCpmmAmmConfig;

    if account.data.len() < AMM_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(amm_config) = amm_config_decode(&account.data[8..AMM_CONFIG_SIZE + 8]) {
        Some(DexEvent::RaydiumCpmmAmmConfigAccountEvent(RaydiumCpmmAmmConfigAccountEvent {
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

pub const POOL_STATE_SIZE: usize = 629;

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
    metadata.event_type = EventType::AccountRaydiumCpmmPoolState;

    if account.data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    if let Some(pool_state) = pool_state_decode(&account.data[8..POOL_STATE_SIZE + 8]) {
        Some(DexEvent::RaydiumCpmmPoolStateAccountEvent(RaydiumCpmmPoolStateAccountEvent {
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
