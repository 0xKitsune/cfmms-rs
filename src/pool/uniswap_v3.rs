use std::{
    collections::HashMap,
    ops::{BitAnd, Shr},
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
    pub a_to_b: bool,
    pub liquidity: u128,
    pub sqrt_price: U256,
    pub tick: i32,
    pub tick_spacing: i32,
    pub liquidity_net: i32,
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
        tick: i32,
        tick_spacing: i32,
        liquidity_net: i32,
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
            tick,
            tick_spacing,
            liquidity_net,
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
            tick: 0,
            tick_spacing: 0,
            liquidity_net: 0,
            fee: 300,
        };

        (pool.token_a_decimals, pool.token_b_decimals) =
            pool.get_token_decimals(provider.clone()).await?;
        pool.token_a = pool.get_token_0(provider.clone()).await?;
        pool.token_b = pool.get_token_1(provider.clone()).await?;
        pool.a_to_b = true;
        pool.fee = pool.get_fee(provider.clone()).await?;
        pool.tick_spacing = pool.get_tick_spacing(provider.clone()).await?;
        pool.liquidity = pool.get_liquidity(provider.clone()).await?;

        let slot_0 = pool.get_slot_0(provider.clone()).await?;
        pool.tick = slot_0.1;
        pool.sqrt_price = slot_0.0;

        pool.liquidity_net = pool.get_liquidity_net(pool.tick, provider.clone()).await?;

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

        self.fee = self.get_fee(provider.clone()).await?;
        self.tick_spacing = self.get_tick_spacing(provider.clone()).await?;

        Ok(())
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
    ) -> Result<i32, PairSyncError<P>> {
        let v3_pool = abi::IUniswapV3Pool::new(self.address, provider.clone());
        let tick_info_bytes = v3_pool.ticks(tick).call().await?;

        Ok(0)
        //TODO:
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

    //TODO: add decode swap log data, make it an associated function as well as a method?

    //TODO: Update pool from swap log method

    pub async fn simulate_swap<P: 'static + JsonRpcClient>(
        &self,
        token_in: H160,
        amount_in: u128,
        provider: Arc<Provider<P>>,
    ) -> Result<u128, PairSyncError<P>> {
        let zero_for_one = token_in == self.token_a;

        let initial_tick = abi::IUniswapV3Pool::new(self.address, provider)
            .slot_0()
            .call()
            .await?
            .1;

        //TODO: update this
        Ok(0)
    }

    pub fn simulate_swap_mut(&mut self, token_in: H160, amount_in: u128) -> u128 {
        //TODO: update this
        0
    }
}
