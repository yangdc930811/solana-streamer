use std::ops::{BitAnd, BitOr, BitXor};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::protocols::raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID;
use anchor_lang::{err, require, require_keys_eq, Result};
use borsh_derive::BorshDeserialize;
use ruint::aliases::U1024;
use serde::{Deserialize, Serialize};
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::DexEvent;
use crate::streaming::event_parser::protocols::raydium_clmm::error::ErrorCode;
use crate::streaming::event_parser::protocols::raydium_clmm::libraries::{check_current_tick_array_is_initialized, tick_array_bit_map, tick_math};
use crate::streaming::event_parser::protocols::raydium_clmm::RaydiumClmmPoolStateAccountEvent;
use crate::streaming::event_parser::protocols::raydium_clmm::states::tick_array::TickArrayState;
use crate::streaming::event_parser::protocols::raydium_clmm::states::tickarray_bitmap_extension::TickArrayBitmapExtension;
use crate::streaming::grpc::AccountPretty;

/// Seed to derive account address and signature
pub const POOL_SEED: &str = "pool";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_REWARD_VAULT_SEED: &str = "pool_reward_vault";
pub const POOL_TICK_ARRAY_BITMAP_SEED: &str = "pool_tick_array_bitmap_extension";
// Number of rewards Token
pub const REWARD_NUM: usize = 3;

pub enum PoolStatusBitIndex {
    OpenPositionOrIncreaseLiquidity,
    DecreaseLiquidity,
    CollectFee,
    CollectReward,
    Swap,
}

#[derive(PartialEq, Eq)]
pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}

/// The pool state
///
/// PDA of `[POOL_SEED, config, token_mint_0, token_mint_1]`
///
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PoolState {
    /// Bump to identify PDA
    pub bump: [u8; 1],
    // Which config the pool belongs
    pub amm_config: Pubkey,
    // Pool creator
    pub owner: Pubkey,

    /// Token pair of the pool, where token_mint_0 address < token_mint_1 address
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,

    /// Token pair vault
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,

    /// observation account key
    pub observation_key: Pubkey,

    /// mint0 and mint1 decimals
    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,

    /// The minimum number of ticks between initialized ticks
    pub tick_spacing: u16,
    /// The currently in range liquidity available to the pool.
    pub liquidity: u128,
    /// The current price of the pool as a sqrt(token_1/token_0) Q64.64 value
    pub sqrt_price_x64: u128,
    /// The current tick of the pool, i.e. according to the last tick transition that was run.
    pub tick_current: i32,

    pub padding3: u16,
    pub padding4: u16,

    /// The fee growth as a Q64.64 number, i.e. fees of token_0 and token_1 collected per
    /// unit of liquidity for the entire life of the pool.
    pub fee_growth_global_0_x64: u128,
    pub fee_growth_global_1_x64: u128,

    /// The amounts of token_0 and token_1 that are owed to the protocol.
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    /// The amounts in and out of swap token_0 and token_1
    pub swap_in_amount_token_0: u128,
    pub swap_out_amount_token_1: u128,
    pub swap_in_amount_token_1: u128,
    pub swap_out_amount_token_0: u128,

    /// Bitwise representation of the state of the pool
    /// bit0, 1: disable open position and increase liquidity, 0: normal
    /// bit1, 1: disable decrease liquidity, 0: normal
    /// bit2, 1: disable collect fee, 0: normal
    /// bit3, 1: disable collect reward, 0: normal
    /// bit4, 1: disable swap, 0: normal
    pub status: u8,
    /// Leave blank for future use
    pub padding: [u8; 7],

    pub reward_infos: [RewardInfo; REWARD_NUM],

    /// Packed initialized tick array state
    pub tick_array_bitmap: [u64; 16],

    /// except protocol_fee and fund_fee
    pub total_fees_token_0: u64,
    /// except protocol_fee and fund_fee
    pub total_fees_claimed_token_0: u64,
    pub total_fees_token_1: u64,
    pub total_fees_claimed_token_1: u64,

    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,

    // The timestamp allowed for swap in the pool.
    // Note: The open_time is disabled for now.
    pub open_time: u64,
    // account recent update epoch
    pub recent_epoch: u64,

    // Unused bytes for future upgrades.
    pub padding1: [u64; 24],
    pub padding2: [u64; 32],
}

