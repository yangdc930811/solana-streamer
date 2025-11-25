use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::streaming::event_parser::protocols::meteora_damm_v2::math::safe_math::SafeMath;
use crate::streaming::event_parser::protocols::meteora_damm_v2::math::u128x128_math::Rounding;
use crate::streaming::event_parser::protocols::meteora_damm_v2::math::utils_math::{safe_mul_div_cast_u64, safe_shl_div_cast};
use crate::streaming::event_parser::protocols::meteora_damm_v2::types::{BaseFeeStruct, DynamicFeeStruct, PoolFeesStruct};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::base_fee::{get_base_fee_handler, BaseFeeHandler, BaseFeeMode};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::constants::{BASIS_POINT_MAX, ONE_Q64};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::constants::fee::FEE_DENOMINATOR;
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::error::PoolError;
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::fee_rate_limiter::FeeRateLimiter;
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::swap::TradeDirection;

#[derive(Default, Debug)]
pub struct FeeMode {
    pub fees_on_input: bool,
    pub fees_on_token_a: bool,
    pub has_referral: bool,
}

#[derive(Debug, PartialEq)]
pub struct FeeOnAmountResult {
    pub amount: u64,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    IntoPrimitive,
    TryFromPrimitive,
)]
pub enum CollectFeeMode {
    /// Both token, in this mode only out token is collected
    BothToken,
    /// Only token B, we just need token B, because if user want to collect fee in token A, they just need to flip order of tokens
    OnlyB,
}

impl FeeMode {
    pub fn get_fee_mode(
        collect_fee_mode: u8,
        trade_direction: TradeDirection,
        has_referral: bool,
    ) -> Result<FeeMode> {
        let collect_fee_mode = CollectFeeMode::try_from(collect_fee_mode)
            .map_err(|_| PoolError::InvalidCollectFeeMode)?;

        let (fees_on_input, fees_on_token_a) = match (collect_fee_mode, trade_direction) {
            // When collecting fees on output token
            (CollectFeeMode::BothToken, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::BothToken, TradeDirection::BtoA) => (false, true),

            // When collecting fees on tokenB
            (CollectFeeMode::OnlyB, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::OnlyB, TradeDirection::BtoA) => (true, false),
        };

        Ok(FeeMode {
            fees_on_input,
            fees_on_token_a,
            has_referral,
        })
    }
}

impl BaseFeeStruct {
    pub fn get_fee_rate_limiter(&self) -> Result<FeeRateLimiter> {
        let base_fee_mode =
            BaseFeeMode::try_from(self.base_fee_mode).map_err(|_| PoolError::InvalidBaseFeeMode)?;
        if base_fee_mode == BaseFeeMode::RateLimiter {
            Ok(FeeRateLimiter {
                cliff_fee_numerator: self.cliff_fee_numerator,
                fee_increment_bps: self.first_factor,
                max_limiter_duration: u32::from_le_bytes(
                    self.second_factor[0..4]
                        .try_into()
                        .map_err(|_| PoolError::TypeCastFailed)?,
                ),
                max_fee_bps: u32::from_le_bytes(
                    self.second_factor[4..8]
                        .try_into()
                        .map_err(|_| PoolError::TypeCastFailed)?,
                ),
                reference_amount: self.third_factor,
            })
        } else {
            Err(PoolError::InvalidFeeRateLimiter.into())
        }
    }

    pub fn get_base_fee_handler(&self) -> Result<Box<dyn BaseFeeHandler>> {
        get_base_fee_handler(
            self.cliff_fee_numerator,
            self.first_factor,
            self.second_factor,
            self.third_factor,
            self.base_fee_mode,
        )
    }
}

