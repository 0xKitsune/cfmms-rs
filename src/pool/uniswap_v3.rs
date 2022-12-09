use std::{
    collections::HashMap,
    ops::{Add, BitAnd, Shl, Shr},
    str::FromStr,
    sync::Arc,
};

use ethers::{
    abi::{decode, ParamType},
    prelude::k256::elliptic_curve::consts::{U160, U2},
    providers::{JsonRpcClient, Provider},
    types::{H160, I256, U256},
};
use num_bigfloat::BigFloat;

use crate::{abi, error::PairSyncError};

#[derive(Clone, Copy)]
pub struct UniswapV3Pool {
    pub address: H160,
    pub token_a: H160,
    pub token_a_decimals: u8,
    pub token_b: H160,
    pub token_b_decimals: u8,
    pub liquidity: u128,
    pub sqrt_price: U256,
    pub fee: u32,
    pub tick: i32,
    pub tick_spacing: i32,
    pub liquidity_net: i128,
    pub initialized: bool,
    pub tick_word: U256,
}

impl UniswapV3Pool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: H160,
        token_a: H160,
        token_a_decimals: u8,
        token_b: H160,
        token_b_decimals: u8,
        liquidity: u128,
        sqrt_price: U256,
        tick: i32,
        tick_spacing: i32,
        liquidity_net: i128,
        initialized: bool,
        fee: u32,
        tick_word: U256,
    ) -> UniswapV3Pool {
        UniswapV3Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            liquidity,
            sqrt_price,
            tick,
            tick_spacing,
            liquidity_net,
            initialized,
            fee,
            tick_word,
        }
    }

    //Creates a new instance of the pool from the pair address
    pub async fn new_from_address<P: 'static + JsonRpcClient>(
        pair_address: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<Self, PairSyncError<P>> {
        let mut pool = UniswapV3Pool {
            address: pair_address,
            token_a: H160::zero(),
            token_a_decimals: 0,
            token_b: H160::zero(),
            token_b_decimals: 0,
            liquidity: 0,
            sqrt_price: U256::zero(),
            tick: 0,
            tick_spacing: 0,
            liquidity_net: 0,
            initialized: false,
            tick_word: U256::zero(),
            fee: 0,
        };

        (pool.token_a_decimals, pool.token_b_decimals) =
            pool.get_token_decimals(provider.clone()).await?;
        pool.token_a = pool.get_token_0(provider.clone()).await?;
        pool.token_b = pool.get_token_1(provider.clone()).await?;
        pool.fee = pool.get_fee(provider.clone()).await?;
        pool.tick_spacing = pool.get_tick_spacing(provider.clone()).await?;
        pool.liquidity = pool.get_liquidity(provider.clone()).await?;

        let slot_0 = pool.get_slot_0(provider.clone()).await?;
        pool.tick = slot_0.1;
        pool.sqrt_price = slot_0.0;
        //256 bit word surrounding the current tick

        let tick_info = pool.get_tick_info(pool.tick, provider.clone()).await?;
        pool.liquidity_net = tick_info.1;
        pool.initialized = tick_info.7;
        pool.get_pool_data(provider.clone()).await?;

        Ok(pool)
    }

    pub async fn get_pool_data<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        self.token_a = self.get_token_0(provider.clone()).await?;
        self.token_b = self.get_token_1(provider.clone()).await?;

        (self.token_a_decimals, self.token_b_decimals) =
            self.get_token_decimals(provider.clone()).await?;

        self.fee = self.get_fee(provider.clone()).await?;
        self.tick_spacing = self.get_tick_spacing(provider.clone()).await?;
        self.tick_word = self.get_tick_word(provider.clone()).await?;
        Ok(())
    }

    pub async fn get_tick_word<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<U256, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        let (position, word) = uniswap_v3_math::tick_bit_map::position(self.tick);
        Ok(v3_pool.tick_bitmap(position).call().await?)
    }

    pub async fn get_tick_spacing<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<i32, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        Ok(v3_pool.tick_spacing().call().await?)
    }

    pub async fn get_tick<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<i32, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        Ok(self.get_slot_0(provider).await?.1)
    }

    pub async fn get_tick_info<P: JsonRpcClient>(
        &self,
        tick: i32,
        provider: Arc<Provider<P>>,
    ) -> Result<(u128, i128, U256, U256, i64, U256, u32, bool), PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());

        let tick_info_bytes = v3_pool.ticks(tick).call().await?;

        let tick_info = decode(
            &vec![
                ParamType::Uint(128), //liquidityGross
                ParamType::Int(128),  //liquidityNet
                ParamType::Uint(256), //feeGrowthOutside0X128
                ParamType::Uint(256), //feeGrowthOutside1X128
                ParamType::Int(64),   //tickCumulativeOutside
                ParamType::Uint(256), //secondsPerLiquidityOutsideX128
                ParamType::Uint(32),  //secondsOutside
                ParamType::Bool,      //initialized
            ],
            &tick_info_bytes,
        )
        .expect("Could not get log data");

        let liquidity_gross = tick_info[0]
            .to_owned()
            .into_uint()
            .expect("Could not convert liquidityGross into Uint")
            .as_u128();

        let liquidity_net = I256::from_raw(
            tick_info[1]
                .to_owned()
                .into_int()
                .expect("Could not convert liquidityNet to Int"),
        )
        .as_i128();

        let fee_growth_outside_0_x_128 = tick_info[2]
            .to_owned()
            .into_uint()
            .expect("Could not convert feeGrowthOutside0X128 into Uint");

        let fee_growth_outside_1_x_128 = tick_info[3]
            .to_owned()
            .into_uint()
            .expect("Could not convert feeGrowthOutside1X128 to Uint");

        let tick_cumulative_outside = I256::from_raw(
            tick_info[4]
                .to_owned()
                .into_int()
                .expect("Could not convert tickCumulativeOutside to Int"),
        )
        .as_i64();

        let seconds_per_liquidity_outside_x_128 = tick_info[5]
            .to_owned()
            .into_uint()
            .expect("Could not convert secondsPerLiquidityOutsideX128 to Uint");

        let seconds_outside = tick_info[6]
            .to_owned()
            .into_uint()
            .expect("Could not convert secondsOutside to Uint")
            .as_u32();

        let initialized = tick_info[7]
            .to_owned()
            .into_bool()
            .expect("Coud not convert Initialzied into Bool");

        Ok((
            liquidity_gross,
            liquidity_net,
            fee_growth_outside_0_x_128,
            fee_growth_outside_1_x_128,
            tick_cumulative_outside,
            seconds_per_liquidity_outside_x_128,
            seconds_outside,
            initialized,
        ))
    }

    pub async fn get_liquidity_net<P: JsonRpcClient>(
        &self,
        tick: i32,
        provider: Arc<Provider<P>>,
    ) -> Result<i128, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        let tick_info = self.get_tick_info(tick, provider.clone()).await?;
        Ok(tick_info.1)
    }

    pub async fn get_initialized<P: JsonRpcClient>(
        &self,
        tick: i32,
        provider: Arc<Provider<P>>,
    ) -> Result<bool, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        let tick_info = self.get_tick_info(tick, provider.clone()).await?;
        Ok(tick_info.7)
    }

    pub async fn get_slot_0<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<(U256, i32, u16, u16, u16, u8, bool), PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        Ok(v3_pool.slot_0().call().await?)
    }

    pub async fn get_liquidity<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<u128, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        Ok(v3_pool.liquidity().call().await?)
    }

    pub async fn get_sqrt_price<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<U256, PairSyncError<P>> {
        Ok(self.get_slot_0(provider).await?.0)
    }

    //TODO: check this if we need anything else with the updates
    pub async fn sync_pool<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        self.liquidity = self.get_liquidity(provider.clone()).await?;
        self.sqrt_price = self.get_sqrt_price(provider.clone()).await?;

        Ok(())
    }

    pub async fn get_token_decimals<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(u8, u8), PairSyncError<P>> {
        let token_a_decimals = abi::IErc20::new(self.token_a, provider.clone())
            .decimals()
            .call()
            .await?;

        let token_b_decimals = abi::IErc20::new(self.token_b, provider)
            .decimals()
            .call()
            .await?;

        Ok((token_a_decimals, token_b_decimals))
    }

    pub async fn get_fee<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<u32, PairSyncError<P>> {
        let fee = abi::IUniswapV3Pool::new(self.address, provider.clone())
            .fee()
            .call()
            .await?;

        Ok(fee)
    }

    pub async fn get_token_0<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<H160, PairSyncError<P>> {
        let v2_pair = abi::IUniswapV2Pair::new(self.address, provider);

        let token0 = match v2_pair.token_0().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(PairSyncError::ContractError(contract_error)),
        };

        Ok(token0)
    }

    pub async fn get_token_1<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<H160, PairSyncError<P>> {
        let v2_pair = abi::IUniswapV2Pair::new(self.address, provider);

        let token1 = match v2_pair.token_1().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(PairSyncError::ContractError(contract_error)),
        };

        Ok(token1)
    }

    pub fn calculate_virtual_reserves(&self) -> (u128, u128) {
        let price = BigFloat::from_u128(
            ((self.sqrt_price.overflowing_mul(self.sqrt_price).0) >> 128).as_u128(),
        )
        .div(&BigFloat::from(2f64.powf(64.0)))
        .mul(&BigFloat::from_f64(10f64.powf(
            (self.token_a_decimals as i8 - self.token_b_decimals as i8) as f64,
        )));

        let sqrt_price = price.sqrt();
        let liquidity = BigFloat::from_u128(self.liquidity);

        //Sqrt price is stored as a Q64.96 so we need to left shift the liquidity by 96 to be represented as Q64.96
        //We cant right shift sqrt_price because it could move the value to 0, making divison by 0 to get reserve_x
        let liquidity = liquidity;

        let (reserve_0, reserve_1) = if !sqrt_price.is_zero() {
            let reserve_x = liquidity.div(&sqrt_price);
            let reserve_y = liquidity.mul(&sqrt_price);

            (reserve_x, reserve_y)
        } else {
            (BigFloat::from(0), BigFloat::from(0))
        };

        (
            reserve_0
                .to_u128()
                .expect("Could not convert reserve_0 to uin128"),
            reserve_1
                .to_u128()
                .expect("Could not convert reserve_1 to uin128"),
        )
    }

    pub fn calculate_price(&self, base_token: H160) -> f64 {
        let price = BigFloat::from_u128(
            ((self.sqrt_price.overflowing_mul(self.sqrt_price).0) >> 128).as_u128(),
        )
        .div(&BigFloat::from(2f64.powf(64.0)))
        .mul(&BigFloat::from_f64(10f64.powf(
            (self.token_a_decimals as i8 - self.token_b_decimals as i8) as f64,
        )));

        if self.token_a == base_token {
            price.to_f64()
        } else {
            1.0 / price.to_f64()
        }
    }

    pub fn address(&self) -> H160 {
        self.address
    }

    pub async fn simulate_swap<P: 'static + JsonRpcClient>(
        &self,
        token_in: H160,
        amount_in: u128,
        provider: Arc<Provider<P>>,
    ) -> Result<u128, PairSyncError<P>> {
        //Initialize zero_for_one to true if token_in is token_a
        let zero_for_one = token_in == self.token_a;

        //Set sqrt_price_limit_x_96 to the max or min sqrt price in the pool depending on zero_for_one
        let sqrt_price_limit_x_96 = if zero_for_one {
            MIN_SQRT_RATIO + 1
        } else {
            MAX_SQRT_RATIO - 1
        };

        //Initialize a mutable state state struct to hold the dynamic simulated state of the pool
        let mut current_state = CurrentState {
            sqrt_price_x_96: self.sqrt_price, //Active price on the pool
            amount_calculated: I256::zero(),  //Amount of token_out that has been calculated
            amount_specified_remaining: I256::from(amount_in), //Amount of token_in that has not been swapped
            tick: self.tick,                                   //Current i24 tick of the pool
            liquidity: self.liquidity, //Current available liquidity in the tick range
        };

        //Grab liquidity_net from the pool. This is the net liquidity change in the pool uppon crossing self.tick
        let liquidity_net = self.liquidity_net;

        //While there is still an amount remaining to be swapped
        //Loop through swap steps until the amount_specified_remaining is 0.
        while current_state.amount_specified_remaining > I256::zero() {
            //Initialize a new step struct to hold the dynamic state of the pool at each step
            let mut step = StepComputations::default();
            //Get the next initialized tick within one word of the current tick
            //@0xKitsune If step.initialized is 0, then there are no more initialized ticks left in the `current_word`.
            (step.tick_next, step.initialized) =
                uniswap_v3_math::tick_bit_map::next_initialized_tick_within_one_word(
                    self.tick_word,
                    current_state.tick,
                    self.tick_spacing,
                    zero_for_one,
                );

            //TODO: @0xKitsune If step.initialized is 0, then get the next word from the TickBitmap on the pool.
            // (word_position, bit_position)=position(self.tick) gives you the current word pos. The next word position
            // will be word_position + 1. Then you can get the next initialized tick from the next word.
            // By calling tickBitmap[word_position +1] on the contract.
            // But first you should add this logic before setting the step.tick_next to the next initialized tick.
            /*
               // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
                   if step.tick_next < MIN_TICK {
                       step.tick_next = MIN_TICK;
                   } else if step.tick_next > MAX_TICK {
                       step.tick_next = MAX_TICK;
                   }
            */
            let amount_in_remaining: U256 =
                U256::from(current_state.amount_specified_remaining.as_u128());

            //Get the next sqrt price from the input amount
            step.sqrt_price_next_x96 =
                uniswap_v3_math::sqrt_price_math::get_next_sqrt_price_from_input(
                    current_state.sqrt_price_x_96,
                    current_state.liquidity,
                    amount_in_remaining,
                    zero_for_one,
                )?;

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            if step.tick_next < MIN_TICK {
                step.tick_next = MIN_TICK;
            } else if step.tick_next > MAX_TICK {
                step.tick_next = MAX_TICK;
            }

            //Amount used during the swap for the input and output tokens
            let amount_used: U256;
            let amount_received: U256;
            let sqrt_price_at_tick_next =
                uniswap_v3_math::tick_math::get_sqrt_ratio_at_tick(step.tick_next)?;
            //Compute swap step and update the current state
            (
                current_state.sqrt_price_x_96,
                amount_used,
                amount_received,
                step.fee_amount,
            ) = uniswap_v3_math::swap_math::compute_swap_step(
                current_state.sqrt_price_x_96,
                sqrt_price_limit_x_96,
                self.liquidity,
                current_state.amount_specified_remaining,
                self.fee,
            )?;

            //Decrement the amount remaining to be swapped and amount received from the step
            current_state.amount_specified_remaining -=
                I256::from_raw(amount_used.add(step.fee_amount));
            current_state.amount_calculated -= I256::from_raw(amount_received);

            //If the price moved all the way to the next price, recompute the liquidity change for the next iteration
            if current_state.sqrt_price_x_96 == step.sqrt_price_next_x96 {
                if step.initialized {
                    // we are on a tick boundary, and the next tick is initialized, so we must charge a protocol fee
                    if zero_for_one {
                        let liquidity_temp = uniswap_v3_math::full_math::mul_div(
                            current_state.sqrt_price_x_96,
                            step.sqrt_price_next_x96,
                            U256::from(0x1000000000000000000000000),
                        )?;
                        current_state.liquidity = uniswap_v3_math::full_math::mul_div(
                            amount_used,
                            liquidity_temp,
                            current_state.sqrt_price_x_96 - step.sqrt_price_next_x96,
                        )?
                        .as_u128();
                    } else {
                        current_state.liquidity = uniswap_v3_math::full_math::mul_div(
                            amount_used,
                            U256::from(0x1000000000000000000000000),
                            step.sqrt_price_next_x96 - current_state.sqrt_price_x_96,
                        )?
                        .as_u128();
                    }
                }
                //Increment the current tick
                current_state.tick = if zero_for_one {
                    current_state.tick - 1
                } else {
                    current_state.tick + 1
                }
            } else if current_state.sqrt_price_x_96 != self.sqrt_price {
                current_state.tick = uniswap_v3_math::sqrt_price_math::get_tick_at_sqrt_ratio(
                    current_state.sqrt_price_x_96,
                )?;
            }
        }

        //TODO: update this
        Ok(0)
    }

    pub fn simulate_swap_mut(&mut self, token_in: H160, amount_in: u128) -> u128 {
        //TODO: update this
        0
    }
}

//TODO: we can bench using a struct vs not and decide if we are keeping the struct
pub struct CurrentState {
    amount_specified_remaining: I256,
    amount_calculated: I256,
    sqrt_price_x_96: U256,
    tick: i32,
    liquidity: u128,
}

#[derive(Default)]
pub struct StepComputations {
    sqrt_price_start_x_96: U256,
    tick_next: i32,
    initialized: bool,
    sqrt_price_next_x96: U256,
    amount_in: U256,
    amount_out: U256,
    fee_amount: U256,
}

const MAX_SQRT_RATIO: U256 = U256::from(4295128739);
const MIN_SQRT_RATIO: U256 = U256::from("0xFFFD8963EFD1FC6A506488495D951D5263988D26");
const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

pub struct Tick {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub fee_growth_outside_0_x_128: U256,
    pub fee_growth_outside_1_x_128: U256,
    pub tick_cumulative_outside: U256,
    pub seconds_per_liquidity_outside_x_128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}
