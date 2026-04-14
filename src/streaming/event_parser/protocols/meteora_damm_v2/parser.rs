use crate::streaming::event_parser::{
    common::{EventMetadata, EventType},
    DexEvent,
};
use solana_sdk::instruction::AccountMeta;
use crate::streaming::event_parser::protocols::meteora_damm_v2;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::{discriminators, meteora_damm_v2_initialize_pool_event_decode, meteora_damm_v2_swap_event_decode, MeteoraDammV2InitializeCustomizablePoolEvent, MeteoraDammV2InitializePoolEvent, MeteoraDammV2InitializePoolWithDynamicConfigEvent, MeteoraDammV2Swap2Event, MeteoraDammV2SwapEvent};

/// 解析 Meteora DAMM v2 instruction data
///
/// 根据判别器路由到具体的 instruction 解析函数
pub fn parse_meteora_damm_v2_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[AccountMeta],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP_IX => parse_swap_instruction(data, accounts, metadata),
        discriminators::SWAP2_IX => parse_swap2_instruction(data, accounts, metadata),
        discriminators::INITIALIZE_POOL_IX => {
            parse_initialize_pool_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_CUSTOMIZABLE_POOL_IX => {
            parse_initialize_customizable_pool_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_POOL_WITH_DYNAMIC_CONFIG_IX => {
            parse_initialize_pool_with_dynamic_config_instruction(data, accounts, metadata)
        }
        _ => None,
    }
}

/// 解析 Meteora DAMM v2 inner instruction data (CPI events)
///
/// 根据判别器路由到具体的 inner instruction 解析函数
pub fn parse_meteora_damm_v2_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::SWAP_EVENT => parse_swap_inner_instruction(data, metadata),
        discriminators::INITIALIZE_POOL_EVENT => {
            parse_initialize_pool_inner_instruction(data, metadata)
        }
        _ => None,
    }
}

/// 解析 swap 指令
fn parse_swap_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2Swap;

    if data.len() < 16 || accounts.len() < 14 {
        return None;
    }

    // 跳过 discriminator (8 bytes)
    let amount_in = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let minimum_amount_out = u64::from_le_bytes(data[8..16].try_into().unwrap());

    Some(DexEvent::MeteoraDammV2SwapEvent(MeteoraDammV2SwapEvent {
        metadata,
        pool_authority: accounts[0].pubkey,
        pool: accounts[1].pubkey,
        input_token_account: accounts[2].pubkey,
        output_token_account: accounts[3].pubkey,
        token_a_vault: accounts[4].pubkey,
        token_b_vault: accounts[5].pubkey,
        token_a_mint: accounts[6].pubkey,
        token_b_mint: accounts[7].pubkey,
        payer: accounts[8].pubkey,
        token_a_program: accounts[9].pubkey,
        token_b_program: accounts[10].pubkey,
        referral_token_account: Some(accounts[11].pubkey),
        event_authority: accounts[12].pubkey,
        program: accounts[13].pubkey,
        amount_0: amount_in,
        amount_1: minimum_amount_out,
        ..Default::default()
    }))
}

/// 解析 swap2 指令
fn parse_swap2_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2Swap2;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    // 跳过 discriminator (8 bytes)
    let amount_0 = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let amount_1 = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let swap_mode = data[16];

    // swap2 可能有 15 个账户(带 referral)或 14 个账户
    let has_referral = accounts.len() >= 15;

    Some(DexEvent::MeteoraDammV2Swap2Event(MeteoraDammV2Swap2Event {
        metadata,
        pool_authority: accounts[0].pubkey,
        pool: accounts[1].pubkey,
        input_token_account: accounts[2].pubkey,
        output_token_account: accounts[3].pubkey,
        token_a_vault: accounts[4].pubkey,
        token_b_vault: accounts[5].pubkey,
        token_a_mint: accounts[6].pubkey,
        token_b_mint: accounts[7].pubkey,
        payer: accounts[8].pubkey,
        token_a_program: accounts[9].pubkey,
        token_b_program: accounts[10].pubkey,
        referral_token_account: if has_referral && accounts.len() > 11 {
            Some(accounts[11].pubkey)
        } else {
            None
        },
        event_authority: accounts[if has_referral { 12 } else { 11 }].pubkey,
        program: accounts[if has_referral { 13 } else { 12 }].pubkey,
        sysvar: accounts[if has_referral { 14 } else { 13 }].pubkey,
        amount_0,
        amount_1,
        swap_mode,
        has_referral,
        ..Default::default()
    }))
}

