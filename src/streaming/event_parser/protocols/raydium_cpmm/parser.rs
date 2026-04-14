use solana_sdk::instruction::AccountMeta;

use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType},
    protocols::raydium_cpmm::{
        discriminators, RaydiumCpmmDepositEvent, RaydiumCpmmInitializeEvent, RaydiumCpmmSwapEvent,
        RaydiumCpmmWithdrawEvent,
    },
    DexEvent,
};

/// 解析 Raydium CPMM instruction data
///
/// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_raydium_cpmm_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[AccountMeta],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP_BASE_IN => {
            parse_swap_base_input_instruction(data, accounts, metadata)
        }
        discriminators::SWAP_BASE_OUT => {
            parse_swap_base_output_instruction(data, accounts, metadata)
        }
        discriminators::DEPOSIT => parse_deposit_instruction(data, accounts, metadata),
        discriminators::INITIALIZE => parse_initialize_instruction(data, accounts, metadata),
        discriminators::WITHDRAW => parse_withdraw_instruction(data, accounts, metadata),
        _ => None,
    }
}

/// 解析 Raydium CPMM inner instruction data
///
/// Raydium CPMM 没有 inner instruction 事件
pub fn parse_raydium_cpmm_inner_instruction_data(
    _discriminator: &[u8],
    _data: &[u8],
    _metadata: EventMetadata,
) -> Option<DexEvent> {
    None
}


/// 解析 Raydium CPMM 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_raydium_cpmm_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::AMM_CONFIG => {
            crate::streaming::event_parser::protocols::raydium_cpmm::types::amm_config_parser(account, metadata)
        }
        discriminators::POOL_STATE => {
            crate::streaming::event_parser::protocols::raydium_cpmm::types::pool_state_parser(account, metadata)
        }
        _ => None,
    }
}


/// 解析提款指令事件
fn parse_withdraw_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumCpmmWithdraw;

    if data.len() < 24 || accounts.len() < 14 {
        return None;
    }
    Some(DexEvent::RaydiumCpmmWithdrawEvent(RaydiumCpmmWithdrawEvent {
        metadata,
        lp_token_amount: read_u64_le(data, 0)?,
        minimum_token0_amount: read_u64_le(data, 8)?,
        minimum_token1_amount: read_u64_le(data, 16)?,
        owner: accounts[0].pubkey,
        authority: accounts[1].pubkey,
        pool_state: accounts[2].pubkey,
        owner_lp_token: accounts[3].pubkey,
        token_0_account: accounts[4].pubkey,
        token_1_account: accounts[5].pubkey,
        token_0_vault: accounts[6].pubkey,
        token_1_vault: accounts[7].pubkey,
        token_program: accounts[8].pubkey,
        token_program2022: accounts[9].pubkey,
        vault_0_mint: accounts[10].pubkey,
        vault_1_mint: accounts[11].pubkey,
        lp_mint: accounts[12].pubkey,
        memo_program: accounts[13].pubkey,
    }))
}

/// 解析初始化指令事件
fn parse_initialize_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumCpmmInitialize;

    if data.len() < 24 || accounts.len() < 20 {
        return None;
    }
    Some(DexEvent::RaydiumCpmmInitializeEvent(RaydiumCpmmInitializeEvent {
        metadata,
        init_amount0: read_u64_le(data, 0)?,
        init_amount1: read_u64_le(data, 8)?,
        open_time: read_u64_le(data, 16)?,
        creator: accounts[0].pubkey,
        amm_config: accounts[1].pubkey,
        authority: accounts[2].pubkey,
        pool_state: accounts[3].pubkey,
        token_0_mint: accounts[4].pubkey,
        token_1_mint: accounts[5].pubkey,
        lp_mint: accounts[6].pubkey,
        creator_token_0: accounts[7].pubkey,
        creator_token_1: accounts[8].pubkey,
        creator_lp_token: accounts[9].pubkey,
        token_0_vault: accounts[10].pubkey,
        token_1_vault: accounts[11].pubkey,
        create_pool_fee: accounts[12].pubkey,
        observation_state: accounts[13].pubkey,
        token_program: accounts[14].pubkey,
        token_0_program: accounts[15].pubkey,
        token_1_program: accounts[16].pubkey,
        associated_token_program: accounts[17].pubkey,
        system_program: accounts[18].pubkey,
        rent: accounts[19].pubkey,
    }))
}

/// 解析存款指令事件
fn parse_deposit_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumCpmmDeposit;

    if data.len() < 24 || accounts.len() < 13 {
        return None;
    }
    Some(DexEvent::RaydiumCpmmDepositEvent(RaydiumCpmmDepositEvent {
        metadata,
        lp_token_amount: read_u64_le(data, 0)?,
        maximum_token0_amount: read_u64_le(data, 8)?,
        maximum_token1_amount: read_u64_le(data, 16)?,
        owner: accounts[0].pubkey,
        authority: accounts[1].pubkey,
        pool_state: accounts[2].pubkey,
        owner_lp_token: accounts[3].pubkey,
        token_0_account: accounts[4].pubkey,
        token_1_account: accounts[5].pubkey,
        token_0_vault: accounts[6].pubkey,
        token_1_vault: accounts[7].pubkey,
        token_program: accounts[8].pubkey,
        token_program2022: accounts[9].pubkey,
        vault_0_mint: accounts[10].pubkey,
        vault_1_mint: accounts[11].pubkey,
        lp_mint: accounts[12].pubkey,
    }))
}

/// 解析买入指令事件
fn parse_swap_base_input_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumCpmmSwapBaseInput;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::RaydiumCpmmSwapEvent(RaydiumCpmmSwapEvent {
        metadata,
        amount_in,
        minimum_amount_out,
        payer: accounts[0].pubkey,
        authority: accounts[1].pubkey,
        amm_config: accounts[2].pubkey,
        pool_state: accounts[3].pubkey,
        input_token_account: accounts[4].pubkey,
        output_token_account: accounts[5].pubkey,
        input_vault: accounts[6].pubkey,
        output_vault: accounts[7].pubkey,
        input_token_program: accounts[8].pubkey,
        output_token_program: accounts[9].pubkey,
        input_token_mint: accounts[10].pubkey,
        output_token_mint: accounts[11].pubkey,
        observation_state: accounts[12].pubkey,
        ..Default::default()
    }))
}

fn parse_swap_base_output_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumCpmmSwapBaseOutput;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    let max_amount_in = read_u64_le(data, 0)?;
    let amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::RaydiumCpmmSwapEvent(RaydiumCpmmSwapEvent {
        metadata,
        max_amount_in,
        amount_out,
        payer: accounts[0].pubkey,
        authority: accounts[1].pubkey,
        amm_config: accounts[2].pubkey,
        pool_state: accounts[3].pubkey,
        input_token_account: accounts[4].pubkey,
        output_token_account: accounts[5].pubkey,
        input_vault: accounts[6].pubkey,
        output_vault: accounts[7].pubkey,
        input_token_program: accounts[8].pubkey,
        output_token_program: accounts[9].pubkey,
        input_token_mint: accounts[10].pubkey,
        output_token_mint: accounts[11].pubkey,
        observation_state: accounts[12].pubkey,
        ..Default::default()
    }))
}
