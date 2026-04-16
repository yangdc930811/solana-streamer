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

/// Legacy pool account body (before `is_cashback_coin` + reserved).
pub const POOL_BODY_LEGACY: usize = 1 + 2 + 32 * 6 + 8 + 32 + 1;
/// Current pool account body including flags and reserved.
pub const POOL_BODY: usize = POOL_BODY_LEGACY + 1 + 7;

pub const POOL_SIZE: usize = POOL_BODY;

pub fn pool_decode(data: &[u8]) -> Option<Pool> {
    if data.len() >= POOL_BODY {
        return borsh::from_slice::<Pool>(&data[..POOL_BODY]).ok();
    }
    if data.len() < POOL_BODY_LEGACY {
        return None;
    }
    let legacy = borsh::from_slice::<PoolLegacy>(&data[..POOL_BODY_LEGACY]).ok()?;
    Some(legacy.into())
}

/// Pre-cashback on-chain layout (Borsh-compatible prefix of `Pool`).
#[derive(Clone, Debug, BorshDeserialize)]
struct PoolLegacy {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
}

impl From<PoolLegacy> for Pool {
    fn from(p: PoolLegacy) -> Self {
        Pool {
            pool_bump: p.pool_bump,
            index: p.index,
            creator: p.creator,
            base_mint: p.base_mint,
            quote_mint: p.quote_mint,
            lp_mint: p.lp_mint,
            pool_base_token_account: p.pool_base_token_account,
            pool_quote_token_account: p.pool_quote_token_account,
            lp_supply: p.lp_supply,
            coin_creator: p.coin_creator,
            is_mayhem_mode: p.is_mayhem_mode,
            is_cashback_coin: false,
            reserved: [0u8; 7],
        }
    }
}

pub fn direct_pool_decode(data: &[u8]) -> Option<Pool> {
    if data.len() < POOL_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<Pool>(&data[8..POOL_SIZE + 8]).ok()
}

pub fn pool_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountPumpSwapPool;

    let body = account.data.get(8..)?;
    if body.len() < POOL_BODY_LEGACY {
        return None;
    }
    if let Some(pool) = pool_decode(body) {
        Some(DexEvent::PumpSwapPoolAccountEvent(PumpSwapPoolAccountEvent {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_decode_legacy_and_extended() {
        let keys: Vec<Pubkey> = (0..6).map(|_| Pubkey::new_unique()).collect();
        let coin = Pubkey::new_unique();
        let mut legacy = Vec::new();
        legacy.push(9u8);
        legacy.extend_from_slice(&7u16.to_le_bytes());
        for k in &keys {
            legacy.extend_from_slice(k.as_ref());
        }
        legacy.extend_from_slice(&99u64.to_le_bytes());
        legacy.extend_from_slice(coin.as_ref());
        legacy.push(1u8);

        let p = pool_decode(&legacy).expect("legacy");
        assert_eq!(p.pool_bump, 9);
        assert_eq!(p.index, 7);
        assert_eq!(p.lp_supply, 99);
        assert!(p.is_mayhem_mode);
        assert!(!p.is_cashback_coin);
        assert_eq!(p.reserved, [0u8; 7]);

        let mut ext = legacy.clone();
        ext.push(1u8);
        ext.extend_from_slice(&[2u8, 3, 4, 5, 6, 7, 8]);

        let p2 = pool_decode(&ext).expect("extended");
        assert!(p2.is_cashback_coin);
        assert_eq!(p2.reserved, [2, 3, 4, 5, 6, 7, 8]);
    }
}
