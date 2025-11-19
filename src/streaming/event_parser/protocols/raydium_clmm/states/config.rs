use borsh_derive::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::raydium_clmm::RaydiumClmmAmmConfigAccountEvent;
use crate::streaming::grpc::AccountPretty;

pub const AMM_CONFIG_SEED: &str = "amm_config";

pub const FEE_RATE_DENOMINATOR_VALUE: u32 = 1_000_000;

/// Holds the current owner of the factory
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct AmmConfig {
    /// Bump to identify PDA
    pub bump: u8,
    pub index: u16,
    /// Address of the protocol owner
    pub owner: Pubkey,
    /// The protocol fee
    pub protocol_fee_rate: u32,
    /// The trade fee, denominated in hundredths of a bip (10^-6)
    pub trade_fee_rate: u32,
    /// The tick spacing
    pub tick_spacing: u16,
    /// The fund fee, denominated in hundredths of a bip (10^-6)
    pub fund_fee_rate: u32,
    // padding space for upgrade
    pub padding_u32: u32,
    pub fund_owner: Pubkey,
    pub padding: [u64; 3],
}

impl AmmConfig {
    pub const LEN: usize = 8 + 1 + 2 + 32 + 4 + 4 + 2 + 64;
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