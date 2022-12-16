use std::sync::Arc;

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{Log, H160, U256},
};

use crate::{abi, error::CFMMError};

use super::{convert_to_common_decimals, convert_to_decimals};

#[derive(Debug, Clone, Copy, Default)]
pub struct UniswapV2Pool {
    pub address: H160,
    pub token_a: H160,
    pub token_a_decimals: u8,
    pub token_b: H160,
    pub token_b_decimals: u8,
    pub reserve_0: u128,
    pub reserve_1: u128,
    pub fee: u32,
}

impl UniswapV2Pool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: H160,
        token_a: H160,
        token_a_decimals: u8,
        token_b: H160,
        token_b_decimals: u8,
        reserve_0: u128,
        reserve_1: u128,
        fee: u32,
    ) -> UniswapV2Pool {
        UniswapV2Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            reserve_0,
            reserve_1,
            fee,
        }
    }

    //Creates a new instance of the pool from the pair address, and syncs the pool data
    pub async fn new_from_address<M: Middleware>(
        pair_address: H160,
        middleware: Arc<M>,
    ) -> Result<Self, CFMMError<M>> {
        let mut pool = UniswapV2Pool {
            address: pair_address,
            token_a: H160::zero(),
            token_a_decimals: 0,
            token_b: H160::zero(),
            token_b_decimals: 0,
            reserve_0: 0,
            reserve_1: 0,
            fee: 300,
        };

        pool.get_pool_data(middleware.clone()).await?;
        pool.sync_pool(middleware).await?;

        Ok(pool)
    }

    pub async fn get_pool_data<M: Middleware>(
        &mut self,
        middleware: Arc<M>,
    ) -> Result<(), CFMMError<M>> {
        self.token_a = self.get_token_0(self.address, middleware.clone()).await?;
        self.token_b = self.get_token_1(self.address, middleware.clone()).await?;

        (self.token_a_decimals, self.token_b_decimals) =
            self.get_token_decimals(middleware).await?;

        Ok(())
    }

    pub async fn get_reserves<M: Middleware>(
        &self,
        middleware: Arc<M>,
    ) -> Result<(u128, u128), CFMMError<M>> {
        //Initialize a new instance of the Pool
        let v2_pair = abi::IUniswapV2Pair::new(self.address, middleware);

        // Make a call to get the reserves
        let (reserve_0, reserve_1, _) = match v2_pair.get_reserves().call().await {
            Ok(result) => result,

            Err(contract_error) => return Err(CFMMError::ContractError(contract_error)),
        };

        Ok((reserve_0, reserve_1))
    }

    pub async fn sync_pool<M: Middleware>(
        &mut self,
        middleware: Arc<M>,
    ) -> Result<(), CFMMError<M>> {
        (self.reserve_0, self.reserve_1) = self.get_reserves(middleware).await?;

        Ok(())
    }

    pub async fn get_token_decimals<M: Middleware>(
        &mut self,
        middleware: Arc<M>,
    ) -> Result<(u8, u8), CFMMError<M>> {
        let token_a_decimals = abi::IErc20::new(self.token_a, middleware.clone())
            .decimals()
            .call()
            .await?;

        let token_b_decimals = abi::IErc20::new(self.token_b, middleware)
            .decimals()
            .call()
            .await?;

        Ok((token_a_decimals, token_b_decimals))
    }

    pub async fn get_token_0<M: Middleware>(
        &self,
        pair_address: H160,
        middleware: Arc<M>,
    ) -> Result<H160, CFMMError<M>> {
        let v2_pair = abi::IUniswapV2Pair::new(pair_address, middleware);

        let token0 = match v2_pair.token_0().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(CFMMError::ContractError(contract_error)),
        };

        Ok(token0)
    }

    pub async fn get_token_1<M: Middleware>(
        &self,
        pair_address: H160,
        middleware: Arc<M>,
    ) -> Result<H160, CFMMError<M>> {
        let v2_pair = abi::IUniswapV2Pair::new(pair_address, middleware);

        let token1 = match v2_pair.token_1().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(CFMMError::ContractError(contract_error)),
        };

        Ok(token1)
    }

    pub fn calculate_price(&self, base_token: H160) -> f64 {
        let reserve_0 = self.reserve_0 as f64 / 10f64.powf(self.token_a_decimals.into());
        let reserve_1 = self.reserve_1 as f64 / 10f64.powf(self.token_b_decimals.into());

        if base_token == self.token_a {
            reserve_0 / reserve_1
        } else {
            reserve_1 / reserve_0
        }
    }

    pub fn address(&self) -> H160 {
        self.address
    }

    pub fn update_pool_from_sync_log(&mut self, sync_log: &Log) {
        (self.reserve_0, self.reserve_1) = self.decode_sync_log(sync_log);
    }

    //Returns reserve0, reserve1
    pub fn decode_sync_log(&self, sync_log: &Log) -> (u128, u128) {
        let data = ethers::abi::decode(
            &[
                ParamType::Uint(128), //reserve0
                ParamType::Uint(128),
            ],
            &sync_log.data,
        )
        .expect("Could not get log data");

        (
            data[0]
                .to_owned()
                .into_uint()
                .expect("Could not convert reserve0 in to uint")
                .as_u128(),
            data[1]
                .to_owned()
                .into_uint()
                .expect("Could not convert reserve1 in to uint")
                .as_u128(),
        )
    }

    pub fn simulate_swap(&self, token_in: H160, amount_in: U256) -> U256 {
        let (reserve_0, reserve_1, common_decimals) = convert_to_common_decimals(
            U256::from(self.reserve_0),
            self.token_a_decimals,
            U256::from(self.reserve_1),
            self.token_b_decimals,
        );

        //Apply fee on amount in
        //Fee will always be .3% for Univ2
        let amount_in = amount_in * U256::from(997) / U256::from(1000);

        // x * y = k
        // (x + ∆x) * (y - ∆y) = k
        // y - (k/(x + ∆x)) = ∆y
        let k = reserve_0 * reserve_1;

        if self.token_a == token_in {
            convert_to_decimals(
                reserve_1 - k / (reserve_0 + amount_in),
                common_decimals,
                self.token_b_decimals,
            )
        } else {
            convert_to_decimals(
                reserve_0 - k / (reserve_1 + amount_in),
                common_decimals,
                self.token_a_decimals,
            )
        }
    }

    pub fn simulate_swap_mut(&mut self, token_in: H160, amount_in: U256) -> U256 {
        let (reserve_0, reserve_1, common_decimals) = convert_to_common_decimals(
            U256::from(self.reserve_0),
            self.token_a_decimals,
            U256::from(self.reserve_1),
            self.token_b_decimals,
        );

        //Apply fee on amount in
        //Fee will always be .3% for Univ2
        let amount_in = amount_in * U256::from(997) / U256::from(1000);

        // x * y = k
        // (x + ∆x) * (y - ∆y) = k
        // y - (k/(x + ∆x)) = ∆y
        let k = reserve_0 * reserve_1;

        if self.token_a == token_in {
            let amount_out = convert_to_decimals(
                reserve_1 - k / (reserve_0 + amount_in),
                common_decimals,
                self.token_b_decimals,
            );

            self.reserve_0 -= amount_in.as_u128();
            self.reserve_1 += amount_out.as_u128();
            amount_out
        } else {
            let amount_out = convert_to_decimals(
                reserve_0 - k / (reserve_1 + amount_in),
                common_decimals,
                self.token_a_decimals,
            );

            self.reserve_0 += amount_out.as_u128();
            self.reserve_1 -= amount_in.as_u128();
            amount_out
        }
    }
}
