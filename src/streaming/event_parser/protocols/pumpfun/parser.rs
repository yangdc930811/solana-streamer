use crate::streaming::event_parser::{
    common::{EventMetadata, EventType},
    protocols::pumpfun::{
        discriminators, pumpfun_create_v2_token_event_log_decode, pumpfun_migrate_event_log_decode,
        pumpfun_trade_event_log_decode, PumpFunCreateTokenEvent, PumpFunCreateV2TokenEvent,
        PumpFunMigrateEvent, PumpFunTradeEvent,
    },
    DexEvent,
};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

/// 解析 PumpFun instruction data
///
/// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_pumpfun_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[AccountMeta],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::CREATE_TOKEN_IX => parse_create_token_instruction(data, accounts, metadata),
        discriminators::CREATE_V2_TOKEN_IX => {
            parse_create_v2_token_instruction(data, accounts, metadata)
        }
        discriminators::BUY_IX => parse_buy_instruction(data, accounts, metadata),
        discriminators::BUY_EXACT_SOL_IN_IX => parse_buy_exact_sol_in_instruction(data, accounts, metadata),
        discriminators::SELL_IX => parse_sell_instruction(data, accounts, metadata),
        discriminators::MIGRATE_IX => parse_migrate_instruction(data, accounts, metadata),
        _ => None,
    }
}

/// 解析 PumpFun inner instruction data
///
/// 根据判别器路由到具体的 inner instruction 解析函数
pub fn parse_pumpfun_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::CREATE_TOKEN_EVENT => parse_create_token_inner_instruction(data, metadata),
        discriminators::TRADE_EVENT => parse_trade_inner_instruction(data, metadata),
        discriminators::COMPLETE_PUMP_AMM_MIGRATION_EVENT => {
            parse_migrate_inner_instruction(data, metadata)
        }
        _ => None,
    }
}

/// 解析 PumpFun 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_pumpfun_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::BONDING_CURVE_ACCOUNT => {
            crate::streaming::event_parser::protocols::pumpfun::types::bonding_curve_parser(
                account, metadata,
            )
        }
        discriminators::GLOBAL_ACCOUNT => {
            crate::streaming::event_parser::protocols::pumpfun::types::global_parser(
                account, metadata,
            )
        }
        _ => None,
    }
}

/// 解析迁移事件
fn parse_migrate_inner_instruction(data: &[u8], mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunMigrate;
    if let Some(event) = pumpfun_migrate_event_log_decode(data) {
        Some(DexEvent::PumpFunMigrateEvent(PumpFunMigrateEvent { metadata, ..event }))
    } else {
        None
    }
}

/// 解析创建代币日志事件
fn parse_create_token_inner_instruction(
    data: &[u8],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunCreateToken;
    if let Some(event) = pumpfun_create_v2_token_event_log_decode(data) {
        Some(DexEvent::PumpFunCreateV2TokenEvent(PumpFunCreateV2TokenEvent { metadata, ..event }))
    } else {
        None
    }
}

/// 解析交易事件 (inner instruction 不设置 event_type，因为不知道是 Buy 还是 Sell)
fn parse_trade_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 注意：inner instruction 的 trade event 不设置 event_type
    // 因为它会被合并到 instruction event 中，而 instruction event 已经设置了正确的 event_type
    if let Some(event) = pumpfun_trade_event_log_decode(data) {
        Some(DexEvent::PumpFunTradeEvent(PumpFunTradeEvent { metadata, ..event }))
    } else {
        None
    }
}

