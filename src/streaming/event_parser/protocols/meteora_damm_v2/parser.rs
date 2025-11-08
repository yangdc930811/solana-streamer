use solana_program::pubkey::Pubkey;
use crate::streaming::event_parser::common::{read_u64_le, EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::meteora_damm_v2;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::{discriminators, MeteoraDammV2Swap2Event, MeteoraDammV2SwapEvent, SwapParameters, SwapParameters2};

pub const METEORA_DAMM_V2_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

pub fn parse_meteora_damm_v2_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP => {
            parse_swap_instruction(data, accounts, metadata)
        }
        discriminators::SWAP2 => {
            parse_swap2_instruction(data, accounts, metadata)
        }
        _ => None,
    }
}

fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2Swap;

    if data.len() < 16 || accounts.len() < 14 {
        return None;
    }

    Some(DexEvent::MeteoraDammV2SwapEvent(MeteoraDammV2SwapEvent {
        metadata,
        pool: accounts[1],
        token_a_vault: accounts[4],
        token_b_vault: accounts[5],
        token_a_mint: accounts[6],
        token_b_mint: accounts[7],
        token_a_program: accounts[9],
        token_b_program: accounts[10],
        ..Default::default()
    }))
}

fn parse_swap2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2Swap;

    if data.len() < 17 || accounts.len() < 14 {
        return None;
    }

    Some(DexEvent::MeteoraDammV2Swap2Event(MeteoraDammV2Swap2Event {
        metadata,
        pool: accounts[1],
        token_a_vault: accounts[4],
        token_b_vault: accounts[5],
        token_a_mint: accounts[6],
        token_b_mint: accounts[7],
        token_a_program: accounts[9],
        token_b_program: accounts[10],
        ..Default::default()
    }))
}

pub fn parse_meteora_damm_v2_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::POOL_ACCOUNT => {
            meteora_damm_v2::types::pool_parser(account, metadata)
        }
        _ => None,
    }
}