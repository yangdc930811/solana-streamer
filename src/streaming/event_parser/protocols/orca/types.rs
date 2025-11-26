use borsh_derive::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use sol_common::protocols::orca::Whirlpool;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::orca::events::OrcaPoolAccountEvent;
use crate::streaming::grpc::AccountPretty;

pub const POOL_SIZE: usize = 645;

pub fn pool_decode(data: &[u8]) -> Option<Whirlpool> {
    if data.len() < POOL_SIZE {
        return None;
    }
    borsh::from_slice::<Whirlpool>(&data[..POOL_SIZE]).ok()
}

pub fn direct_pool_decode(data: &[u8]) -> Option<Whirlpool> {
    if data.len() < POOL_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<Whirlpool>(&data[8..POOL_SIZE + 8]).ok()
}

pub fn pool_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountOrcaPool;

    if account.data.len() < POOL_SIZE + 8 {
        return None;
    }
    if let Some(pool) = pool_decode(&account.data[8..POOL_SIZE + 8]) {
        Some(DexEvent::OrcaPoolAccountEvent(OrcaPoolAccountEvent {
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