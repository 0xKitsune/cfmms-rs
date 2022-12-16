use std::{ops::Add, sync::Arc};

use ethers::{
    abi::{decode, ParamType},
    providers::{JsonRpcClient, Middleware, Provider},
    types::{Log, H160, I256, U256},
};
use num_bigfloat::BigFloat;

use crate::{abi, error::CFFMError};

#[derive(Clone, Copy, Debug, Default)]
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
    pub tick_word: U256,
    pub liquidity_net: i128,
}

impl UniswapV3Pool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: H160,
        token_a: H160,
        token_a_decimals: u8,
        token_b: H160,
        token_b_decimals: u8,
        fee: u32,
        liquidity: u128,
        sqrt_price: U256,
        tick: i32,
        tick_spacing: i32,
        tick_word: U256,
        liquidity_net: i128,
    ) -> UniswapV3Pool {
        UniswapV3Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            fee,
            liquidity,
            sqrt_price,
            tick,
            tick_spacing,
            tick_word,
            liquidity_net,
        }
    }

    //Creates a new instance of the pool from the pair address
    pub async fn new_from_address<M: Middleware>(
        pair_address: H160,
        middlewear: Arc<M>,
    ) -> Result<Self, CFFMError<M>> {
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
            tick_word: U256::zero(),
            fee: 0,
            liquidity_net: 0,
        };

        pool.get_pool_data(middlewear.clone()).await?;

        pool.sync_pool(middlewear).await?;

        Ok(pool)
    }

    pub async fn get_pool_data<M: Middleware>(
        &mut self,
        middlewear: Arc<M>,
    ) -> Result<(), CFFMError<M>> {
        self.token_a = self.get_token_0(middlewear.clone()).await?;
        self.token_b = self.get_token_1(middlewear.clone()).await?;
        (self.token_a_decimals, self.token_b_decimals) =
            self.get_token_decimals(middlewear.clone()).await?;
        self.fee = self.get_fee(middlewear.clone()).await?;
        self.tick_spacing = self.get_tick_spacing(middlewear.clone()).await?;
        Ok(())
    }

    pub async fn get_tick_word<M: Middleware>(
        &self,
        tick: i32,
        middlewear: Arc<M>,
    ) -> Result<U256, CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear);
        let (word_position, _) = uniswap_v3_math::tick_bit_map::position(tick);
        Ok(v3_pool.tick_bitmap(word_position).call().await?)
    }

    pub async fn get_next_word<M: Middleware>(
        &self,
        word_position: i16,
        middlewear: Arc<M>,
    ) -> Result<U256, CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear);
        Ok(v3_pool.tick_bitmap(word_position).call().await?)
    }

    pub async fn get_tick_spacing<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<i32, CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear);
        Ok(v3_pool.tick_spacing().call().await?)
    }

    pub async fn get_tick<M: Middleware>(&self, middlewear: Arc<M>) -> Result<i32, CFFMError<M>> {
        Ok(self.get_slot_0(middlewear).await?.1)
    }

    pub async fn get_tick_info<M: Middleware>(
        &self,
        tick: i32,
        middlewear: Arc<M>,
    ) -> Result<(u128, i128, U256, U256, i64, U256, u32, bool), CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear.clone());

        let tick_info = v3_pool.ticks(tick).call().await?;

        Ok((
            tick_info.0,
            tick_info.1,
            tick_info.2,
            tick_info.3,
            tick_info.4,
            tick_info.5,
            tick_info.6,
            tick_info.7,
        ))
    }

    pub async fn get_liquidity_net<M: Middleware>(
        &self,
        tick: i32,
        middlewear: Arc<M>,
    ) -> Result<i128, CFFMError<M>> {
        let tick_info = self.get_tick_info(tick, middlewear).await?;
        Ok(tick_info.1)
    }

    pub async fn get_initialized<M: Middleware>(
        &self,
        tick: i32,
        middlewear: Arc<M>,
    ) -> Result<bool, CFFMError<M>> {
        let tick_info = self.get_tick_info(tick, middlewear).await?;
        Ok(tick_info.7)
    }

    pub async fn get_slot_0<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<(U256, i32, u16, u16, u16, u8, bool), CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear);
        Ok(v3_pool.slot_0().call().await?)
    }

    pub async fn get_liquidity<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<u128, CFFMError<M>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, middlewear);
        Ok(v3_pool.liquidity().call().await?)
    }

    pub async fn get_sqrt_price<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<U256, CFFMError<M>> {
        Ok(self.get_slot_0(middlewear).await?.0)
    }

    pub async fn sync_pool<M: Middleware>(
        &mut self,
        middlewear: Arc<M>,
    ) -> Result<(), CFFMError<M>> {
        self.liquidity = self.get_liquidity(middlewear).await?;

        let slot_0 = self.get_slot_0(middlewear.clone()).await?;
        self.sqrt_price = slot_0.0;
        self.tick = slot_0.1;

        self.tick_word = self.get_tick_word(self.tick, middlewear.clone()).await?;
        self.liquidity_net = self.get_liquidity_net(self.tick, middlewear).await?;

        Ok(())
    }

    pub async fn update_pool_from_swap_log<M: Middleware>(
        &mut self,
        swap_log: &Log,
        middlewear: Arc<M>,
    ) -> Result<(), CFFMError<M>> {
        (_, _, self.sqrt_price, self.liquidity, self.tick) = self.decode_swap_log(swap_log);

        self.tick_word = self.get_tick_word(self.tick, middlewear.clone()).await?;
        self.liquidity_net = self.get_liquidity_net(self.tick, middlewear).await?;

        Ok(())
    }

    //Returns reserve0, reserve1
    pub fn decode_swap_log(&self, swap_log: &Log) -> (I256, I256, U256, u128, i32) {
        let log_data = decode(
            &[
                ParamType::Int(256),  //amount0
                ParamType::Int(256),  //amount1
                ParamType::Uint(160), //sqrtPriceX96
                ParamType::Uint(128), //liquidity
                ParamType::Int(24),
            ],
            &swap_log.data,
        )
        .expect("Could not get log data");

        let amount_0 = I256::from_raw(log_data[1].to_owned().into_int().unwrap());
        let amount_1 = I256::from_raw(log_data[1].to_owned().into_int().unwrap());
        let sqrt_price = log_data[2].to_owned().into_uint().unwrap();
        let liquidity = log_data[3].to_owned().into_uint().unwrap().as_u128();
        let tick = log_data[4].to_owned().into_uint().unwrap().as_u32() as i32;

        (amount_0, amount_1, sqrt_price, liquidity, tick)
    }

    pub async fn get_token_decimals<M: Middleware>(
        &mut self,
        middlewear: Arc<M>,
    ) -> Result<(u8, u8), CFFMError<M>> {
        let token_a_decimals = abi::IErc20::new(self.token_a, middlewear.clone())
            .decimals()
            .call()
            .await?;

        let token_b_decimals = abi::IErc20::new(self.token_b, middlewear)
            .decimals()
            .call()
            .await?;

        Ok((token_a_decimals, token_b_decimals))
    }

    pub async fn get_fee<M: Middleware>(
        &mut self,
        middlewear: Arc<M>,
    ) -> Result<u32, CFFMError<M>> {
        let fee = abi::IUniswapV3Pool::new(self.address, middlewear)
            .fee()
            .call()
            .await?;

        Ok(fee)
    }

    pub async fn get_token_0<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<H160, CFFMError<M>> {
        let v2_pair = abi::IUniswapV2Pair::new(self.address, middlewear);

        let token0 = match v2_pair.token_0().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(CFFMError::ContractError(contract_error)),
        };

        Ok(token0)
    }

    pub async fn get_token_1<M: Middleware>(
        &self,
        middlewear: Arc<M>,
    ) -> Result<H160, CFFMError<M>> {
        let v2_pair = abi::IUniswapV2Pair::new(self.address, middlewear);

        let token1 = match v2_pair.token_1().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(CFFMError::ContractError(contract_error)),
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

    pub async fn simulate_swap<M: Middleware>(
        &self,
        token_in: H160,
        amount_in: U256,
        middlewear: Arc<M>,
    ) -> Result<U256, CFFMError<M>> {
        //Initialize zero_for_one to true if token_in is token_a
        let zero_for_one = token_in == self.token_a;

        //Set sqrt_price_limit_x_96 to the max or min sqrt price in the pool depending on zero_for_one
        let sqrt_price_limit_x_96 = if zero_for_one {
            U256::from("4295128739") + 1
        } else {
            U256::from("0xFFFD8963EFD1FC6A506488495D951D5263988D26") - 1
        };

        //Initialize a mutable state state struct to hold the dynamic simulated state of the pool
        let mut current_state = CurrentState {
            sqrt_price_x_96: self.sqrt_price, //Active price on the pool
            amount_calculated: I256::zero(),  //Amount of token_out that has been calculated
            amount_specified_remaining: I256::from_raw(amount_in), //Amount of token_in that has not been swapped
            tick: self.tick,                                       //Current i24 tick of the pool
            liquidity: self.liquidity, //Current available liquidity in the tick range
        };

        let pool = abi::IUniswapV3Pool::new(self.address, middlewear.clone());

        //Get the first initialized tick within one word of the current tick
        let mut tick_word = pool
            .tick_bitmap(uniswap_v3_math::tick_bit_map::position(current_state.tick).0)
            .call()
            .await?;

        while current_state.amount_specified_remaining > I256::zero() {
            //Initialize a new step struct to hold the dynamic state of the pool at each step
            let mut step = StepComputations::default();

            //Get the next initialized tick within one word of the current tick
            (step.tick_next, step.initialized) =
                uniswap_v3_math::tick_bit_map::next_initialized_tick_within_one_word(
                    tick_word,
                    current_state.tick,
                    self.tick_spacing,
                    zero_for_one,
                );

            //Set the sqrt_price_start_x_96 to the current sqrt_price_x_96
            step.sqrt_price_start_x_96 = current_state.sqrt_price_x_96;

            //If there are no initialized ticks within the current word, then we need to get the next word, at word_position + 1
            if !step.initialized {
                let (word_position, _) = uniswap_v3_math::tick_bit_map::position(self.tick);
                tick_word = self
                    .get_next_word(word_position, middlewear.clone())
                    .await?;

                (step.tick_next, step.initialized) =
                    uniswap_v3_math::tick_bit_map::next_initialized_tick_within_one_word(
                        tick_word,
                        current_state.tick,
                        self.tick_spacing,
                        zero_for_one,
                    );
            }

            //Get the next sqrt price from the input amount
            step.sqrt_price_next_x96 =
                uniswap_v3_math::tick_math::get_sqrt_ratio_at_tick(step.tick_next)?;

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            step.tick_next = step.tick_next.clamp(MIN_TICK, MAX_TICK);

            //Target spot price
            let swap_target_sqrt_ratio = if zero_for_one {
                if step.sqrt_price_next_x96 < sqrt_price_limit_x_96 {
                    sqrt_price_limit_x_96
                } else {
                    step.sqrt_price_next_x96
                }
            } else if step.sqrt_price_next_x96 > sqrt_price_limit_x_96 {
                step.sqrt_price_next_x96
            } else {
                sqrt_price_limit_x_96
            };

            //Compute swap step and update the current state
            (
                current_state.sqrt_price_x_96,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = uniswap_v3_math::swap_math::compute_swap_step(
                current_state.sqrt_price_x_96,
                swap_target_sqrt_ratio,
                current_state.liquidity,
                current_state.amount_specified_remaining,
                self.fee,
            )?;

            //Decrement the amount remaining to be swapped and amount received from the step
            current_state.amount_specified_remaining -=
                I256::from_raw(step.amount_in.add(step.fee_amount));
            current_state.amount_calculated -= I256::from_raw(step.amount_out);

            //If the price moved all the way to the next price, recompute the liquidity change for the next iteration
            if current_state.sqrt_price_x_96 == step.sqrt_price_next_x96 {
                if step.initialized {
                    let mut liquidity_net = self
                        .get_liquidity_net(step.tick_next, middlewear.clone())
                        .await?;

                    // we are on a tick boundary, and the next tick is initialized, so we must charge a protocol fee
                    if zero_for_one {
                        liquidity_net = -liquidity_net;
                    }

                    current_state.liquidity = uniswap_v3_math::liquidity_math::add_delta(
                        current_state.liquidity,
                        liquidity_net,
                    )?;
                }
                //Increment the current tick
                current_state.tick = if zero_for_one {
                    step.tick_next - 1
                } else {
                    step.tick_next
                }
                //If the current_state sqrt price is not equal to the step sqrt price, then we are not on the same tick.
                //Update the current_state.tick to the tick at the current_state.sqrt_price_x_96
            } else if current_state.sqrt_price_x_96 != step.sqrt_price_start_x_96 {
                current_state.tick = uniswap_v3_math::sqrt_price_math::get_tick_at_sqrt_ratio(
                    current_state.sqrt_price_x_96,
                )?;
            }
        }

        Ok(U256::from(
            (-current_state.amount_calculated.as_i128()) as u128,
        ))
    }

    pub async fn simulate_swap_mut<M: Middleware>(
        &mut self,
        token_in: H160,
        amount_in: U256,
        middlewear: Arc<M>,
    ) -> Result<U256, CFFMError<M>> {
        //Initialize zero_for_one to true if token_in is token_a
        let zero_for_one = token_in == self.token_a;

        //Set sqrt_price_limit_x_96 to the max or min sqrt price in the pool depending on zero_for_one
        let sqrt_price_limit_x_96 = if zero_for_one {
            U256::from("4295128739") + 1
        } else {
            U256::from("0xFFFD8963EFD1FC6A506488495D951D5263988D26") - 1
        };

        //Initialize a mutable state state struct to hold the dynamic simulated state of the pool
        let mut current_state = CurrentState {
            sqrt_price_x_96: self.sqrt_price, //Active price on the pool
            amount_calculated: I256::zero(),  //Amount of token_out that has been calculated
            amount_specified_remaining: I256::from_raw(amount_in), //Amount of token_in that has not been swapped
            tick: self.tick,                                       //Current i24 tick of the pool
            liquidity: self.liquidity, //Current available liquidity in the tick range
        };

        let pool = abi::IUniswapV3Pool::new(self.address, middlewear.clone());

        //Get the first initialized tick within one word of the current tick
        let mut tick_word = pool
            .tick_bitmap(uniswap_v3_math::tick_bit_map::position(current_state.tick).0)
            .call()
            .await?;

        let mut liquidity_net = self.liquidity_net;

        while current_state.amount_specified_remaining > I256::zero() {
            //Initialize a new step struct to hold the dynamic state of the pool at each step
            let mut step = StepComputations {
                sqrt_price_start_x_96: current_state.sqrt_price_x_96,
                ..Default::default()
            };

            //Get the next initialized tick within one word of the current tick
            (step.tick_next, step.initialized) =
                uniswap_v3_math::tick_bit_map::next_initialized_tick_within_one_word(
                    tick_word,
                    current_state.tick,
                    self.tick_spacing,
                    zero_for_one,
                );
            //If there are no initialized ticks within the current word, then we need to get the next word, at word_position + 1
            if !step.initialized {
                let (word_position, _) = uniswap_v3_math::tick_bit_map::position(self.tick);
                tick_word = self
                    .get_next_word(word_position, middlewear.clone())
                    .await?;

                (step.tick_next, step.initialized) =
                    uniswap_v3_math::tick_bit_map::next_initialized_tick_within_one_word(
                        tick_word,
                        current_state.tick,
                        self.tick_spacing,
                        zero_for_one,
                    );
            }

            //Get the next sqrt price from the input amount
            step.sqrt_price_next_x96 =
                uniswap_v3_math::tick_math::get_sqrt_ratio_at_tick(step.tick_next)?;

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds

            step.tick_next = step.tick_next.clamp(MIN_TICK, MAX_TICK);

            //Target spot price
            let swap_target_sqrt_ratio = if zero_for_one {
                if step.sqrt_price_next_x96 < sqrt_price_limit_x_96 {
                    sqrt_price_limit_x_96
                } else {
                    step.sqrt_price_next_x96
                }
            } else if step.sqrt_price_next_x96 > sqrt_price_limit_x_96 {
                step.sqrt_price_next_x96
            } else {
                sqrt_price_limit_x_96
            };

            //Compute swap step and update the current state
            (
                current_state.sqrt_price_x_96,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = uniswap_v3_math::swap_math::compute_swap_step(
                current_state.sqrt_price_x_96,
                swap_target_sqrt_ratio,
                current_state.liquidity,
                current_state.amount_specified_remaining,
                self.fee,
            )?;

            //Decrement the amount remaining to be swapped and amount received from the step
            current_state.amount_specified_remaining -=
                I256::from_raw(step.amount_in.add(step.fee_amount));
            current_state.amount_calculated -= I256::from_raw(step.amount_out);

            //If the price moved all the way to the next price, recompute the liquidity change for the next iteration
            if current_state.sqrt_price_x_96 == step.sqrt_price_next_x96 {
                if step.initialized {
                    liquidity_net = self
                        .get_liquidity_net(step.tick_next, middlewear.clone())
                        .await?;

                    // we are on a tick boundary, and the next tick is initialized, so we must charge a protocol fee
                    if zero_for_one {
                        liquidity_net = -liquidity_net;
                    }

                    current_state.liquidity = uniswap_v3_math::liquidity_math::add_delta(
                        current_state.liquidity,
                        liquidity_net,
                    )?;
                }
                //Increment the current tick
                current_state.tick = if zero_for_one {
                    step.tick_next - 1
                } else {
                    step.tick_next
                }
                //If the current_state sqrt price is not equal to the step sqrt price, then we are not on the same tick.
                //Update the current_state.tick to the tick at the current_state.sqrt_price_x_96
            } else if current_state.sqrt_price_x_96 != step.sqrt_price_start_x_96 {
                current_state.tick = uniswap_v3_math::sqrt_price_math::get_tick_at_sqrt_ratio(
                    current_state.sqrt_price_x_96,
                )?;
            }
        }

        //Update the pool state
        self.liquidity = current_state.liquidity;
        self.sqrt_price = current_state.sqrt_price_x_96;
        self.tick = current_state.tick;
        self.liquidity_net = liquidity_net;
        self.tick_word = tick_word;

        Ok(U256::from(
            (-current_state.amount_calculated.as_i128()) as u128,
        ))
    }
}

pub struct CurrentState {
    amount_specified_remaining: I256,
    amount_calculated: I256,
    sqrt_price_x_96: U256,
    tick: i32,
    liquidity: u128,
}

#[derive(Default)]
pub struct StepComputations {
    pub sqrt_price_start_x_96: U256,
    pub tick_next: i32,
    pub initialized: bool,
    pub sqrt_price_next_x96: U256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub fee_amount: U256,
}

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
