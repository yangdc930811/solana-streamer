use sol_common::protocols::raydium_amm_v4::{AmmInfo, MarketState};
use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::raydium_amm_v4::RaydiumAmmV4AmmInfoAccountEvent,
        DexEvent,
    },
    grpc::AccountPretty,
};



pub const AMM_INFO_SIZE: usize = 752;

pub fn amm_info_decode(data: &[u8]) -> Option<AmmInfo> {
    if data.len() < AMM_INFO_SIZE {
        return None;
    }
    borsh::from_slice::<AmmInfo>(&data[..AMM_INFO_SIZE]).ok()
}

pub fn amm_info_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumAmmV4AmmInfo;

    if account.data.len() < AMM_INFO_SIZE {
        return None;
    }
    if let Some(amm_info) = amm_info_decode(&account.data[..AMM_INFO_SIZE]) {
        Some(DexEvent::RaydiumAmmV4AmmInfoAccountEvent(RaydiumAmmV4AmmInfoAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            amm_info: amm_info,
        }))
    } else {
        None
    }
}

pub const MARKET_STATE_SIZE: usize = 388;

pub fn market_state_decode(data: &[u8]) -> Option<MarketState> {
    if data.len() < MARKET_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<MarketState>(&data[..MARKET_STATE_SIZE]).ok()
}
