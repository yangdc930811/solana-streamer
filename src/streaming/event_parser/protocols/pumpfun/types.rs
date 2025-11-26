use sol_common::protocols::pumpfun::{BondingCurve, Global};
use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::pumpfun::{PumpFunBondingCurveAccountEvent, PumpFunGlobalAccountEvent},
        DexEvent,
    },
    grpc::AccountPretty,
};

pub const BONDING_CURVE_SIZE: usize = 8 * 5 + 1 + 32 + 1;

pub fn bonding_curve_decode(data: &[u8]) -> Option<BondingCurve> {
    if data.len() < BONDING_CURVE_SIZE {
        return None;
    }
    borsh::from_slice::<BondingCurve>(&data[..BONDING_CURVE_SIZE]).ok()
}

pub fn bonding_curve_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountPumpFunBondingCurve;

    if account.data.len() < BONDING_CURVE_SIZE + 8 {
        return None;
    }
    if let Some(bonding_curve) = bonding_curve_decode(&account.data[8..BONDING_CURVE_SIZE + 8]) {
        Some(DexEvent::PumpFunBondingCurveAccountEvent(PumpFunBondingCurveAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            bonding_curve,
        }))
    } else {
        None
    }
}

pub const GLOBAL_SIZE: usize = 1 + 32 * 2 + 8 * 5 + 32 + 1 + 8 * 2 + 32 * 7 + 32 * 2 + 1 + 32 * 2 + 1;

pub fn global_decode(data: &[u8]) -> Option<Global> {
    if data.len() < GLOBAL_SIZE {
        return None;
    }
    borsh::from_slice::<Global>(&data[..GLOBAL_SIZE]).ok()
}

pub fn global_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountPumpFunGlobal;

    if account.data.len() < GLOBAL_SIZE + 8 {
        return None;
    }
    if let Some(global) = global_decode(&account.data[8..GLOBAL_SIZE + 8]) {
        Some(DexEvent::PumpFunGlobalAccountEvent(PumpFunGlobalAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            global,
        }))
    } else {
        None
    }
}
