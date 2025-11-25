use anchor_lang::InitSpace;

#[derive(Debug, InitSpace, Default)]
pub struct BaseFeeConfig {
    pub cliff_fee_numerator: u64,
    // In fee scheduler first_factor: number_of_period, second_factor: period_frequency, third_factor: reduction_factor
    // in rate limiter: first_factor: fee_increment_bps, second_factor: max_limiter_duration, max_fee_bps, third_factor: reference_amount
    pub base_fee_mode: u8,
    pub padding: [u8; 5],
    pub first_factor: u16,
    pub second_factor: [u8; 8],
    pub third_factor: u64,
}

#[derive(Debug, InitSpace, Default)]
pub struct DynamicFeeConfig {
    pub initialized: u8, // 0, ignore for dynamic fee
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub padding_1: [u8; 8], // Align to 16 bytes for `u128`
    pub bin_step_u128: u128,
}

#[derive(Debug, InitSpace, Default)]
pub struct PoolFeesConfig {
    pub base_fee: BaseFeeConfig,
    pub dynamic_fee: DynamicFeeConfig,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub padding_0: [u8; 5],
    pub padding_1: [u64; 5],
}