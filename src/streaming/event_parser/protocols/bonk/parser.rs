use solana_sdk::instruction::AccountMeta;

use crate::streaming::event_parser::{
    common::{utils::*, EventMetadata, EventType},
    protocols::bonk::{
        bonk_pool_create_event_log_decode, bonk_trade_event_log_decode, discriminators, AmmFeeOn,
        BonkMigrateToAmmEvent, BonkMigrateToCpswapEvent, BonkPoolCreateEvent, BonkTradeEvent,
        ConstantCurve, CurveParams, FixedCurve, LinearCurve, MintParams, TradeDirection,
        VestingParams,
    },
    DexEvent,
};

/// 解析 Bonk instruction data
///
/// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_bonk_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[AccountMeta],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::BUY_EXACT_IN => {
            parse_buy_exact_in_instruction(data, accounts, metadata)
        }
        discriminators::BUY_EXACT_OUT => {
            parse_buy_exact_out_instruction(data, accounts, metadata)
        }
        discriminators::SELL_EXACT_IN => {
            parse_sell_exact_in_instruction(data, accounts, metadata)
        }
        discriminators::SELL_EXACT_OUT => {
            parse_sell_exact_out_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE => {
            parse_initialize_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_V2 => {
            parse_initialize_v2_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_WITH_TOKEN_2022 => {
            parse_initialize_with_token_2022_instruction(data, accounts, metadata)
        }
        discriminators::MIGRATE_TO_AMM => {
            parse_migrate_to_amm_instruction(data, accounts, metadata)
        }
        discriminators::MIGRATE_TO_CP_SWAP => {
            parse_migrate_to_cpswap_instruction(data, accounts, metadata)
        }
        _ => None,
    }
}

/// 解析 Bonk inner instruction data
///
/// 根据判别器路由到具体的 inner instruction 解析函数
pub fn parse_bonk_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::TRADE_EVENT => {
            parse_trade_inner_instruction(data, metadata)
        }
        discriminators::POOL_CREATE_EVENT => {
            parse_pool_create_inner_instruction(data, metadata)
        }
        _ => None,
    }
}

/// 解析 Bonk 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_bonk_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::POOL_STATE_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::pool_state_parser(account, metadata)
        }
        discriminators::GLOBAL_CONFIG_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::global_config_parser(account, metadata)
        }
        discriminators::PLATFORM_CONFIG_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::platform_config_parser(account, metadata)
        }
        _ => None,
    }
}

/// Parse pool creation event
fn parse_pool_create_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    // Note: event_type will be set by the instruction parser, not here
    // Because different initialize instructions have different event types
    if let Some(event) = bonk_pool_create_event_log_decode(data) {
        Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse trade event
fn parse_trade_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    if let Some(event) = bonk_trade_event_log_decode(data) {
        if metadata.event_type == EventType::BonkBuyExactIn
            || metadata.event_type == EventType::BonkBuyExactOut
        {
            if event.trade_direction != TradeDirection::Buy {
                return None;
            }
        } else if (metadata.event_type == EventType::BonkSellExactIn
            || metadata.event_type == EventType::BonkSellExactOut)
            && event.trade_direction != TradeDirection::Sell
        {
            return None;
        }
        Some(DexEvent::BonkTradeEvent(BonkTradeEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse buy instruction event
fn parse_buy_exact_in_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkBuyExactIn;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_in,
        minimum_amount_out,
        share_fee_rate,
        payer: accounts[0].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[4].pubkey,
        user_base_token: accounts[5].pubkey,
        user_quote_token: accounts[6].pubkey,
        base_vault: accounts[7].pubkey,
        quote_vault: accounts[8].pubkey,
        base_token_mint: accounts[9].pubkey,
        quote_token_mint: accounts[10].pubkey,
        base_token_program: accounts[11].pubkey,
        quote_token_program: accounts[12].pubkey,
        system_program: accounts[15].pubkey,
        platform_associated_account: accounts[16].pubkey,
        creator_associated_account: accounts[17].pubkey,
        trade_direction: TradeDirection::Buy,
        ..Default::default()
    }))
}

fn parse_buy_exact_out_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkBuyExactOut;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_out = read_u64_le(data, 0)?;
    let maximum_amount_in = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_out,
        maximum_amount_in,
        share_fee_rate,
        payer: accounts[0].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[4].pubkey,
        user_base_token: accounts[5].pubkey,
        user_quote_token: accounts[6].pubkey,
        base_vault: accounts[7].pubkey,
        quote_vault: accounts[8].pubkey,
        base_token_mint: accounts[9].pubkey,
        quote_token_mint: accounts[10].pubkey,
        base_token_program: accounts[11].pubkey,
        quote_token_program: accounts[12].pubkey,
        system_program: accounts[15].pubkey,
        platform_associated_account: accounts[16].pubkey,
        creator_associated_account: accounts[17].pubkey,
        trade_direction: TradeDirection::Buy,
        ..Default::default()
    }))
}

