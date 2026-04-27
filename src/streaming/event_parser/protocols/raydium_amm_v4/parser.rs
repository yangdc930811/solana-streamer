use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType},
    protocols::raydium_amm_v4::{
        discriminators, RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event,
        RaydiumAmmV4SwapEvent, RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
    },
    DexEvent,
};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};
use crate::streaming::event_parser::protocols::raydium_amm_v4::RaydiumAmmV4SwapV2Event;

/// 解析 Raydium AMM V4 instruction data
///
/// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_raydium_amm_v4_instruction_data(
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
        discriminators::INITIALIZE2 => parse_initialize2_instruction(data, accounts, metadata),
        discriminators::WITHDRAW => parse_withdraw_instruction(data, accounts, metadata),
        discriminators::WITHDRAW_PNL => {
            parse_withdraw_pnl_instruction(data, accounts, metadata)
        }
        discriminators::SWAP_BASE_IN_V2 => {
            parse_swap_base_input_v2_instruction(data, accounts, metadata)
        }
        discriminators::SWAP_BASE_OUT_V2 => {
            parse_swap_base_output_v2_instruction(data, accounts, metadata)
        }
        _ => None,
    }
}

/// 解析 Raydium AMM V4 inner instruction data
///
/// Raydium AMM V4 没有 inner instruction 事件
pub fn parse_raydium_amm_v4_inner_instruction_data(
    _discriminator: &[u8],
    _data: &[u8],
    _metadata: EventMetadata,
) -> Option<DexEvent> {
    None
}


/// 解析 Raydium AMM V4 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_raydium_amm_v4_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::AMM_INFO => {
            crate::streaming::event_parser::protocols::raydium_amm_v4::types::amm_info_parser(account, metadata)
        }
        _ => None,
    }
}


/// 解析提现指令事件
fn parse_withdraw_pnl_instruction(
    _data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4WithdrawPnl;

    if accounts.len() < 17 {
        return None;
    }

    Some(DexEvent::RaydiumAmmV4WithdrawPnlEvent(RaydiumAmmV4WithdrawPnlEvent {
        metadata,
        token_program: accounts[0].pubkey,
        amm: accounts[1].pubkey,
        amm_config: accounts[2].pubkey,
        amm_authority: accounts[3].pubkey,
        amm_open_orders: accounts[4].pubkey,
        pool_coin_token_account: accounts[5].pubkey,
        pool_pc_token_account: accounts[6].pubkey,
        coin_pnl_token_account: accounts[7].pubkey,
        pc_pnl_token_account: accounts[8].pubkey,
        pnl_owner_account: accounts[9].pubkey,
        amm_target_orders: accounts[10].pubkey,
        serum_program: accounts[11].pubkey,
        serum_market: accounts[12].pubkey,
        serum_event_queue: accounts[13].pubkey,
        serum_coin_vault_account: accounts[14].pubkey,
        serum_pc_vault_account: accounts[15].pubkey,
        serum_vault_signer: accounts[16].pubkey,
    }))
}

/// 解析移除流动性指令事件
fn parse_withdraw_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4Withdraw;

    if data.len() < 8 || accounts.len() < 22 {
        return None;
    }
    let amount = read_u64_le(data, 0)?;

    Some(DexEvent::RaydiumAmmV4WithdrawEvent(RaydiumAmmV4WithdrawEvent {
        metadata,
        amount,

        token_program: accounts[0].pubkey,
        amm: accounts[1].pubkey,
        amm_authority: accounts[2].pubkey,
        amm_open_orders: accounts[3].pubkey,
        amm_target_orders: accounts[4].pubkey,
        lp_mint_address: accounts[5].pubkey,
        pool_coin_token_account: accounts[6].pubkey,
        pool_pc_token_account: accounts[7].pubkey,
        pool_withdraw_queue: accounts[8].pubkey,
        pool_temp_lp_token_account: accounts[9].pubkey,
        serum_program: accounts[10].pubkey,
        serum_market: accounts[11].pubkey,
        serum_coin_vault_account: accounts[12].pubkey,
        serum_pc_vault_account: accounts[13].pubkey,
        serum_vault_signer: accounts[14].pubkey,
        user_lp_token_account: accounts[15].pubkey,
        user_coin_token_account: accounts[16].pubkey,
        user_pc_token_account: accounts[17].pubkey,
        user_owner: accounts[18].pubkey,
        serum_event_queue: accounts[19].pubkey,
        serum_bids: accounts[20].pubkey,
        serum_asks: accounts[21].pubkey,
    }))
}

/// 解析初始化指令事件
fn parse_initialize2_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4Initialize2;

    if data.len() < 25 || accounts.len() < 21 {
        return None;
    }
    let nonce = data[0];
    let open_time = read_u64_le(data, 1)?;
    let init_pc_amount = read_u64_le(data, 9)?;
    let init_coin_amount = read_u64_le(data, 17)?;

    Some(DexEvent::RaydiumAmmV4Initialize2Event(RaydiumAmmV4Initialize2Event {
        metadata,
        nonce,
        open_time,
        init_pc_amount,
        init_coin_amount,

        token_program: accounts[0].pubkey,
        spl_associated_token_account: accounts[1].pubkey,
        system_program: accounts[2].pubkey,
        rent: accounts[3].pubkey,
        amm: accounts[4].pubkey,
        amm_authority: accounts[5].pubkey,
        amm_open_orders: accounts[6].pubkey,
        lp_mint: accounts[7].pubkey,
        coin_mint: accounts[8].pubkey,
        pc_mint: accounts[9].pubkey,
        pool_coin_token_account: accounts[10].pubkey,
        pool_pc_token_account: accounts[11].pubkey,
        pool_withdraw_queue: accounts[12].pubkey,
        amm_target_orders: accounts[13].pubkey,
        pool_temp_lp: accounts[14].pubkey,
        serum_program: accounts[15].pubkey,
        serum_market: accounts[16].pubkey,
        user_wallet: accounts[17].pubkey,
        user_token_coin: accounts[18].pubkey,
        user_token_pc: accounts[19].pubkey,
        user_lp_token_account: accounts[20].pubkey,
    }))
}