impl DynamicFeeStruct {
    // we approximate Px / Py = (1 + b) ^ delta_bin  = 1 + b * delta_bin (if b is too small)
    // Ex: (1+1/10000)^ 5000 / (1+5000 * 1/10000) = 1.1 (10% diff if sqrt_price diff is (1+1/10000)^ 5000 = 1.64 times)
    pub fn get_delta_bin_id(
        bin_step_u128: u128,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
    ) -> Result<u128> {
        let (upper_sqrt_price, lower_sqrt_price) = if sqrt_price_a > sqrt_price_b {
            (sqrt_price_a, sqrt_price_b)
        } else {
            (sqrt_price_b, sqrt_price_a)
        };

        let price_ratio: u128 =
            safe_shl_div_cast(upper_sqrt_price, lower_sqrt_price, 64, Rounding::Down)?;

        let delta_bin_id = price_ratio.safe_sub(ONE_Q64)?.safe_div(bin_step_u128)?;

        Ok(delta_bin_id.safe_mul(2)?)
    }
    pub fn update_volatility_accumulator(&mut self, sqrt_price: u128) -> Result<()> {
        let delta_price =
            Self::get_delta_bin_id(self.bin_step_u128, sqrt_price, self.sqrt_price_reference)?;

        let volatility_accumulator = self
            .volatility_reference
            .safe_add(delta_price.safe_mul(BASIS_POINT_MAX.into())?)?;

        self.volatility_accumulator = std::cmp::min(
            volatility_accumulator,
            self.max_volatility_accumulator.into(),
        );
        Ok(())
    }

    pub fn update_references(
        &mut self,
        sqrt_price_current: u128,
        current_timestamp: u64,
    ) -> Result<()> {
        // it is fine to use saturating_sub, because never a chance current_timestamp is lesser than last_update_timestamp on-chain
        // but that can benefit off-chain components for simulation when clock is not synced and pool is high frequency trading
        // furthermore, the function doesn't update fee in pre-swap, so quoting won't be affected
        let elapsed = current_timestamp.saturating_sub(self.last_update_timestamp);
        // Not high frequency trade
        if elapsed >= self.filter_period as u64 {
            // Update sqrt of last transaction
            self.sqrt_price_reference = sqrt_price_current;
            // filter period < t < decay_period. Decay time window.
            if elapsed < self.decay_period as u64 {
                let volatility_reference = self
                    .volatility_accumulator
                    .safe_mul(self.reduction_factor.into())?
                    .safe_div(BASIS_POINT_MAX.into())?;

                self.volatility_reference = volatility_reference;
            }
            // Out of decay time window
            else {
                self.volatility_reference = 0;
            }
        }
        Ok(())
    }

    pub fn is_dynamic_fee_enable(&self) -> bool {
        self.initialized != 0
    }

    pub fn get_variable_fee(&self) -> Result<u128> {
        if self.is_dynamic_fee_enable() {
            let square_vfa_bin: u128 = self
                .volatility_accumulator
                .safe_mul(self.bin_step.into())?
                .checked_pow(2)
                .unwrap();
            // Variable fee control, volatility accumulator, bin step are in basis point unit (10_000)
            // This is 1e20. Which > 1e9. Scale down it to 1e9 unit and ceiling the remaining.
            let v_fee = square_vfa_bin.safe_mul(self.variable_fee_control.into())?;

            let scaled_v_fee = v_fee.safe_add(99_999_999_999)?.safe_div(100_000_000_000)?;

            Ok(scaled_v_fee)
        } else {
            Ok(0)
        }
    }
}

impl PoolFeesStruct {
    fn get_total_fee_numerator(
        &self,
        base_fee_numerator: u64,
        max_fee_numerator: u64,
    ) -> Result<u64> {
        let dynamic_fee = self.dynamic_fee.get_variable_fee()?;
        let total_fee_numerator = dynamic_fee.safe_add(base_fee_numerator.into())?;
        let total_fee_numerator: u64 = total_fee_numerator
            .try_into()
            .map_err(|_| PoolError::TypeCastFailed)?;

        if total_fee_numerator > max_fee_numerator {
            Ok(max_fee_numerator)
        } else {
            Ok(total_fee_numerator)
        }
    }

