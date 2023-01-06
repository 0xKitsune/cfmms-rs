use std::sync::Arc;

use ethers::{
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256},
};

use crate::{
    abi,
    error::CFMMError,
    pool::{self, Pool, UniswapV2Pool, UniswapV3Pool},
};

use self::{uniswap_v2::UniswapV2Dex, uniswap_v3::UniswapV3Dex};

pub mod uniswap_v2;
pub mod uniswap_v3;

#[derive(Debug, Clone, Copy)]
pub enum Dex {
    UniswapV2(UniswapV2Dex),
    UniswapV3(UniswapV3Dex),
}

impl Dex {
    pub fn new(factory_address: H160, dex_variant: DexVariant, creation_block: u64) -> Dex {
        match dex_variant {
            DexVariant::UniswapV2 => Dex::UniswapV2(UniswapV2Dex::new(
                factory_address,
                BlockNumber::Number(creation_block.into()),
            )),

            DexVariant::UniswapV3 => Dex::UniswapV3(UniswapV3Dex::new(
                factory_address,
                BlockNumber::Number(creation_block.into()),
            )),
        }
    }

    pub fn factory_address(&self) -> H160 {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => uniswap_v2_dex.factory_address,
            Dex::UniswapV3(uniswap_v3_dex) => uniswap_v3_dex.factory_address,
        }
    }

    pub fn creation_block(&self) -> BlockNumber {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => uniswap_v2_dex.creation_block,
            Dex::UniswapV3(uniswap_v3_dex) => uniswap_v3_dex.creation_block,
        }
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => uniswap_v2_dex.pool_created_event_signature(),
            Dex::UniswapV3(uniswap_v3_dex) => uniswap_v3_dex.pool_created_event_signature(),
        }
    }

    pub async fn new_pool_from_event_log<M: Middleware>(
        &self,
        log: Log,
        middleware: Arc<M>,
    ) -> Result<Pool, CFMMError<M>> {
        pool::Pool::new_from_event_log(log, middleware).await
    }

    pub fn new_empty_pool_from_event_log<M: Middleware>(
        &self,
        log: Log,
    ) -> Result<Pool, CFMMError<M>> {
        pool::Pool::new_empty_pool_from_event_log(log)
    }

    //TODO: rename this to be specific to what it needs to do
    //This should get the pool with the best liquidity from the dex variant.
    //If univ2, there will only be one pool, if univ3 there will be multiple
    pub async fn get_pool_with_best_liquidity<M: Middleware>(
        &self,
        token_a: H160,
        token_b: H160,
        middleware: Arc<M>,
    ) -> Result<Option<Pool>, CFMMError<M>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(uniswap_v2_dex.factory_address, middleware.clone());

                let pair_address = uniswap_v2_factory.get_pair(token_a, token_b).call().await?;

                if pair_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(Pool::UniswapV2(
                        UniswapV2Pool::new_from_address(pair_address, middleware).await?,
                    )))
                }
            }

            Dex::UniswapV3(uniswap_v3_dex) => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(uniswap_v3_dex.factory_address, middleware.clone());

                let mut best_liquidity = 0;
                let mut best_pool_address = H160::zero();

                for fee in [100, 300, 500, 1000] {
                    let pool_address = match uniswap_v3_factory
                        .get_pool(token_a, token_b, fee)
                        .call()
                        .await
                    {
                        Ok(address) => {
                            if !address.is_zero() {
                                address
                            } else {
                                continue;
                            }
                        }
                        Err(_) => {
                            //TODO: return descriptive errors if there is an issue with the contract or if the pair does not exist
                            continue;
                        }
                    };

                    let uniswap_v3_pool =
                        abi::IUniswapV3Pool::new(pool_address, middleware.clone());

                    let liquidity = uniswap_v3_pool.liquidity().call().await?;
                    if best_liquidity < liquidity {
                        best_liquidity = liquidity;
                        best_pool_address = pool_address;
                    }
                }

                if best_pool_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(Pool::UniswapV3(
                        UniswapV3Pool::new_from_address(best_pool_address, middleware).await?,
                    )))
                }
            }
        }
    }

    //If univ2, there will only be one pool, if univ3 there will be multiple
    pub async fn get_all_pools_for_pair<M: Middleware>(
        &self,
        token_a: H160,
        token_b: H160,
        middleware: Arc<M>,
    ) -> Result<Option<Vec<Pool>>, CFMMError<M>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(uniswap_v2_dex.factory_address, middleware.clone());

                let pair_address = uniswap_v2_factory.get_pair(token_a, token_b).call().await?;

                if pair_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(vec![Pool::UniswapV2(
                        UniswapV2Pool::new_from_address(pair_address, middleware).await?,
                    )]))
                }
            }

            Dex::UniswapV3(uniswap_v3_dex) => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(uniswap_v3_dex.factory_address, middleware.clone());

                let mut pools = vec![];

                for fee in [100, 300, 500, 1000] {
                    match uniswap_v3_factory
                        .get_pool(token_a, token_b, fee)
                        .call()
                        .await
                    {
                        Ok(address) => {
                            if !address.is_zero() {
                                pools.push(Pool::UniswapV3(
                                    UniswapV3Pool::new_from_address(address, middleware.clone())
                                        .await?,
                                ))
                            }
                        }

                        Err(_) => {
                            //TODO: return descriptive errors if there is an issue with the contract or if the pair does not exist
                            continue;
                        }
                    };
                }

                if pools.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(pools))
                }
            }
        }
    }
}

pub enum DexVariant {
    UniswapV2,
    UniswapV3,
}
impl DexVariant {
    pub fn pool_created_event_signature(&self) -> H256 {
        match self {
            DexVariant::UniswapV2 => uniswap_v2::PAIR_CREATED_EVENT_SIGNATURE,
            DexVariant::UniswapV3 => uniswap_v3::POOL_CREATED_EVENT_SIGNATURE,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr, sync::Arc};

    use ethers::{
        providers::{Http, Provider},
        types::H160,
    };

    use super::{Dex, DexVariant};

    #[test]
    fn test_factory_address() {}

    #[test]
    fn test_get_pool_with_best_liquidity() {}

    #[tokio::test]
    async fn test_get_all_pools_for_pair() {
        //Univ3 on ethereum
        let univ3_pool = Dex::new(
            H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            DexVariant::UniswapV3,
            12369621,
        );

        let usdc = H160::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        let weth = H160::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();

        let provider = Arc::new(
            Provider::<Http>::try_from(
                env::var("ETHEREUM_MAINNET_ENDPOINT").expect("Could not initialize provider"),
            )
            .unwrap(),
        );

        let pools = univ3_pool
            .get_all_pools_for_pair(usdc, weth, provider)
            .await
            .expect("Could not get all pools for pair");

        println!("Pools: {:?}", pools);
    }
}
