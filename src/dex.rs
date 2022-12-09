use std::{str::FromStr, sync::Arc};

use ethers::{
    prelude::U256,
    providers::{JsonRpcClient, Provider},
    types::{Address, BlockNumber, Log, H160, H256},
};

use crate::{
    abi,
    error::CFFMError,
    pool::{Pool, UniswapV2Pool, UniswapV3Pool},
};

#[derive(Debug, Clone, Copy)]
pub struct Dex {
    pub factory_address: H160,
    pub dex_variant: DexVariant,
    pub creation_block: BlockNumber,
}

#[derive(Debug, Clone, Copy)]
pub enum DexVariant {
    UniswapV2,
    UniswapV3,
}

impl DexVariant {
    //Sync event or comparable reserve update event (Ex. Swap for Univ3)
    pub fn sync_event_signature(&self) -> H256 {
        match self {
            DexVariant::UniswapV2 => {
                H256::from_str("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1")
                    .unwrap()
            }

            DexVariant::UniswapV3 => {
                H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67")
                    .unwrap()
            }
        }
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        match self {
            DexVariant::UniswapV2 => {
                H256::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9")
                    .unwrap()
            }

            DexVariant::UniswapV3 => {
                H256::from_str("0x783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118")
                    .unwrap()
            }
        }
    }
}

impl Dex {
    pub fn new(factory_address: H160, dex_variant: DexVariant, creation_block: u64) -> Dex {
        Dex {
            factory_address,
            dex_variant,
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
    ) -> Result<Option<Pool>, CFFMError<P>> {
        match self.dex_variant {
            DexVariant::UniswapV2 => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(self.factory_address, provider.clone());

                let pair_address = uniswap_v2_factory.get_pair(token_a, token_b).call().await?;

                if pair_address.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(Pool::UniswapV2(
                        UniswapV2Pool::new_from_address(pair_address, provider).await?,
                    )))
                }
            }

            DexVariant::UniswapV3 => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(self.factory_address, provider.clone());

                let mut best_liquidity = 0;
                let mut best_pool_address = H160::zero();

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

    pub fn new_empty_pool_from_event<P: JsonRpcClient>(
        &self,
        log: Log,
        provider: Arc<Provider<P>>,
    ) -> Result<Pool, CFFMError<P>> {
        match self.dex_variant {
            DexVariant::UniswapV2 => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(self.factory_address, provider);

                let (token_a, token_b, address, _) =
                    uniswap_v2_factory.decode_event::<(Address, Address, Address, U256)>(
                        "PairCreated",
                        log.topics,
                        log.data,
                    )?;

                Ok(Pool::UniswapV2(UniswapV2Pool {
                    address,
                    token_a,
                    token_b,
                    //Initialize the following variables as zero values
                    //They will be populated when getting pair reserves
                    token_a_decimals: 0,
                    token_b_decimals: 0,
                    reserve_0: 0,
                    reserve_1: 0,
                    fee: 300,
                }))
            }
            DexVariant::UniswapV3 => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(self.factory_address, provider);

                let (token_a, token_b, fee, _, address) =
                    uniswap_v3_factory.decode_event::<(Address, Address, u32, u128, Address)>(
                        "PoolCreated",
                        log.topics,
                        log.data,
                    )?;

                Ok(Pool::UniswapV3(UniswapV3Pool {
                    address,
                    token_a,
                    token_b,
                    //Initialize the following variables as zero values
                    //They will be populated when getting pair reserves
                    token_a_decimals: 0,
                    token_b_decimals: 0,
                    liquidity: 0,
                    sqrt_price: U256::zero(),
                    tick_spacing: 0,
                    tick: 0,
                    tick_word: U256::zero(),
                    liquidity_net: 0,
                    initialized: false,
                    fee,
                }))
            }
        }
    }

    pub async fn new_pool_from_event<P: 'static + JsonRpcClient>(
        &self,
        log: Log,
        provider: Arc<Provider<P>>,
    ) -> Result<Pool, CFFMError<P>> {
        match self.dex_variant {
            DexVariant::UniswapV2 => {
                let uniswap_v2_factory =
                    abi::IUniswapV2Factory::new(self.factory_address, provider.clone());

                let (_token_a, _token_b, address, _) =
                    uniswap_v2_factory.decode_event::<(Address, Address, Address, U256)>(
                        "PairCreated",
                        log.topics,
                        log.data,
                    )?;

                //Initialize token decimals, reserves and a_to_b variables as zero values
                //They will be populated when getting pool reserves
                Ok(Pool::UniswapV2(
                    UniswapV2Pool::new_from_address(address, provider).await?,
                ))
            }
            DexVariant::UniswapV3 => {
                let uniswap_v3_factory =
                    abi::IUniswapV3Factory::new(self.factory_address, provider.clone());

                let (_token_a, _token_b, _fee, _, address) =
                    uniswap_v3_factory.decode_event::<(Address, Address, u32, u128, Address)>(
                        "PoolCreated",
                        log.topics,
                        log.data,
                    )?;

                //Initialize token decimals, reserves and a_to_b variables as zero values
                //They will be populated when getting pool reserves
                Ok(Pool::UniswapV3(
                    UniswapV3Pool::new_from_address(address, provider).await?,
                ))
            }
        }
    }
}