/// 解析 initialize_pool 指令
fn parse_initialize_pool_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2InitializePool;

    if accounts.len() < 20 {
        return None;
    }

    // 解析 instruction data (不包含 discriminator，已被调用者移除)
    // 结构: liquidity (u128 = 16 bytes) + sqrt_price (u128 = 16 bytes) + activation_point (Option<u64> = 1 + 8 bytes)
    if data.len() < 33 {
        return None;
    }

    let mut offset = 0;

    // 读取 liquidity (u128)
    let liquidity = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 sqrt_price (u128)
    let sqrt_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 activation_point (Option<u64>)
    let option_tag = data[offset];
    offset += 1;
    let _activation_point = if option_tag == 1 && data.len() >= offset + 8 {
        Some(u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?))
    } else {
        None
    };

    Some(DexEvent::MeteoraDammV2InitializePoolEvent(MeteoraDammV2InitializePoolEvent {
        metadata,
        creator: accounts[0].pubkey,
        position_nft_mint: accounts[1].pubkey,
        position_nft_account: accounts[2].pubkey,
        payer: accounts[3].pubkey,
        config: accounts[4].pubkey,
        pool_authority: accounts[5].pubkey,
        pool: accounts[6].pubkey,
        position: accounts[7].pubkey,
        token_a_mint: accounts[8].pubkey,
        token_b_mint: accounts[9].pubkey,
        token_a_vault: accounts[10].pubkey,
        token_b_vault: accounts[11].pubkey,
        payer_token_a: accounts[12].pubkey,
        payer_token_b: accounts[13].pubkey,
        token_a_program: accounts[14].pubkey,
        token_b_program: accounts[15].pubkey,
        event_authority: accounts[18].pubkey,
        program: accounts[19].pubkey,
        remaining_accounts: accounts[20..].iter().map(|m| m.pubkey).collect(),
        liquidity,
        sqrt_price,
        ..Default::default()
    }))
}

/// 解析 initialize_customizable_pool 指令
fn parse_initialize_customizable_pool_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2InitializeCustomizablePool;

    if accounts.len() < 19 {
        return None;
    }

    // 解析 instruction data (不包含 discriminator)
    // 结构: PoolFeeParameters + sqrt_min_price + sqrt_max_price + has_alpha_vault + liquidity + sqrt_price + activation_type + collect_fee_mode + activation_point
    if data.len() < 99 {
        return None;
    }

    let mut offset = 0;

    // 解析 PoolFeeParameters
    use crate::streaming::event_parser::protocols::meteora_damm_v2::events::PoolFeeParameters;
    use borsh::BorshDeserialize;

    // PoolFeeParameters size: 8 + 2 + 8 + 8 + 1 + 3 + 1 + (optional DynamicFee)
    // 先读取前 31 bytes (不包含 dynamic_fee option tag)
    let pool_fees = PoolFeeParameters::deserialize(&mut &data[offset..]).ok()?;

    // 计算 pool_fees 消耗的字节数
    // BaseFee: 8 + 2 + 8 + 8 + 1 = 27 bytes
    // padding: 3 bytes
    // option tag: 1 byte
    // 如果 dynamic_fee 存在: 2 + 16 + 2 + 2 + 2 + 4 + 4 = 32 bytes
    let pool_fees_size = 31 + if pool_fees.dynamic_fee.is_some() { 32 } else { 0 };
    offset += pool_fees_size;

    // 读取 sqrt_min_price (u128)
    let sqrt_min_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 sqrt_max_price (u128)
    let sqrt_max_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 has_alpha_vault (bool)
    let _has_alpha_vault = data[offset];
    offset += 1;

    // 读取 liquidity (u128)
    let liquidity = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 sqrt_price (u128)
    let sqrt_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 activation_type (u8)
    let activation_type = data[offset];
    offset += 1;

    // 读取 collect_fee_mode (u8)
    let collect_fee_mode = data[offset];
    offset += 1;

    // 读取 activation_point (Option<u64>)
    let option_tag = data[offset];
    let _activation_point = if option_tag == 1 && data.len() >= offset + 9 {
        Some(u64::from_le_bytes(data[offset + 1..offset + 9].try_into().ok()?))
    } else {
        None
    };

    Some(DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(
        MeteoraDammV2InitializeCustomizablePoolEvent {
            metadata,
            creator: accounts[0].pubkey,
            position_nft_mint: accounts[1].pubkey,
            position_nft_account: accounts[2].pubkey,
            payer: accounts[3].pubkey,
            pool_authority: accounts[4].pubkey,
            pool: accounts[5].pubkey,
            position: accounts[6].pubkey,
            token_a_mint: accounts[7].pubkey,
            token_b_mint: accounts[8].pubkey,
            token_a_vault: accounts[9].pubkey,
            token_b_vault: accounts[10].pubkey,
            payer_token_a: accounts[11].pubkey,
            payer_token_b: accounts[12].pubkey,
            token_a_program: accounts[13].pubkey,
            token_b_program: accounts[14].pubkey,
            token_2022_program: accounts[15].pubkey,
            system_program: accounts[16].pubkey,
            event_authority: accounts[17].pubkey,
            program: accounts[18].pubkey,
            remaining_accounts: accounts[19..].iter().map(|m| m.pubkey).collect(),
            pool_fees,
            sqrt_min_price,
            sqrt_max_price,
            activation_type,
            collect_fee_mode,
            liquidity,
            sqrt_price,
            ..Default::default()
        },
    ))
}

