use solana_program::pubkey::Pubkey;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::{meteora_damm_v2, meteora_dlmm};
use crate::streaming::event_parser::protocols::meteora_dlmm::events::{discriminators, meteora_dlmm_swap_event_log_decode, MeteoraDlmmSwapEvent};

pub const METEORA_DLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_meteora_dlmm_instruction_data(
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

/// 根据判别器路由到具体的 inner instruction 解析函数
pub fn parse_meteora_dlmm_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP_EVENT => parse_swap_inner_instruction(data, metadata),
        _ => None,
    }
}

fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDlmmSwap;

    if data.len() < 16 || accounts.len() < 15 {
        return None;
    }

    Some(DexEvent::MeteoraDlmmSwapEvent(MeteoraDlmmSwapEvent {
        metadata,
        lb_pair: accounts[0],
        reserve_x: accounts[2],
        reserve_y: accounts[3],
        token_x_mint: accounts[6],
        token_y_mint: accounts[7],
        oracle: accounts[8],
        token_x_program: accounts[11],
        token_y_program: accounts[12],
        event_authority: accounts[13],
        ..Default::default()
    }))
}

/// 解析交易日志事件
fn parse_swap_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // Note: event_type will be set by instruction parser
    if let Some(event) = meteora_dlmm_swap_event_log_decode(data) {
        Some(DexEvent::MeteoraDlmmSwapEvent(MeteoraDlmmSwapEvent { metadata, ..event }))
    } else {
        None
    }
}

pub fn parse_meteora_dlmm_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::POOL_ACCOUNT => {
            meteora_dlmm::types::pool_parser(account, metadata)
        }
        discriminators::BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT => {
            meteora_dlmm::types::bin_array_bitmap_extension_parser(account, metadata)
        }
        _ => None,
    }
}