/// 解析添加流动性指令事件
fn parse_deposit_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4Deposit;

    if data.len() < 24 || accounts.len() < 14 {
        return None;
    }
    let max_coin_amount = read_u64_le(data, 0)?;
    let max_pc_amount = read_u64_le(data, 8)?;
    let base_side = read_u64_le(data, 16)?;

    Some(DexEvent::RaydiumAmmV4DepositEvent(RaydiumAmmV4DepositEvent {
        metadata,
        max_coin_amount,
        max_pc_amount,
        base_side,

        token_program: accounts[0].pubkey,
        amm: accounts[1].pubkey,
        amm_authority: accounts[2].pubkey,
        amm_open_orders: accounts[3].pubkey,
        amm_target_orders: accounts[4].pubkey,
        lp_mint_address: accounts[5].pubkey,
        pool_coin_token_account: accounts[6].pubkey,
        pool_pc_token_account: accounts[7].pubkey,
        serum_market: accounts[8].pubkey,
        user_coin_token_account: accounts[9].pubkey,
        user_pc_token_account: accounts[10].pubkey,
        user_lp_token_account: accounts[11].pubkey,
        user_owner: accounts[12].pubkey,
        serum_event_queue: accounts[13].pubkey,
    }))
}

/// 解析买入指令事件
fn parse_swap_base_output_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4SwapBaseOut;

    if data.len() < 16 || accounts.len() < 17 {
        return None;
    }
    let max_amount_in = read_u64_le(data, 0)?;
    let amount_out = read_u64_le(data, 8)?;

    let mut accounts = accounts.to_vec();
    if accounts.len() == 17 {
        // 添加一个默认的 Pubkey 作为 amm_target_orders 的占位符
        // 因为在某些情况下，amm_target_orders 可能是可选的
        accounts.insert(4, AccountMeta::new_readonly(Pubkey::default(), false));
    }

    Some(DexEvent::RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent {
        metadata,
        max_amount_in,
        amount_out,

        token_program: accounts[0].pubkey,
        amm: accounts[1].pubkey,
        amm_authority: accounts[2].pubkey,
        amm_open_orders: accounts[3].pubkey,
        amm_target_orders: Some(accounts[4].pubkey),
        pool_coin_token_account: accounts[5].pubkey,
        pool_pc_token_account: accounts[6].pubkey,
        serum_program: accounts[7].pubkey,
        serum_market: accounts[8].pubkey,
        serum_bids: accounts[9].pubkey,
        serum_asks: accounts[10].pubkey,
        serum_event_queue: accounts[11].pubkey,
        serum_coin_vault_account: accounts[12].pubkey,
        serum_pc_vault_account: accounts[13].pubkey,
        serum_vault_signer: accounts[14].pubkey,
        user_source_token_account: accounts[15].pubkey,
        user_destination_token_account: accounts[16].pubkey,
        user_source_owner: accounts[17].pubkey,

        ..Default::default()
    }))
}

/// 解析买入指令事件
fn parse_swap_base_input_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4SwapBaseIn;

    if data.len() < 16 || accounts.len() < 17 {
        return None;
    }
    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    let mut accounts = accounts.to_vec();
    if accounts.len() == 17 {
        // 添加一个默认的 Pubkey 作为 amm_target_orders 的占位符
        // 因为在某些情况下，amm_target_orders 可能是可选的
        accounts.insert(4, AccountMeta::new_readonly(Pubkey::default(), false));
    }

    Some(DexEvent::RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent {
        metadata,
        amount_in,
        minimum_amount_out,

        token_program: accounts[0].pubkey,
        amm: accounts[1].pubkey,
        amm_authority: accounts[2].pubkey,
        amm_open_orders: accounts[3].pubkey,
        amm_target_orders: Some(accounts[4].pubkey),
        pool_coin_token_account: accounts[5].pubkey,
        pool_pc_token_account: accounts[6].pubkey,
        serum_program: accounts[7].pubkey,
        serum_market: accounts[8].pubkey,
        serum_bids: accounts[9].pubkey,
        serum_asks: accounts[10].pubkey,
        serum_event_queue: accounts[11].pubkey,
        serum_coin_vault_account: accounts[12].pubkey,
        serum_pc_vault_account: accounts[13].pubkey,
        serum_vault_signer: accounts[14].pubkey,
        user_source_token_account: accounts[15].pubkey,
        user_destination_token_account: accounts[16].pubkey,
        user_source_owner: accounts[17].pubkey,

        ..Default::default()
    }))
}

fn parse_swap_base_input_v2_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4SwapBaseInV2;

    if data.len() < 16 || accounts.len() < 8 {
        return None;
    }
    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::RaydiumAmmV4SwapV2Event(RaydiumAmmV4SwapV2Event {
        metadata,
        amount_in,
        minimum_amount_out,
        ..Default::default()
    }))
}

fn parse_swap_base_output_v2_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::RaydiumAmmV4SwapBaseOutV2;

    if data.len() < 16 || accounts.len() < 8 {
        return None;
    }
    let max_amount_in = read_u64_le(data, 0)?;
    let amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::RaydiumAmmV4SwapV2Event(RaydiumAmmV4SwapV2Event {
        metadata,
        max_amount_in,
        amount_out,
        ..Default::default()
    }))
}