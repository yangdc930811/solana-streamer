use sol_common::protocols::meteora_damm_v2::types::Pool;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::MeteoraDammV2PoolAccountEvent;
use crate::streaming::grpc::AccountPretty;

pub const POOL_SIZE: usize = 1104;

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
    metadata.event_type = EventType::AccountMeteoraDammV2Pool;

    if account.data.len() < POOL_SIZE + 8 {
        return None;
    }
    if let Some(pool) = pool_decode(&account.data[8..POOL_SIZE + 8]) {
        Some(DexEvent::MeteoraDammV2PoolAccountEvent(MeteoraDammV2PoolAccountEvent {
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