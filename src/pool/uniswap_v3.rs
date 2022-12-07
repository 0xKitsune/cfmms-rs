use std::{
    collections::HashMap,
    ops::{BitAnd, Shr},
    str::FromStr,
    sync::Arc,
};

use ethers::{
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
    pub a_to_b: bool,
    pub liquidity: u128,
    pub sqrt_price: U256,
    pub fee: u32,
}

impl UniswapV3Pool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: H160,
        token_a: H160,
        token_a_decimals: u8,
        token_b: H160,
        token_b_decimals: u8,
        a_to_b: bool,
        liquidity: u128,
        sqrt_price: U256,
        fee: u32,
    ) -> UniswapV3Pool {
        UniswapV3Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            a_to_b,
            liquidity,
            sqrt_price,
            fee,
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
            a_to_b: false,
            liquidity: 0,
            sqrt_price: U256::zero(),
            fee: 300,
        };

        pool.token_a = pool.get_token_0(provider.clone()).await?;
        pool.token_b = pool.get_token_1(provider.clone()).await?;
        pool.a_to_b = true;

        pool.fee = pool.get_fee(provider.clone()).await?;

        (pool.token_a_decimals, pool.token_b_decimals) =
            pool.get_token_decimals(provider.clone()).await?;

        (pool.liquidity, pool.sqrt_price) =
            pool.get_liquidity_and_sqrt_price(provider.clone()).await?;

        Ok(pool)
    }

    pub async fn get_pool_data<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        self.token_a = self.get_token_0(provider.clone()).await?;
        self.token_b = self.get_token_1(provider.clone()).await?;
        self.a_to_b = true;

        (self.token_a_decimals, self.token_b_decimals) =
            self.get_token_decimals(provider.clone()).await?;

        Ok(())
    }

    pub async fn get_liquidity_and_sqrt_price<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<(u128, U256), PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());

        //Sqrt price is stored as a Q64.96 so we need to left shift the liquidity by 96 to be represented as Q64.96
        //We cant right shift sqrt_price because it could move the value to 0, making divison by 0 to get reserve_x
        let liquidity = &v3_pool.liquidity().call().await?;
        let sqrt_price = v3_pool.slot_0().call().await?.0;

        Ok((*liquidity, sqrt_price))
    }

    pub async fn sync_pool<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        (self.liquidity, self.sqrt_price) =
            self.get_liquidity_and_sqrt_price(provider.clone()).await?;

        (self.liquidity, self.sqrt_price) = self.get_liquidity_and_sqrt_price(provider).await?;

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

        if self.a_to_b {
            if self.token_a == base_token {
                price.to_f64()
            } else {
                1.0 / price.to_f64()
            }
        } else if self.token_a == base_token {
            1.0 / price.to_f64()
        } else {
            price.to_f64()
        }
    }

    pub fn address(&self) -> H160 {
        self.address
    }

    pub fn simulate_swap(&self, token_in: H160, amount_in: u128) -> u128 {
        //TODO: update this
        0
    }

    pub fn simulate_swap_mut(&mut self, token_in: H160, amount_in: u128) -> u128 {
        //TODO: update this
        0
    }
}

//Univ3 swap simulation logic

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

