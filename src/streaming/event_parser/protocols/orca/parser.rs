use solana_program::pubkey::Pubkey;
use crate::streaming::event_parser::common::{read_u128_le, read_u64_le, read_u8_le, EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::orca;
use crate::streaming::event_parser::protocols::orca::events::{discriminators, OrcaSwapEvent};

// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_orca_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP => parse_swap_instruction(data, accounts, metadata),
        _ => None,
    }
}

fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::OrcaSwap;

    if data.len() < 34 || accounts.len() < 11 {
        return None;
    }

    let amount = read_u64_le(data, 0)?;
    let other_amount_threshold = read_u64_le(data, 8)?;
    let sqrt_price_limit = read_u128_le(data, 16)?;
    let amount_specified_is_input = read_u8_le(data, 32)? != 0;
    let a_to_b = read_u8_le(data, 33)? != 0;

    Some(DexEvent::OrcaSwapEvent(OrcaSwapEvent {
        metadata,
        amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input,
        a_to_b,
        token_program: accounts[0],
        whirlpool: accounts[2],
        token_owner_account_a: accounts[3],
        token_vault_a: accounts[4],
        token_owner_account_b: accounts[5],
        token_vault_b: accounts[6],
        oracle: accounts[10],
    }))
}

pub fn parse_orca_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::POOL_ACCOUNT => {
            orca::types::pool_parser(account, metadata)
        }
        _ => None,
    }
}