/// 解析 initialize_pool_with_dynamic_config 指令
fn parse_initialize_pool_with_dynamic_config_instruction(
    data: &[u8],
    accounts: &[AccountMeta],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2InitializePoolWithDynamicConfig;

    if accounts.len() < 21 {
        return None;
    }

    if data.len() < 99 {
        return None;
    }

    let mut offset = 0;

    // 解析 PoolFeeParameters
    use crate::streaming::event_parser::protocols::meteora_damm_v2::events::PoolFeeParameters;
    use borsh::BorshDeserialize;

    let pool_fees = PoolFeeParameters::deserialize(&mut &data[offset..]).ok()?;

    // 计算 pool_fees 消耗的字节数
    // BaseFee: 8 + 2 + 8 + 8 + 1 = 27 bytes
    // padding: 3 bytes
    // option tag: 1 byte
    // 如果 dynamic_fee 存在: 2 + 16 + 2 + 2 + 2 + 4 + 4 = 32 bytes
    let pool_fees_size = 31 + if pool_fees.dynamic_fee.is_some() { 32 } else { 0 };
    offset += pool_fees_size;

    // 读取 sqrt_min_price (u128)
    let sqrt_min_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 sqrt_max_price (u128)
    let sqrt_max_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 has_alpha_vault (bool)
    let _has_alpha_vault = data[offset];
    offset += 1;

    // 读取 liquidity (u128)
    let liquidity = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 sqrt_price (u128)
    let sqrt_price = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // 读取 activation_type (u8)
    let activation_type = data[offset];
    offset += 1;

    // 读取 collect_fee_mode (u8)
    let collect_fee_mode = data[offset];
    offset += 1;

    // 读取 activation_point (Option<u64>)
    let option_tag = data[offset];
    let _activation_point = if option_tag == 1 && data.len() >= offset + 9 {
        Some(u64::from_le_bytes(data[offset + 1..offset + 9].try_into().ok()?))
    } else {
        None
    };

    Some(DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(
        MeteoraDammV2InitializePoolWithDynamicConfigEvent {
            metadata,
            creator: accounts[0].pubkey,
            position_nft_mint: accounts[1].pubkey,
            position_nft_account: accounts[2].pubkey,
            payer: accounts[3].pubkey,
            pool_creator_authority: accounts[4].pubkey,
            pool_authority: accounts[6].pubkey,
            pool: accounts[7].pubkey,
            position: accounts[8].pubkey,
            token_a_mint: accounts[9].pubkey,
            token_b_mint: accounts[10].pubkey,
            token_a_vault: accounts[11].pubkey,
            token_b_vault: accounts[12].pubkey,
            payer_token_a: accounts[13].pubkey,
            payer_token_b: accounts[14].pubkey,
            token_a_program: accounts[15].pubkey,
            token_b_program: accounts[16].pubkey,
            token_2022_program: accounts[17].pubkey,
            system_program: accounts[18].pubkey,
            event_authority: accounts[19].pubkey,
            program: accounts[20].pubkey,
            config: accounts[5].pubkey,
            pool_fees,
            sqrt_min_price,
            sqrt_max_price,
            activation_type,
            collect_fee_mode,
            liquidity,
            sqrt_price,
            ..Default::default()
        },
    ))
}

/// 解析 swap inner instruction (CPI event)
fn parse_swap_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    if let Some(event) = meteora_damm_v2_swap_event_decode(data) {
        Some(DexEvent::MeteoraDammV2SwapEvent(MeteoraDammV2SwapEvent { metadata, ..event }))
    } else {
        None
    }
}

/// 解析 initialize pool inner instruction (CPI event)
fn parse_initialize_pool_inner_instruction(
    data: &[u8],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::MeteoraDammV2InitializePool;
    if let Some(event) = meteora_damm_v2_initialize_pool_event_decode(data) {
        Some(DexEvent::MeteoraDammV2InitializePoolEvent(MeteoraDammV2InitializePoolEvent {
            metadata,
            ..event
        }))
    } else {
        None
    }
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