impl PoolState {
    pub const LEN: usize = 8
        + 1
        + 32 * 7
        + 1
        + 1
        + 2
        + 16
        + 16
        + 4
        + 2
        + 2
        + 16
        + 16
        + 8
        + 8
        + 16
        + 16
        + 16
        + 16
        + 8
        + RewardInfo::LEN * REWARD_NUM
        + 8 * 16
        + 512;

    pub fn seeds(&self) -> [&[u8]; 5] {
        [
            &POOL_SEED.as_bytes(),
            self.amm_config.as_ref(),
            self.token_mint_0.as_ref(),
            self.token_mint_1.as_ref(),
            self.bump.as_ref(),
        ]
    }

    pub fn key(&self) -> Pubkey {
        Pubkey::create_program_address(&self.seeds(), &RAYDIUM_CLMM_PROGRAM_ID).unwrap()
    }

    pub fn get_tick_array_offset(&self, tick_array_start_index: i32) -> Result<usize> {
        require!(
            TickArrayState::check_is_valid_start_index(tick_array_start_index, self.tick_spacing),
            ErrorCode::InvalidTickIndex
        );
        let tick_array_offset_in_bitmap = tick_array_start_index
            / TickArrayState::tick_count(self.tick_spacing)
            + tick_array_bit_map::TICK_ARRAY_BITMAP_SIZE;
        Ok(tick_array_offset_in_bitmap as usize)
    }

    pub fn get_first_initialized_tick_array(
        &self,
        tickarray_bitmap_extension: &mut Option<TickArrayBitmapExtension>,
        zero_for_one: bool,
    ) -> Result<(bool, i32)> {
        let (is_initialized, start_index) =
            if self.is_overflow_default_tickarray_bitmap(vec![self.tick_current]) {
                tickarray_bitmap_extension.as_ref()
                    .unwrap()
                    .check_tick_array_is_initialized(
                        TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing),
                        self.tick_spacing,
                    )?
            } else {
                let u1024 = U1024::from_limbs(self.tick_array_bitmap);
                check_current_tick_array_is_initialized(
                    u1024,
                    self.tick_current,
                    self.tick_spacing.into(),
                )?
            };
        if is_initialized {
            return Ok((true, start_index));
        }
        let next_start_index = self.next_initialized_tick_array_start_index(
            tickarray_bitmap_extension,
            TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing),
            zero_for_one,
        )?;
        require!(
            next_start_index.is_some(),
            ErrorCode::InsufficientLiquidityForDirection
        );
        return Ok((false, next_start_index.unwrap()));
    }

    pub fn next_initialized_tick_array_start_index(
        &self,
        tickarray_bitmap_extension: &Option<TickArrayBitmapExtension>,
        mut last_tick_array_start_index: i32,
        zero_for_one: bool,
    ) -> Result<Option<i32>> {
        last_tick_array_start_index =
            TickArrayState::get_array_start_index(last_tick_array_start_index, self.tick_spacing);

        loop {
            let u1024 = U1024::from_limbs(self.tick_array_bitmap);
            let (is_found, start_index) =
                tick_array_bit_map::next_initialized_tick_array_start_index(
                    u1024,
                    last_tick_array_start_index,
                    self.tick_spacing,
                    zero_for_one,
                );
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if tickarray_bitmap_extension.is_none() {
                return err!(ErrorCode::MissingTickArrayBitmapExtensionAccount);
            }

            let (is_found, start_index) = tickarray_bitmap_extension.as_ref()
                .unwrap()
                .next_initialized_tick_array_from_one_bitmap(
                    last_tick_array_start_index,
                    self.tick_spacing,
                    zero_for_one,
                )?;
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if last_tick_array_start_index < tick_math::MIN_TICK
                || last_tick_array_start_index > tick_math::MAX_TICK
            {
                return Ok(None);
            }
        }
    }

    pub fn set_status(&mut self, status: u8) {
        self.status = status
    }

    pub fn set_status_by_bit(&mut self, bit: PoolStatusBitIndex, flag: PoolStatusBitFlag) {
        let s = u8::from(1) << (bit as u8);
        if flag == PoolStatusBitFlag::Disable {
            self.status = self.status.bitor(s);
        } else {
            let m = u8::from(255).bitxor(s);
            self.status = self.status.bitand(m);
        }
    }

    /// Get status by bit, if it is `noraml` status, return true
    pub fn get_status_by_bit(&self, bit: PoolStatusBitIndex) -> bool {
        let status = u8::from(1) << (bit as u8);
        self.status.bitand(status) == 0
    }

    pub fn is_overflow_default_tickarray_bitmap(&self, tick_indexs: Vec<i32>) -> bool {
        let (min_tick_array_start_index_boundary, max_tick_array_index_boundary) =
            self.tick_array_start_index_range();
        for tick_index in tick_indexs {
            let tick_array_start_index =
                TickArrayState::get_array_start_index(tick_index, self.tick_spacing);
            if tick_array_start_index >= max_tick_array_index_boundary
                || tick_array_start_index < min_tick_array_start_index_boundary
            {
                return true;
            }
        }
        false
    }

    // the range of tick array start index that default tickarray bitmap can represent
    // if tick_spacing = 1, the result range is [-30720, 30720)
    pub fn tick_array_start_index_range(&self) -> (i32, i32) {
        // the range of ticks that default tickarrary can represent
        let mut max_tick_boundary =
            tick_array_bit_map::max_tick_in_tickarray_bitmap(self.tick_spacing);
        let mut min_tick_boundary = -max_tick_boundary;
        if max_tick_boundary > tick_math::MAX_TICK {
            max_tick_boundary =
                TickArrayState::get_array_start_index(tick_math::MAX_TICK, self.tick_spacing);
            // find the next tick array start index
            max_tick_boundary = max_tick_boundary + TickArrayState::tick_count(self.tick_spacing);
        }
        if min_tick_boundary < tick_math::MIN_TICK {
            min_tick_boundary =
                TickArrayState::get_array_start_index(tick_math::MIN_TICK, self.tick_spacing);
        }
        (min_tick_boundary, max_tick_boundary)
    }
}