fn parse_sell_exact_in_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkSellExactIn;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_in,
        minimum_amount_out,
        share_fee_rate,
        payer: accounts[0].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[4].pubkey,
        user_base_token: accounts[5].pubkey,
        user_quote_token: accounts[6].pubkey,
        base_vault: accounts[7].pubkey,
        quote_vault: accounts[8].pubkey,
        base_token_mint: accounts[9].pubkey,
        quote_token_mint: accounts[10].pubkey,
        base_token_program: accounts[11].pubkey,
        quote_token_program: accounts[12].pubkey,
        system_program: accounts[15].pubkey,
        platform_associated_account: accounts[16].pubkey,
        creator_associated_account: accounts[17].pubkey,
        trade_direction: TradeDirection::Sell,
        ..Default::default()
    }))
}

fn parse_sell_exact_out_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkSellExactOut;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_out = read_u64_le(data, 0)?;
    let maximum_amount_in = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_out,
        maximum_amount_in,
        share_fee_rate,
        payer: accounts[0].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[4].pubkey,
        user_base_token: accounts[5].pubkey,
        user_quote_token: accounts[6].pubkey,
        base_vault: accounts[7].pubkey,
        quote_vault: accounts[8].pubkey,
        base_token_mint: accounts[9].pubkey,
        quote_token_mint: accounts[10].pubkey,
        base_token_program: accounts[11].pubkey,
        quote_token_program: accounts[12].pubkey,
        system_program: accounts[15].pubkey,
        platform_associated_account: accounts[16].pubkey,
        creator_associated_account: accounts[17].pubkey,
        trade_direction: TradeDirection::Sell,
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitialize;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0].pubkey,
        creator: accounts[1].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[5].pubkey,
        base_mint: accounts[6].pubkey,
        quote_mint: accounts[7].pubkey,
        base_vault: accounts[8].pubkey,
        quote_vault: accounts[9].pubkey,
        base_mint_param,
        curve_param,
        vesting_param,
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_v2_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitializeV2;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;
    let amm_fee_on = data[offset];

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0].pubkey,
        creator: accounts[1].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[5].pubkey,
        base_mint: accounts[6].pubkey,
        quote_mint: accounts[7].pubkey,
        base_vault: accounts[8].pubkey,
        quote_vault: accounts[9].pubkey,
        base_mint_param,
        curve_param,
        vesting_param,
        amm_fee_on: if amm_fee_on == 0 {
            Some(AmmFeeOn::QuoteToken)
        } else {
            Some(AmmFeeOn::BothToken)
        },
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_with_token_2022_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitializeWithToken2022;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;
    let amm_fee_on = data[offset];

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0].pubkey,
        creator: accounts[1].pubkey,
        global_config: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        pool_state: accounts[5].pubkey,
        base_mint: accounts[6].pubkey,
        quote_mint: accounts[7].pubkey,
        base_vault: accounts[8].pubkey,
        quote_vault: accounts[9].pubkey,
        base_mint_param,
        curve_param,
        vesting_param,
        amm_fee_on: if amm_fee_on == 0 {
            Some(AmmFeeOn::QuoteToken)
        } else {
            Some(AmmFeeOn::BothToken)
        },
        ..Default::default()
    }))
}

/// Parse MintParams structure
fn parse_mint_params(data: &[u8], offset: &mut usize) -> Option<MintParams> {
    // Read decimals (1 byte)
    let decimals = read_u8(data, *offset)?;
    *offset += 1;

    // Read name string length and content
    let name_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + name_len {
        return None;
    }
    let name = String::from_utf8(data[*offset..*offset + name_len].to_vec()).ok()?;
    *offset += name_len;

    // Read symbol string length and content
    let symbol_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + symbol_len {
        return None;
    }
    let symbol = String::from_utf8(data[*offset..*offset + symbol_len].to_vec()).ok()?;
    *offset += symbol_len;

    // Read uri string length and content
    let uri_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + uri_len {
        return None;
    }
    let uri = String::from_utf8(data[*offset..*offset + uri_len].to_vec()).ok()?;
    *offset += uri_len;

    Some(MintParams { decimals, name, symbol, uri })
}

