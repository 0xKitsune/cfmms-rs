use std::{str::FromStr, sync::Arc};

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256},
};

use crate::{
    abi,
    error::CFMMError,
    pool::{Pool, UniswapV2Pool},
};

use super::DexVariant;

#[derive(Debug, Clone, Copy)]
pub struct UniswapV2Dex {
    pub factory_address: H160,
    pub creation_block: BlockNumber,
}

impl UniswapV2Dex {
    pub fn new(factory_address: H160, creation_block: BlockNumber) -> UniswapV2Dex {
        UniswapV2Dex {
            factory_address,
            creation_block,
        }
    }

    pub fn sync_event_signature(&self) -> H256 {
        //TODO: make this a const
        abi::IUNISWAPV2PAIR_ABI.event("Sync").unwrap().signature()
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        //TODO: make this a const
        abi::IUNISWAPV2FACTORY_ABI
            .event("PairCreated")
            .unwrap()
            .signature()
    }

    pub async fn new_pool_from_event<M: Middleware>(
        &self,
        log: Log,
        middleware: Arc<M>,
    ) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Address, ParamType::Uint(256)], &log.data)?;
        let pair_address = tokens[0].to_owned().into_address().unwrap();
        Pool::new_from_address(pair_address, DexVariant::UniswapV2, middleware).await
    }

    pub fn new_empty_pool_from_event<M: Middleware>(&self, log: Log) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Address, ParamType::Uint(256)], &log.data)?;
        let token_a = H160::from(log.topics[0]);
        let token_b = H160::from(log.topics[1]);
        let address = tokens[0].to_owned().into_address().unwrap();

        Ok(Pool::UniswapV2(UniswapV2Pool {
            address,
            token_a,
            token_b,
            token_a_decimals: 0,
            token_b_decimals: 0,
            reserve_0: 0,
            reserve_1: 0,
            fee: 300,
        }))
    }
}
