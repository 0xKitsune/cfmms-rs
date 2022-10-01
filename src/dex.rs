use std::sync::Arc;

use ethers::{
    prelude::U256,
    providers::{JsonRpcClient, Provider},
    types::{Address, BlockNumber, Log, H160},
};

use crate::{
    abi,
    error::PairSyncError,
    pool::{Pool, PoolVariant},
};

#[derive(Debug, Clone, Copy)]
pub struct Dex {
    pub factory_address: H160,
    pub pool_variant: PoolVariant,
    pub creation_block: BlockNumber,
}

impl Dex {
    pub fn new(factory_address: H160, pool_variant: PoolVariant, creation_block: u64) -> Dex {
        Dex {
            factory_address,
            pool_variant,
            creation_block: BlockNumber::Number(creation_block.into()),
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
    ) -> Result<(H160, u32), PairSyncError<P>> {
        match self.pool_variant {
            PoolVariant::UniswapV2 => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(self.factory_address, provider);

                Ok((
                    uniswap_v2_factory.get_pair(token_a, token_b).call().await?,
                    300,
                ))
            }

            PoolVariant::UniswapV3 => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(self.factory_address, provider.clone());

                let mut best_liquidity = 0;
                let mut best_pool_address = H160::zero();
                let mut best_fee = 100;

                for fee in [100, 300, 500, 1000] {
                    let pool_address = uniswap_v3_factory
                        .get_pool(token_a, token_b, fee)
                        .call()
                        .await?;

                    let uniswap_v3_pool = abi::IUniswapV3Pool::new(pool_address, provider.clone());

                    let liquidity = uniswap_v3_pool.liquidity().call().await?;
                    if best_liquidity < liquidity {
                        best_liquidity = liquidity;
                        best_pool_address = pool_address;
                        best_fee = fee;
                    }
                }

                Ok((best_pool_address, best_fee))
            }
        }
    }

    pub fn new_pool_from_event<P: JsonRpcClient>(
        &self,
        log: Log,
        provider: Arc<Provider<P>>,
    ) -> Result<Pool, PairSyncError<P>> {
        match self.pool_variant {
            PoolVariant::UniswapV2 => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(self.factory_address, provider);

                let (token_a, token_b, address, _) =
                    uniswap_v2_factory.decode_event::<(Address, Address, Address, U256)>(
                        "PairCreated",
                        log.topics,
                        log.data,
                    )?;

                Ok(Pool {
                    pool_variant: PoolVariant::UniswapV2,
                    address,
                    token_a,
                    token_b,
                    //Initialize the following variables as zero values
                    //They will be populated when getting pair reserves
                    token_a_decimals: 0,
                    token_b_decimals: 0,
                    a_to_b: false,
                    reserve_0: 0,
                    reserve_1: 0,
                    fee: 300,
                })
            }
            PoolVariant::UniswapV3 => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(self.factory_address, provider);

                let (token_a, token_b, fee, _, address) =
                    uniswap_v3_factory.decode_event::<(Address, Address, u32, u128, Address)>(
                        "PoolCreated",
                        log.topics,
                        log.data,
                    )?;

                Ok(Pool {
                    pool_variant: PoolVariant::UniswapV3,
                    address,
                    token_a,
                    token_b,
                    //Initialize the following variables as zero values
                    //They will be populated when getting pool reserves
                    token_a_decimals: 0,
                    token_b_decimals: 0,
                    a_to_b: false,
                    reserve_0: 0,
                    reserve_1: 0,
                    fee,
                })
            }
        }
    }
}
