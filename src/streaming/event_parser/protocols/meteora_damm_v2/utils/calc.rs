use anchor_lang::prelude::*;
use crate::streaming::event_parser::protocols::meteora_damm_v2::math::u128x128_math::Rounding;
use crate::streaming::event_parser::protocols::meteora_damm_v2::types::Pool;
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::constants::fee::{get_max_fee_numerator, CURRENT_POOL_VERSION};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::curve::{get_delta_amount_a_unsigned, get_delta_amount_b_unsigned, get_next_sqrt_price_from_input};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::error::PoolError;
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::fee::{FeeMode, FeeOnAmountResult};
use crate::streaming::event_parser::protocols::meteora_damm_v2::utils::swap::TradeDirection;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct SwapResult2 {
    // This is excluded_transfer_fee_amount_in
    pub included_fee_input_amount: u64,
    pub excluded_fee_input_amount: u64,
    pub amount_left: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

pub struct SwapAmountFromInput {
    output_amount: u64,
    next_sqrt_price: u128,
    amount_left: u64,
}

impl Pool {
    fn calculate_a_to_b_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        // finding new target price
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, true)?;

        if next_sqrt_price < self.sqrt_min_price {
            return Err(PoolError::PriceRangeViolation.into());
        }

        // finding output amount
        let output_amount = get_delta_amount_b_unsigned(
            next_sqrt_price,
            self.sqrt_price,
            self.liquidity,
            Rounding::Down,
        )?;

        Ok(SwapAmountFromInput {
            output_amount,
            next_sqrt_price,
            amount_left: 0,
        })
    }

    fn calculate_b_to_a_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        // finding new target price
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, false)?;

        if next_sqrt_price > self.sqrt_max_price {
            return Err(PoolError::PriceRangeViolation.into());
        }
        // finding output amount
        let output_amount = get_delta_amount_a_unsigned(
            self.sqrt_price,
            next_sqrt_price,
            self.liquidity,
            Rounding::Down,
        )?;

        Ok(SwapAmountFromInput {
            output_amount,
            next_sqrt_price,
            amount_left: 0,
        })
    }

    pub fn has_partner(&self) -> bool {
        self.partner != Pubkey::default()
    }

    pub fn get_swap_result_from_exact_input(
        &self,
        amount_in: u64,
        fee_mode: &FeeMode,
        trade_direction: TradeDirection,
        current_point: u64,
    ) -> Result<SwapResult2> {
        let mut actual_protocol_fee = 0;
        let mut actual_trading_fee = 0;
        let mut actual_referral_fee = 0;
        let mut actual_partner_fee = 0;

        let max_fee_numerator = get_max_fee_numerator(CURRENT_POOL_VERSION)?;

        // We can compute the trade_fee_numerator here. Instead of separately for amount_in, and amount_out.
        // This is because FeeRateLimiter (fee rate scale based on amount) only applied when fee_mode.fees_on_input
        // (a.k.a TradeDirection::QuoteToBase + CollectFeeMode::QuoteToken)
        // For the rest of the time, the fee rate is not dependent on amount.
        let trade_fee_numerator = self
            .pool_fees
            .get_total_trading_fee_from_included_fee_amount(
                current_point,
                self.activation_point,
                amount_in,
                trade_direction,
                max_fee_numerator,
            )?;

        let actual_amount_in = if fee_mode.fees_on_input {
            let FeeOnAmountResult {
                amount,
                trading_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                amount_in,
                trade_fee_numerator,
                fee_mode.has_referral,
                self.has_partner(),
            )?;

            actual_protocol_fee = protocol_fee;
            actual_trading_fee = trading_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount
        } else {
            amount_in
        };

        let SwapAmountFromInput {
            output_amount,
            next_sqrt_price,
            amount_left,
        } = match trade_direction {
            TradeDirection::AtoB => self.calculate_a_to_b_from_amount_in(actual_amount_in),
            TradeDirection::BtoA => self.calculate_b_to_a_from_amount_in(actual_amount_in),
        }?;

        let actual_amount_out = if fee_mode.fees_on_input {
            output_amount
        } else {
            let FeeOnAmountResult {
                amount,
                trading_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                output_amount,
                trade_fee_numerator,
                fee_mode.has_referral,
                self.has_partner(),
            )?;

            actual_protocol_fee = protocol_fee;
            actual_trading_fee = trading_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount
        };

        Ok(SwapResult2 {
            amount_left,
            included_fee_input_amount: amount_in,
            excluded_fee_input_amount: actual_amount_in,
            output_amount: actual_amount_out,
            next_sqrt_price,
            trading_fee: actual_trading_fee,
            protocol_fee: actual_protocol_fee,
            partner_fee: actual_partner_fee,
            referral_fee: actual_referral_fee,
        })
    }
}