/// Parse CurveParams structure
fn parse_curve_params(data: &[u8], offset: &mut usize) -> Option<CurveParams> {
    // Read curve type identifier (1 byte)
    let curve_type = read_u8(data, *offset)?;
    *offset += 1;

    match curve_type {
        0 => {
            // Constant curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_base_sell = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Constant {
                data: ConstantCurve {
                    supply,
                    total_base_sell,
                    total_quote_fund_raising,
                    migrate_type,
                },
            })
        }
        1 => {
            // Fixed curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Fixed {
                data: FixedCurve { supply, total_quote_fund_raising, migrate_type },
            })
        }
        2 => {
            // Linear curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Linear {
                data: LinearCurve { supply, total_quote_fund_raising, migrate_type },
            })
        }
        _ => None,
    }
}

/// Parse VestingParams structure
fn parse_vesting_params(data: &[u8], offset: &mut usize) -> Option<VestingParams> {
    let total_locked_amount = read_u64_le(data, *offset)?;
    *offset += 8;
    let cliff_period = read_u64_le(data, *offset)?;
    *offset += 8;
    let unlock_period = read_u64_le(data, *offset)?;
    *offset += 8;

    Some(VestingParams { total_locked_amount, cliff_period, unlock_period })
}

/// Parse migrate to AMM event
fn parse_migrate_to_amm_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkMigrateToAmm;

    if data.len() < 16 {
        return None;
    }

    let base_lot_size = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let quote_lot_size = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let market_vault_signer_nonce = data[16];

    Some(DexEvent::BonkMigrateToAmmEvent(BonkMigrateToAmmEvent {
        metadata,
        base_lot_size,
        quote_lot_size,
        market_vault_signer_nonce,
        payer: accounts[0].pubkey,
        base_mint: accounts[1].pubkey,
        quote_mint: accounts[2].pubkey,
        openbook_program: accounts[3].pubkey,
        market: accounts[4].pubkey,
        request_queue: accounts[5].pubkey,
        event_queue: accounts[6].pubkey,
        bids: accounts[7].pubkey,
        asks: accounts[8].pubkey,
        market_vault_signer: accounts[9].pubkey,
        market_base_vault: accounts[10].pubkey,
        market_quote_vault: accounts[11].pubkey,
        amm_program: accounts[12].pubkey,
        amm_pool: accounts[13].pubkey,
        amm_authority: accounts[14].pubkey,
        amm_open_orders: accounts[15].pubkey,
        amm_lp_mint: accounts[16].pubkey,
        amm_base_vault: accounts[17].pubkey,
        amm_quote_vault: accounts[18].pubkey,
        amm_target_orders: accounts[19].pubkey,
        amm_config: accounts[20].pubkey,
        amm_create_fee_destination: accounts[21].pubkey,
        authority: accounts[22].pubkey,
        pool_state: accounts[23].pubkey,
        global_config: accounts[24].pubkey,
        base_vault: accounts[25].pubkey,
        quote_vault: accounts[26].pubkey,
        pool_lp_token: accounts[27].pubkey,
        spl_token_program: accounts[28].pubkey,
        associated_token_program: accounts[29].pubkey,
        system_program: accounts[30].pubkey,
        rent_program: accounts[31].pubkey,
        ..Default::default()
    }))
}

/// Parse migrate to CP Swap event
fn parse_migrate_to_cpswap_instruction(
    _data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkMigrateToCpswap;

    Some(DexEvent::BonkMigrateToCpswapEvent(BonkMigrateToCpswapEvent {
        metadata,
        payer: accounts[0].pubkey,
        base_mint: accounts[1].pubkey,
        quote_mint: accounts[2].pubkey,
        platform_config: accounts[3].pubkey,
        cpswap_program: accounts[4].pubkey,
        cpswap_pool: accounts[5].pubkey,
        cpswap_authority: accounts[6].pubkey,
        cpswap_lp_mint: accounts[7].pubkey,
        cpswap_base_vault: accounts[8].pubkey,
        cpswap_quote_vault: accounts[9].pubkey,
        cpswap_config: accounts[10].pubkey,
        cpswap_create_pool_fee: accounts[11].pubkey,
        cpswap_observation: accounts[12].pubkey,
        lock_program: accounts[13].pubkey,
        lock_authority: accounts[14].pubkey,
        lock_lp_vault: accounts[15].pubkey,
        authority: accounts[16].pubkey,
        pool_state: accounts[17].pubkey,
        global_config: accounts[18].pubkey,
        base_vault: accounts[19].pubkey,
        quote_vault: accounts[20].pubkey,
        pool_lp_token: accounts[21].pubkey,
        base_token_program: accounts[22].pubkey,
        quote_token_program: accounts[23].pubkey,
        associated_token_program: accounts[24].pubkey,
        system_program: accounts[25].pubkey,
        rent_program: accounts[26].pubkey,
        metadata_program: accounts[27].pubkey,
        remaining_accounts: accounts[28..].iter().map(|m| m.pubkey).collect(),
        ..Default::default()
    }))
}
