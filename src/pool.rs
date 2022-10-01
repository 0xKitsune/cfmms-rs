use std::{ops::Shr, str::FromStr, sync::Arc};

use crate::{abi, error::PairSyncError};
use ethers::{
    providers::{JsonRpcClient, Provider},
    types::{H160, H256, U256},
};

#[derive(Debug)]
pub struct Pool {
    pub address: H160,
    pub token_a: H160,
    pub token_a_decimals: u8,
    pub token_b: H160,
    pub token_b_decimals: u8,
    pub a_to_b: bool,
    pub reserve_0: u128,
    pub reserve_1: u128,
    pub fee: u32,
    pub pool_variant: PoolVariant,
}

#[derive(Debug, Clone, Copy)]
pub enum PoolVariant {
    UniswapV2,
    UniswapV3,
}

impl Pool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: H160,
        token_a: H160,
        token_a_decimals: u8,
        token_b: H160,
        token_b_decimals: u8,
        a_to_b: bool,
        reserve_0: u128,
        reserve_1: u128,
        fee: u32,
        pool_variant: PoolVariant,
    ) -> Pool {
        Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            a_to_b,
            reserve_0,
            reserve_1,
            fee,
            pool_variant,
        }
    }

    pub fn empty_pool(pool_variant: PoolVariant) -> Pool {
        Pool {
            address: H160::zero(),
            token_a: H160::zero(),
            token_a_decimals: 0,
            token_b: H160::zero(),
            token_b_decimals: 0,
            a_to_b: false,
            reserve_0: 0,
            reserve_1: 0,
            fee: 0,
            pool_variant,
        }
    }

    pub async fn new_pool_from_address<P: 'static + JsonRpcClient>(
        pair_address: H160,
        fee: u32,
        pool_variant: PoolVariant,
        provider: Arc<Provider<P>>,
    ) -> Result<Pool, PairSyncError<P>> {
        let mut pool = Pool {
            address: pair_address,
            token_a: H160::zero(),
            token_a_decimals: 0,
            token_b: H160::zero(),
            token_b_decimals: 0,
            a_to_b: false,
            reserve_0: 0,
            reserve_1: 0,
            fee,
            pool_variant,
        };

        pool.token_a = pool_variant
            .get_token_0(pair_address, provider.clone())
            .await?;
        pool.token_b = pool_variant
            .get_token_1(pair_address, provider.clone())
            .await?;

        pool.update_token_decimals(provider.clone()).await?;
        pool.update_a_to_b(provider.clone()).await?;
        pool.update_reserves(provider).await?;

        Ok(pool)
    }

    pub fn is_empty(&self) -> bool {
        self.token_a == H160::zero()
    }

    pub fn reserves_are_zero(&self) -> bool {
        self.reserve_0 == 0 && self.reserve_1 == 0
    }

    pub async fn get_reserves<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<(u128, u128), PairSyncError<P>>
where {
        self.pool_variant.get_reserves(self.address, provider).await
    }

    pub async fn update_reserves<P: JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        let (reserve0, reserve1) = self
            .pool_variant
            .get_reserves(self.address, provider)
            .await?;

        self.reserve_0 = reserve0;
        self.reserve_1 = reserve1;

        Ok(())
    }

    pub async fn get_token_0<P: JsonRpcClient>(
        &self,
        provider: Arc<Provider<P>>,
    ) -> Result<H160, PairSyncError<P>> {
        self.pool_variant.get_token_0(self.address, provider).await
    }

    pub async fn update_a_to_b<P: JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        let token0 = self
            .pool_variant
            .get_token_0(self.address, provider)
            .await?;

        self.a_to_b = token0 == self.token_a;

        Ok(())
    }

    pub async fn get_price<P>(
        &self,
        a_per_b: bool,
        provider: Arc<Provider<P>>,
    ) -> Result<f64, PairSyncError<P>>
    where
        P: JsonRpcClient,
    {
        let (reserve_0, reserve_1) = self.get_reserves(provider.clone()).await?;

        let reserve_0 = (reserve_0 * 10u128.pow(self.token_a_decimals.into())) as f64;
        let reserve_1 = (reserve_1 * 10u128.pow(self.token_b_decimals.into())) as f64;

        match self.pool_variant {
            PoolVariant::UniswapV2 => {
                if self.a_to_b {
                    if a_per_b {
                        Ok(reserve_0 / reserve_1)
                    } else {
                        Ok(reserve_1 / reserve_0)
                    }
                } else if a_per_b {
                    Ok(reserve_1 / reserve_0)
                } else {
                    Ok(reserve_0 / reserve_1)
                }
            }

            PoolVariant::UniswapV3 => {
                //TODO: double check this
                if self.a_to_b {
                    if a_per_b {
                        Ok(reserve_0 / reserve_1)
                    } else {
                        Ok(reserve_1 / reserve_0)
                    }
                } else if a_per_b {
                    Ok(reserve_1 / reserve_0)
                } else {
                    Ok(reserve_0 / reserve_1)
                }
            }
        }
    }

    pub async fn update_token_a<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        self.token_a_decimals = abi::IErc20::new(self.token_a, provider.clone())
            .decimals()
            .call()
            .await?;

        self.token_b_decimals = abi::IErc20::new(self.token_a, provider)
            .decimals()
            .call()
            .await?;

        Ok(())
    }

    pub async fn update_token_decimals<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), PairSyncError<P>> {
        self.token_a_decimals = abi::IErc20::new(self.token_a, provider.clone())
            .decimals()
            .call()
            .await?;

        self.token_b_decimals = abi::IErc20::new(self.token_a, provider)
            .decimals()
            .call()
            .await?;

        Ok(())
    }
}

