use std::{io::Read, sync::Arc};

use ethers::{
    abi::{ParamType, Token},
    prelude::{abigen, ContractError},
    providers::Middleware,
    types::{Bytes, H160, U256},
};

use crate::{
    error::CFMMError,
    pool::{Pool, UniswapV2Pool},
};

abigen!(
    GetUniswapV2PairsBatchRequest,
    "src/batch_requests/uniswap_v2/GetUniswapV2PairsBatchRequest.json";
    GetUniswapV2PoolDataBatchRequest,
    "src/batch_requests/uniswap_v2/GetUniswapV2PoolDataBatchRequest.json";
);

pub async fn get_pairs_batch_request<M: Middleware>(
    factory: H160,
    from: U256,
    step: U256,
    middleware: Arc<M>,
) -> Result<Vec<H160>, CFMMError<M>> {
    let mut pairs = vec![];

    let constructor_args = Token::Tuple(vec![
        Token::Uint(from),
        Token::Uint(step),
        Token::Address(factory),
    ]);

    let deployer = GetUniswapV2PairsBatchRequest::deploy(middleware, constructor_args).unwrap();
    let return_data: Bytes = deployer.call_raw().await?;

    for address_bytes in return_data.chunks(32) {
        pairs.push(H160::from_slice(&address_bytes[12..]));
    }

    Ok(pairs)
}

pub async fn get_pool_data_batch_request<M: Middleware>(
    pools: &mut [Pool],
    middleware: Arc<M>,
) -> Result<(), CFMMError<M>> {
    let mut target_addresses = vec![];
    for pool in pools.iter() {
        target_addresses.push(Token::Address(pool.address()));
    }

    let constructor_args = Token::Tuple(vec![Token::Array(target_addresses)]);

    let deployer =
        GetUniswapV2PairsBatchRequest::deploy(middleware.clone(), constructor_args).unwrap();

    let return_data: Bytes = deployer.call_raw().await?;

    let mut pool_idx = 0;
    //Chunk the return data, populate the pools,
    for data in return_data.chunks(160) {
        let tokens = ethers::abi::decode(
            &vec![
                ParamType::Address,   // token a
                ParamType::Uint(8),   // token a decimals
                ParamType::Address,   // token b
                ParamType::Uint(8),   // token b decimals
                ParamType::Uint(112), // reserve 0
                ParamType::Uint(112), // reserve 1
            ],
            data,
        )?;

        //Update the pool data
        if let Pool::UniswapV2(uniswap_v2_pool) = pools.get_mut(pool_idx).unwrap() {
            uniswap_v2_pool.token_a = tokens[0].to_owned().into_address().unwrap();
            uniswap_v2_pool.token_a_decimals =
                tokens[1].to_owned().into_uint().unwrap().as_u32() as u8;
            uniswap_v2_pool.token_b = tokens[2].to_owned().into_address().unwrap();
            uniswap_v2_pool.token_b_decimals =
                tokens[3].to_owned().into_uint().unwrap().as_u32() as u8;
            uniswap_v2_pool.reserve_0 = tokens[3].to_owned().into_uint().unwrap().as_u128();
            uniswap_v2_pool.reserve_1 = tokens[4].to_owned().into_uint().unwrap().as_u128();
        }

        pool_idx += 1;
    }

    Ok(())
}