/// 解析创建代币指令事件
fn parse_create_token_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunCreateToken;

    if data.len() < 16 || accounts.len() < 11 {
        return None;
    }
    let mut offset = 0;
    if offset + 4 > data.len() {
        return None;
    }
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + name_len > data.len() {
        return None;
    }
    let name = String::from_utf8_lossy(&data[offset..offset + name_len]);
    offset += name_len;
    if offset + 4 > data.len() {
        return None;
    }
    let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + symbol_len > data.len() {
        return None;
    }
    let symbol = String::from_utf8_lossy(&data[offset..offset + symbol_len]);
    offset += symbol_len;
    if offset + 4 > data.len() {
        return None;
    }
    let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + uri_len > data.len() {
        return None;
    }
    let uri = String::from_utf8_lossy(&data[offset..offset + uri_len]);
    offset += uri_len;
    let creator = if offset + 32 <= data.len() {
        Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?)
    } else {
        Pubkey::default()
    };

    Some(DexEvent::PumpFunCreateTokenEvent(PumpFunCreateTokenEvent {
        metadata,
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        creator,
        mint: accounts[0].pubkey,
        mint_authority: accounts[1].pubkey,
        bonding_curve: accounts[2].pubkey,
        associated_bonding_curve: accounts[3].pubkey,
        global: accounts[4].pubkey,
        mpl_token_metadata: accounts[5].pubkey,
        metadata_account: accounts[6].pubkey,
        user: accounts[7].pubkey,
        system_program: accounts[8].pubkey,
        token_program: accounts[9].pubkey,
        associated_token_program: accounts[10].pubkey,
        rent: accounts[11].pubkey,
        event_authority: accounts[12].pubkey,
        program: accounts[13].pubkey,
        ..Default::default()
    }))
}

/// 解析创建 V2 代币指令事件 (SPL-22 Token, Mayhem Mode)
fn parse_create_v2_token_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunCreateV2Token;

    if data.len() < 16 || accounts.len() < 11 {
        return None;
    }
    let mut offset = 0;
    if offset + 4 > data.len() {
        return None;
    }
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + name_len > data.len() {
        return None;
    }
    let name = String::from_utf8_lossy(&data[offset..offset + name_len]);
    offset += name_len;
    if offset + 4 > data.len() {
        return None;
    }
    let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + symbol_len > data.len() {
        return None;
    }
    let symbol = String::from_utf8_lossy(&data[offset..offset + symbol_len]);
    offset += symbol_len;
    if offset + 4 > data.len() {
        return None;
    }
    let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + uri_len > data.len() {
        return None;
    }
    let uri = String::from_utf8_lossy(&data[offset..offset + uri_len]);
    offset += uri_len;
    let creator = if offset + 32 <= data.len() {
        Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?)
    } else {
        Pubkey::default()
    };

    Some(DexEvent::PumpFunCreateV2TokenEvent(PumpFunCreateV2TokenEvent {
        metadata,
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        creator,
        mint: accounts[0].pubkey,
        mint_authority: accounts[1].pubkey,
        bonding_curve: accounts[2].pubkey,
        associated_bonding_curve: accounts[3].pubkey,
        global: accounts[4].pubkey,
        user: accounts[5].pubkey,
        system_program: accounts[6].pubkey,
        token_program: accounts[7].pubkey,
        associated_token_program: accounts[8].pubkey,
        mayhem_program_id: accounts[9].pubkey,
        global_params: accounts[10].pubkey,
        sol_vault: accounts[11].pubkey,
        mayhem_state: accounts[12].pubkey,
        mayhem_token_vault: accounts[13].pubkey,
        event_authority: accounts[14].pubkey,
        program: accounts[15].pubkey,
        ..Default::default()
    }))
}

// 解析买入指令事件
fn parse_buy_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunBuy;

    if data.len() < 16 || accounts.len() < 16 {
        return None;
    }
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let max_sol_cost = u64::from_le_bytes(data[8..16].try_into().unwrap());
    Some(DexEvent::PumpFunTradeEvent(PumpFunTradeEvent {
        metadata,
        global: accounts[0].pubkey,
        fee_recipient: accounts[1].pubkey,
        mint: accounts[2].pubkey,
        bonding_curve: accounts[3].pubkey,
        associated_bonding_curve: accounts[4].pubkey,
        associated_user: accounts[5].pubkey,
        user: accounts[6].pubkey,
        system_program: accounts[7].pubkey,
        token_program: accounts[8].pubkey,
        creator_vault: accounts[9].pubkey,
        event_authority: accounts[10].pubkey,
        program: accounts[11].pubkey,
        global_volume_accumulator: accounts[12].pubkey,
        user_volume_accumulator: accounts[13].pubkey,
        fee_config: accounts[14].pubkey,
        fee_program: accounts[15].pubkey,
        max_sol_cost,
        amount,
        is_buy: true,
        ..Default::default()
    }))
}

