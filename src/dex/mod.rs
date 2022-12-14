use std::sync::Arc;

use ethers::{
    providers::{JsonRpcClient, Provider},
    types::{BlockNumber, Log, H160, H256},
};

use crate::{
    abi,
    error::CFFMError,
    pool::{Pool, UniswapV2Pool, UniswapV3Pool},
};

use self::{uniswap_v2::UniswapV2Dex, uniswap_v3::UniswapV3Dex};

pub mod uniswap_v2;
pub mod uniswap_v3;

#[derive(Debug, Clone, Copy)]
pub enum Dex {
    UniswapV2(UniswapV2Dex),
    UniswapV3(UniswapV3Dex),
}

pub enum DexVariant {
    UniswapV2,
    UniswapV3,
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

    pub fn sync_event_signature(&self) -> H256 {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => uniswap_v2_dex.sync_event_signature(),
            Dex::UniswapV3(uniswap_v3_dex) => uniswap_v3_dex.swap_event_signature(),
        }
    }

    pub async fn new_pool_from_event<P: 'static + JsonRpcClient>(
        &self,
        log: Log,
        provider: Arc<Provider<P>>,
    ) -> Result<Pool, CFFMError<P>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => {
                Ok(uniswap_v2_dex.new_pool_from_event(log, provider).await?)
            }
            Dex::UniswapV3(uniswap_v3_dex) => {
                Ok(uniswap_v3_dex.new_pool_from_event(log, provider).await?)
            }
        }
    }

    pub fn new_empty_pool_from_event<P: 'static + JsonRpcClient>(
        &self,
        log: Log,
    ) -> Result<Pool, CFFMError<P>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => uniswap_v2_dex.new_empty_pool_from_event(log),
            Dex::UniswapV3(uniswap_v3_dex) => uniswap_v3_dex.new_empty_pool_from_event(log),
        }
    }

    //TODO: rename this to be specific to what it needs to do
    //This should get the pool with the best liquidity from the dex variant.
    //If univ2, there will only be one pool, if univ3 there will be multiple
    pub async fn get_pool_with_best_liquidity<P: 'static + JsonRpcClient>(
        &self,
        token_a: H160,
        token_b: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<Option<Pool>, CFFMError<P>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(uniswap_v2_dex.factory_address, provider.clone());

                let pair_address = uniswap_v2_factory.get_pair(token_a, token_b).call().await?;

                if pair_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(Pool::UniswapV2(
                        UniswapV2Pool::new_from_address(pair_address, provider).await?,
                    )))
                }
            }

            Dex::UniswapV3(uniswap_v3_dex) => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(uniswap_v3_dex.factory_address, provider.clone());

                let mut best_liquidity = 0;
                let mut best_pool_address = H160::zero();

                for fee in [100, 300, 500, 1000] {
                    let pool_address = match uniswap_v3_factory
                        .get_pool(token_a, token_b, fee)
                        .call()
                        .await
                    {
                        Ok(address) => address,
                        Err(err) => {
                            //TODO: return descriptive errors if there is an issue with the contract or if the pair does not exist
                            continue;
                        }
                    };

                    let uniswap_v3_pool = abi::IUniswapV3Pool::new(pool_address, provider.clone());

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
                        UniswapV3Pool::new_from_address(best_pool_address, provider).await?,
                    )))
                }
            }
        }
    }

    //If univ2, there will only be one pool, if univ3 there will be multiple
    pub async fn get_all_pools_for_pair<P: 'static + JsonRpcClient>(
        &self,
        token_a: H160,
        token_b: H160,
        provider: Arc<Provider<P>>,
    ) -> Result<Option<Vec<Pool>>, CFFMError<P>> {
        match self {
            Dex::UniswapV2(uniswap_v2_dex) => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(uniswap_v2_dex.factory_address, provider.clone());

                let pair_address = uniswap_v2_factory.get_pair(token_a, token_b).call().await?;

                if pair_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(vec![Pool::UniswapV2(
                        UniswapV2Pool::new_from_address(pair_address, provider).await?,
                    )]))
                }
            }

            Dex::UniswapV3(uniswap_v3_dex) => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(uniswap_v3_dex.factory_address, provider.clone());

                let mut pools = vec![];

                for fee in [100, 300, 500, 1000] {
                    match uniswap_v3_factory
                        .get_pool(token_a, token_b, fee)
                        .call()
                        .await
                    {
                        Ok(address) => pools.push(Pool::UniswapV3(
                            UniswapV3Pool::new_from_address(address, provider.clone()).await?,
                        )),

                        Err(_) => {
                            //TODO: return descriptive errors if there is an issue with the contract or if the pair does not exist
                            continue;
                        }
                    };
                }

                Ok(Some(pools))
            }
        }
    }
}
