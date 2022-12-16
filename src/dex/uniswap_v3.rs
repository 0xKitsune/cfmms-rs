use std::{str::FromStr, sync::Arc};

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256, U256},
};

use crate::{
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
        H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67")
            .unwrap()
    }

    pub fn pool_created_event_signature(&self) -> H256 {
        //TODO: make this a const
        H256::from_str("0x783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118")
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
                ParamType::Uint(32),
                ParamType::Uint(128),
                ParamType::Address,
            ],
            &log.data,
        )?;

        let pair_address = tokens[4].to_owned().into_address().unwrap();
        Pool::new_from_address(pair_address, DexVariant::UniswapV3, middleware).await
    }

    pub fn new_empty_pool_from_event<M: Middleware>(&self, log: Log) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(
            &[
                ParamType::Address,
                ParamType::Address,
                ParamType::Uint(32),
                ParamType::Uint(128),
                ParamType::Address,
            ],
            &log.data,
        )?;

        let token_a = tokens[0].to_owned().into_address().unwrap();
        let token_b = tokens[1].to_owned().into_address().unwrap();
        let fee = tokens[2].to_owned().into_uint().unwrap().as_u32();
        let address = tokens[4].to_owned().into_address().unwrap();

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