/// 解析 buy_exact_sol_in 指令事件
/// 注意：参数顺序与 buy 指令不同
/// buy_exact_sol_in: spendable_sol_in (SOL), min_tokens_out (token)
/// buy: amount (token), max_sol_cost (SOL)
fn parse_buy_exact_sol_in_instruction(
    data: &[u8], accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunBuy;

    if data.len() < 16 || accounts.len() < 16 {
        return None;
    }

    // 注意：buy_exact_sol_in 的参数顺序是先 SOL 再 token
    let spendable_sol_in = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let min_tokens_out = u64::from_le_bytes(data[8..16].try_into().unwrap());

    Some(DexEvent::PumpFunTradeEvent(PumpFunTradeEvent {
        metadata,
        global: accounts[0].pubkey,
        fee_recipient: accounts[1].pubkey,
        mint: accounts[2].pubkey,
        bonding_curve: accounts[3].pubkey,
        associated_bonding_curve: accounts[4].pubkey,
        associated_user: accounts[5].pubkey,
        user: accounts[6].pubkey,
        system_program: accounts[7].pubkey,
        token_program: accounts[8].pubkey,
        creator_vault: accounts[9].pubkey,
        event_authority: accounts[10].pubkey,
        program: accounts[11].pubkey,
        global_volume_accumulator: accounts[12].pubkey,
        user_volume_accumulator: accounts[13].pubkey,
        fee_config: accounts[14].pubkey,
        fee_program: accounts[15].pubkey,
        max_sol_cost: spendable_sol_in,  // Map spendable_sol_in to max_sol_cost
        amount: min_tokens_out,           // Map min_tokens_out to amount
        is_buy: true,
        ..Default::default()
    }))
}

// 解析卖出指令事件
fn parse_sell_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunSell;

    if data.len() < 16 || accounts.len() < 14 {
        return None;
    }
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let min_sol_output = u64::from_le_bytes(data[8..16].try_into().unwrap());
    Some(DexEvent::PumpFunTradeEvent(PumpFunTradeEvent {
        metadata,
        global: accounts[0].pubkey,
        fee_recipient: accounts[1].pubkey,
        mint: accounts[2].pubkey,
        bonding_curve: accounts[3].pubkey,
        associated_bonding_curve: accounts[4].pubkey,
        associated_user: accounts[5].pubkey,
        user: accounts[6].pubkey,
        system_program: accounts[7].pubkey,
        creator_vault: accounts[8].pubkey,
        token_program: accounts[9].pubkey,
        event_authority: accounts[10].pubkey,
        program: accounts[11].pubkey,
        global_volume_accumulator: Pubkey::default(),
        user_volume_accumulator: Pubkey::default(),
        fee_config: accounts[12].pubkey,
        fee_program: accounts[13].pubkey,
        min_sol_output,
        amount,
        is_buy: false,
        ..Default::default()
    }))
}

/// 解析迁移指令事件
fn parse_migrate_instruction(
    _data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpFunMigrate;

    if accounts.len() < 24 {
        return None;
    }
    Some(DexEvent::PumpFunMigrateEvent(PumpFunMigrateEvent {
        metadata,
        global: accounts[0].pubkey,
        withdraw_authority: accounts[1].pubkey,
        mint: accounts[2].pubkey,
        bonding_curve: accounts[3].pubkey,
        associated_bonding_curve: accounts[4].pubkey,
        user: accounts[5].pubkey,
        system_program: accounts[6].pubkey,
        token_program: accounts[7].pubkey,
        pump_amm: accounts[8].pubkey,
        pool: accounts[9].pubkey,
        pool_authority: accounts[10].pubkey,
        pool_authority_mint_account: accounts[11].pubkey,
        pool_authority_wsol_account: accounts[12].pubkey,
        amm_global_config: accounts[13].pubkey,
        wsol_mint: accounts[14].pubkey,
        lp_mint: accounts[15].pubkey,
        user_pool_token_account: accounts[16].pubkey,
        pool_base_token_account: accounts[17].pubkey,
        pool_quote_token_account: accounts[18].pubkey,
        token_2022_program: accounts[19].pubkey,
        associated_token_program: accounts[20].pubkey,
        pump_amm_event_authority: accounts[21].pubkey,
        event_authority: accounts[22].pubkey,
        program: accounts[23].pubkey,
        ..Default::default()
    }))
}