pub fn get_sqrt_ratio_at_tick(tick: i32) -> U256 {
    let abs_tick = if tick < 0 {
        let le_bytes = &mut [0u8; 32];
        (-I256::from(tick)).to_little_endian(le_bytes);
        U256::from_little_endian(le_bytes)
    } else {
        U256::from(tick)
    };

    if abs_tick > U256::from(MAX_TICK) {
        //TODO: create uniswap v3 simulation error,
        //revert T, maybe add more descriptive errors
    }

    let mut ratio = if abs_tick.bitand(U256::from(0x1)) != U256::zero() {
        U256::from("0xfffcb933bd6fad37aa2d162d1a594001")
    } else {
        U256::from("0x100000000000000000000000000000000")
    };

    ratio = if !abs_tick.bitand(U256::from(0x2)).is_zero() {
        (ratio * U256::from("0xfff97272373d413259a46990580e213a")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x4)).is_zero() {
        (ratio * U256::from("0xfff2e50f5f656932ef12357cf3c7fdcc")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x8)).is_zero() {
        (ratio * U256::from("0xffe5caca7e10e4e61c3624eaa0941cd0")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x10)).is_zero() {
        (ratio * U256::from("0xffcb9843d60f6159c9db58835c926644")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x20)).is_zero() {
        (ratio * U256::from("0xff973b41fa98c081472e6896dfb254c0")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x40)).is_zero() {
        (ratio * U256::from("0xff2ea16466c96a3843ec78b326b52861")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x80)).is_zero() {
        (ratio * U256::from("0xfe5dee046a99a2a811c461f1969c3053")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x100)).is_zero() {
        (ratio * U256::from("0xfcbe86c7900a88aedcffc83b479aa3a4")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x200)).is_zero() {
        (ratio * U256::from("0xf987a7253ac413176f2b074cf7815e54")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x400)).is_zero() {
        (ratio * U256::from("0xf3392b0822b70005940c7a398e4b70f3")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x800)).is_zero() {
        (ratio * U256::from("0xe7159475a2c29b7443b29c7fa6e889d9")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x1000)).is_zero() {
        (ratio * U256::from("0xd097f3bdfd2022b8845ad8f792aa5825")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x2000)).is_zero() {
        (ratio * U256::from("0xa9f746462d870fdf8a65dc1f90e061e5")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x4000)).is_zero() {
        (ratio * U256::from("0x70d869a156d2a1b890bb3df62baf32f7")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x8000)).is_zero() {
        (ratio * U256::from("0x31be135f97d08fd981231505542fcfa6")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x10000)).is_zero() {
        (ratio * U256::from("0x9aa508b5b7a84e1c677de54f3e99bc9")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x20000)).is_zero() {
        (ratio * U256::from("0x5d6af8dedb81196699c329225ee604")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x40000)).is_zero() {
        (ratio * U256::from("0x2216e584f5fa1ea926041bedfe98")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x80000)).is_zero() {
        (ratio * U256::from("0x48a170391f7dc42444e8fa2")).shr(128)
    } else {
        ratio
    };

    if tick > 0 {
        ratio = U256::MAX / ratio;
    }

    if (ratio.shr(32) + (ratio % (1 << 32))).is_zero() {
        U256::zero()
    } else {
        U256::one()
    }
}

//Returns next and initialized
pub fn next_initialized_tick_within_one_word(
    tick_mapping: HashMap<i16, U256>,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> (i32, bool) {
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = U256::from((1 << bit_pos) - 1 + (1 << bit_pos));
        let masked = tick_mapping.get(&word_pos).unwrap().bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let le_bytes = &mut [0u8; 32];
            masked.to_little_endian(le_bytes);
            let most_significant_bit = le_bytes[0];
            compressed - ((bit_pos - most_significant_bit) as i32 & tick_spacing)
        } else {
            compressed - (bit_pos as i32 * tick_spacing)
        };

        (next, initialized)
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = !U256::from((1 << bit_pos) - 1);

        let masked = tick_mapping.get(&word_pos).unwrap().bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let le_bytes = &mut [0u8; 32];
            masked.to_big_endian(le_bytes);
            let least_significant_bit = le_bytes[0];
            (compressed + 1 + (least_significant_bit - bit_pos) as i32) * tick_spacing
        } else {
            (compressed + 1 + 0xFF - bit_pos as i32) * tick_spacing
        };

        (next, initialized)
    }
}

// returns (int16 wordPos, uint8 bitPos)
fn position(tick: i32) -> (i16, u8) {
    (tick.shr(8) as i16, (tick % 256) as u8)
}

fn cross(tick_mapping: HashMap<i16, U256>, tick: i32) -> i128 {
    //TODO: update this
    0
}

// //returns (
//         uint160 sqrtRatioNextX96,
//         uint256 amountIn,
//         uint256 amountOut,
//         uint256 feeAmount
//     )
fn compute_swap_step(
    sqrt_ratio_current_x_96: U256,
    sqrt_ratio_target_x_96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> (U256, U256, U256, U256) {
    //TODO: update this
    (U256::zero(), U256::zero(), U256::zero(), U256::zero())
}

// returns (uint160 sqrtQX96)
fn get_next_sqrt_price_from_input(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> (U256) {
    //TODO: update this
    U256::zero()
}

// returns (uint160 sqrtQX96)
fn get_next_sqrt_price_From_amount_0_rounding_up(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint160 sqrtQX96)
fn get_next_sqrt_price_From_amount_1_rounding_down(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint256 amount0)
fn get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint256 amount1)
fn get_amount_1_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint256 result)
fn mul_div(a: U256, b: U256, denominator: U256) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint128 z)
fn add_delta(x: u128, y: i128) -> u128 {
    if y < 0 {
        let z = x - (-y as u128);

        if z < x {
            //TODO: revert "LS" error
            0 // right now just zero to avoid linting error
        } else {
            z
        }
    } else {
        let z = x - (y as u128);
        if z >= x {
            //TODO: revert "LA" error
            0 // right now just zero to avoid linting error
        } else {
            z
        }
    }
}
