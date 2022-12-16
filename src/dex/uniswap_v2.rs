use std::{str::FromStr, sync::Arc};

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256},
};

use crate::{
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
        H256::from_str("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1")
            .unwrap()
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        //TODO: make this a const
        H256::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9")
            .unwrap()
    }

    pub async fn new_pool_from_event<M: Middleware>(
        &self,
        log: Log,
        middleware: Arc<M>,
    ) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(
            &[
                ParamType::Address,
                ParamType::Address,
                ParamType::Address,
                ParamType::Uint(256),
            ],
            &log.data,
        )?;

        let pair_address = tokens[2].to_owned().into_address().unwrap();
        Pool::new_from_address(pair_address, DexVariant::UniswapV2, middleware).await
    }

    pub fn new_empty_pool_from_event<M: Middleware>(&self, log: Log) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(
            &[
                ParamType::Address,
                ParamType::Address,
                ParamType::Address,
                ParamType::Uint(256),
            ],
            &log.data,
        )?;

        let token_a = tokens[0].to_owned().into_address().unwrap();
        let token_b = tokens[1].to_owned().into_address().unwrap();
        let address = tokens[2].to_owned().into_address().unwrap();

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
