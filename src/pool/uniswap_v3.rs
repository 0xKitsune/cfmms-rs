use std::sync::Arc;

use ethers::{
    providers::{JsonRpcClient, Provider},
    types::{H160, U256},
};
use num_bigfloat::BigFloat;

use crate::{abi, error::PairSyncError};

use super::{convert_to_common_decimals, convert_to_decimals};

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

    pub async fn simulate_swap<P: 'static + JsonRpcClient>(
        &self,
        token_in: H160,
        amount_in: u128,
        v3_quoter_address: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<U256, PairSyncError<P>> {
        let v3_quoter = abi::IUniswapV3Quoter::new(v3_quoter_address, provider);

        if self.token_a == token_in {
            if self.a_to_b {
                Ok(v3_quoter
                    .quote_exact_input_single(
                        self.token_a,
                        self.token_b,
                        self.fee,
                        U256::from(amount_in),
                        U256::zero(),
                    )
                    .call()
                    .await?)
            } else {
                Ok(v3_quoter
                    .quote_exact_input_single(
                        self.token_b,
                        self.token_a,
                        self.fee,
                        U256::from(amount_in),
                        U256::zero(),
                    )
                    .call()
                    .await?)
            }
        } else if self.a_to_b {
            Ok(v3_quoter
                .quote_exact_input_single(
                    self.token_b,
                    self.token_a,
                    self.fee,
                    U256::from(amount_in),
                    U256::zero(),
                )
                .call()
                .await?)
        } else {
            Ok(v3_quoter
                .quote_exact_input_single(
                    self.token_a,
                    self.token_b,
                    self.fee,
                    U256::from(amount_in),
                    U256::zero(),
                )
                .call()
                .await?)
        }
    }

    fn get_sqrt_ratio_at_tick() {}

    fn next_initialized_tick_within_one_word() {}

    fn cross() {}

    fn compute_swap_step() {}

    fn get_next_sqrt_price_from_input() {}

    fn get_amount_0_delta() {}

    fn get_amount_1_delta() {}

    fn mul_div() {}

    fn add_delta() {}
}
