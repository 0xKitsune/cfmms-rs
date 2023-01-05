use std::{str::FromStr, sync::Arc};

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256, U256},
};

use crate::{
    abi,
    error::CFMMError,
    pool::{Pool, UniswapV3Pool},
};

use super::DexVariant;

#[derive(Debug, Clone, Copy)]
pub struct UniswapV3Dex {
    pub factory_address: H160,
    pub creation_block: BlockNumber,
}

impl UniswapV3Dex {
    pub fn new(factory_address: H160, creation_block: BlockNumber) -> UniswapV3Dex {
        UniswapV3Dex {
            factory_address,
            creation_block,
        }
    }

    pub fn swap_event_signature(&self) -> H256 {
        //TODO: make this a const
        abi::IUNISWAPV3POOL_ABI.event("Swap").unwrap().signature()
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        //TODO: make this a const
        abi::IUNISWAPV3FACTORY_ABI
            .event("PoolCreated")
            .unwrap()
            .signature()
    }

    pub async fn new_pool_from_event<M: Middleware>(
        &self,
        log: Log,
        middleware: Arc<M>,
    ) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Uint(32), ParamType::Address], &log.data)?;
        let pair_address = tokens[1].to_owned().into_address().unwrap();
        Pool::new_from_address(pair_address, DexVariant::UniswapV3, middleware).await
    }

    pub fn new_empty_pool_from_event<M: Middleware>(&self, log: Log) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Uint(32), ParamType::Address], &log.data)?;
        let token_a = H160::from(log.topics[0]);
        let token_b = H160::from(log.topics[1]);
        let fee = tokens[0].to_owned().into_uint().unwrap().as_u32();
        let address = tokens[1].to_owned().into_address().unwrap();

        Ok(Pool::UniswapV3(UniswapV3Pool {
            address,
            token_a,
            token_b,
            token_a_decimals: 0,
            token_b_decimals: 0,
            fee,
            liquidity: 0,
            sqrt_price: U256::zero(),
            tick_spacing: 0,
            tick: 0,
            tick_word: U256::zero(),
            liquidity_net: 0,
        }))
    }
}