    // in numerator
    pub fn get_total_trading_fee_from_included_fee_amount(
        &self,
        current_point: u64,
        activation_point: u64,
        included_fee_amount: u64,
        trade_direction: TradeDirection,
        max_fee_numerator: u64,
    ) -> Result<u64> {
        let base_fee_handler = self.base_fee.get_base_fee_handler()?;

        let base_fee_numerator = base_fee_handler.get_base_fee_numerator_from_included_fee_amount(
            current_point,
            activation_point,
            trade_direction,
            included_fee_amount,
        )?;

        self.get_total_fee_numerator(base_fee_numerator, max_fee_numerator)
    }

    pub fn get_total_trading_fee_from_excluded_fee_amount(
        &self,
        current_point: u64,
        activation_point: u64,
        excluded_fee_amount: u64,
        trade_direction: TradeDirection,
        max_fee_numerator: u64,
    ) -> Result<u64> {
        let base_fee_handler = self.base_fee.get_base_fee_handler()?;

        let base_fee_numerator = base_fee_handler.get_base_fee_numerator_from_excluded_fee_amount(
            current_point,
            activation_point,
            trade_direction,
            excluded_fee_amount,
        )?;

        self.get_total_fee_numerator(base_fee_numerator, max_fee_numerator)
    }

    pub fn get_fee_on_amount(
        &self,
        amount: u64,
        trade_fee_numerator: u64,
        has_referral: bool,
        has_partner: bool,
    ) -> Result<FeeOnAmountResult> {
        let (amount, trading_fee) =
            PoolFeesStruct::get_excluded_fee_amount(trade_fee_numerator, amount)?;

        let SplitFees {
            trading_fee,
            protocol_fee,
            referral_fee,
            partner_fee,
        } = self.split_fees(trading_fee, has_referral, has_partner)?;

        Ok(FeeOnAmountResult {
            amount,
            trading_fee,
            protocol_fee,
            partner_fee,
            referral_fee,
        })
    }

    pub fn get_excluded_fee_amount(
        trade_fee_numerator: u64,
        included_fee_amount: u64,
    ) -> Result<(u64, u64)> {
        let trading_fee: u64 = safe_mul_div_cast_u64(
            included_fee_amount,
            trade_fee_numerator,
            FEE_DENOMINATOR,
            Rounding::Up,
        )?;
        let excluded_fee_amount = included_fee_amount.safe_sub(trading_fee)?;
        Ok((excluded_fee_amount, trading_fee))
    }

    pub fn get_included_fee_amount(
        trade_fee_numerator: u64,
        excluded_fee_amount: u64,
    ) -> Result<(u64, u64)> {
        let included_fee_amount: u64 = safe_mul_div_cast_u64(
            excluded_fee_amount,
            FEE_DENOMINATOR,
            FEE_DENOMINATOR.safe_sub(trade_fee_numerator)?,
            Rounding::Up,
        )?;
        let fee_amount = included_fee_amount.safe_sub(excluded_fee_amount)?;
        Ok((included_fee_amount, fee_amount))
    }

    pub fn split_fees(
        &self,
        fee_amount: u64,
        has_referral: bool,
        has_partner: bool,
    ) -> Result<SplitFees> {
        let protocol_fee = safe_mul_div_cast_u64(
            fee_amount,
            self.protocol_fee_percent.into(),
            100,
            Rounding::Down,
        )?;

        // update trading fee
        let trading_fee: u64 = fee_amount.safe_sub(protocol_fee)?;

        let referral_fee = if has_referral {
            safe_mul_div_cast_u64(
                protocol_fee,
                self.referral_fee_percent.into(),
                100,
                Rounding::Down,
            )?
        } else {
            0
        };

        let protocol_fee_after_referral_fee = protocol_fee.safe_sub(referral_fee)?;

        let partner_fee = if has_partner && self.partner_fee_percent > 0 {
            safe_mul_div_cast_u64(
                protocol_fee_after_referral_fee,
                self.partner_fee_percent.into(),
                100,
                Rounding::Down,
            )?
        } else {
            0
        };

        let protocol_fee = protocol_fee_after_referral_fee.safe_sub(partner_fee)?;

        Ok(SplitFees {
            trading_fee,
            protocol_fee,
            referral_fee,
            partner_fee,
        })
    }
}

pub struct SplitFees {
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub referral_fee: u64,
    pub partner_fee: u64,
}