pub const POOL_STATE_SIZE: usize = 1536;

pub fn pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[..POOL_STATE_SIZE]).ok()
}

pub fn direct_pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[8..POOL_STATE_SIZE + 8]).ok()
}

pub fn pool_state_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmPoolState;

    if account.data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    if let Some(pool_state) = pool_state_decode(&account.data[8..POOL_STATE_SIZE + 8]) {
        Some(DexEvent::RaydiumClmmPoolStateAccountEvent(RaydiumClmmPoolStateAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool_state: pool_state,
        }))
    } else {
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// State of reward
pub enum RewardState {
    /// Reward not initialized
    Uninitialized,
    /// Reward initialized, but reward time is not start
    Initialized,
    /// Reward in progress
    Opening,
    /// Reward end, reward time expire or
    Ended,
}

#[repr(C, packed)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, BorshDeserialize)]
pub struct RewardInfo {
    /// Reward state
    pub reward_state: u8,
    /// Reward open time
    pub open_time: u64,
    /// Reward end time
    pub end_time: u64,
    /// Reward last update time
    pub last_update_time: u64,
    /// Q64.64 number indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// The total amount of reward emissioned
    pub reward_total_emissioned: u64,
    /// The total amount of claimed reward
    pub reward_claimed: u64,
    /// Reward token mint.
    pub token_mint: Pubkey,
    /// Reward vault token account.
    pub token_vault: Pubkey,
    /// The owner that has permission to set reward param
    pub authority: Pubkey,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub reward_growth_global_x64: u128,
}

impl RewardInfo {
    pub const LEN: usize = 1 + 8 + 8 + 8 + 16 + 8 + 8 + 32 + 32 + 32 + 16;

    /// Creates a new RewardInfo
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            ..Default::default()
        }
    }

    /// Returns true if this reward is initialized.
    /// Once initialized, a reward cannot transition back to uninitialized.
    pub fn initialized(&self) -> bool {
        self.token_mint.ne(&Pubkey::default())
    }

    pub fn get_reward_growths(reward_infos: &[RewardInfo; REWARD_NUM]) -> [u128; REWARD_NUM] {
        let mut reward_growths = [0u128; REWARD_NUM];
        for i in 0..REWARD_NUM {
            reward_growths[i] = reward_infos[i].reward_growth_global_x64;
        }
        reward_growths
    }
}