impl PoolVariant {
    pub fn pool_created_event_signature(&self) -> H256 {
        match self {
            PoolVariant::UniswapV2 => {
                H256::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9")
                    .unwrap()
            }
            PoolVariant::UniswapV3 => {
                H256::from_str("0x783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118")
                    .unwrap()
            }
        }
    }

    pub async fn get_reserves<P: JsonRpcClient>(
        &self,
        pair_address: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<(u128, u128), PairSyncError<P>> {
        match self {
            PoolVariant::UniswapV2 => {
                //Initialize a new instance of the Pool
                let v2_pair = abi::IUniswapV2Pair::new(pair_address, provider);

                // Make a call to get the reserves
                let (reserve_0, reserve_1, _) = match v2_pair.get_reserves().call().await {
                    Ok(result) => result,

                    Err(contract_error) => {
                        return Err(PairSyncError::ContractError(contract_error))
                    }
                };

                Ok((reserve_0, reserve_1))
            }
            PoolVariant::UniswapV3 => {
                let v3_pool = abi::IUniswapV3Pool::new(pair_address, provider.clone());

                let liquidity =
                    U256::from_big_endian(&v3_pool.liquidity().call().await?.to_be_bytes());
                let slot_0 = v3_pool.slot_0().call().await?;

                //Sqrt price is stored as a Q64.96 so we need to shift the sqrt_price by 96 to be represented as just the integer numbers
                let sqrt_price = slot_0.0.shr(96);

                let (reserve_0, reserve_1) = if !sqrt_price.is_zero() {
                    let reserve_x = (liquidity / sqrt_price).as_u128();
                    let reserve_y = (liquidity * sqrt_price).as_u128();

                    (reserve_x, reserve_y)
                } else {
                    (0_u128, 0_u128)
                };

                Ok((reserve_0, reserve_1))
            }
        }
    }

    pub async fn get_token_0<P: JsonRpcClient>(
        &self,
        pair_address: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<H160, PairSyncError<P>> {
        match self {
            //Can match on v2 or v3 because they both have the same interface for token0, token1
            PoolVariant::UniswapV2 | PoolVariant::UniswapV3 => {
                //Initialize a new instance of the Pool
                let v2_pair = abi::IUniswapV2Pair::new(pair_address, provider);

                // Make a call to get token0 to initialize a_to_b
                let token0 = match v2_pair.token_0().call().await {
                    Ok(result) => result,
                    Err(contract_error) => {
                        return Err(PairSyncError::ContractError(contract_error))
                    }
                };
                Ok(token0)
            }
        }
    }

    pub async fn get_token_1<P: JsonRpcClient>(
        &self,
        pair_address: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<H160, PairSyncError<P>> {
        match self {
            //Can match on v2 or v3 because they both have the same interface for token0, token1
            PoolVariant::UniswapV2 | PoolVariant::UniswapV3 => {
                //Initialize a new instance of the Pool
                let v2_pair = abi::IUniswapV2Pair::new(pair_address, provider);

                // Make a call to get token0 to initialize a_to_b
                let token0 = match v2_pair.token_0().call().await {
                    Ok(result) => result,
                    Err(contract_error) => {
                        return Err(PairSyncError::ContractError(contract_error))
                    }
                };
                Ok(token0)
            }
        }
    }
}
