use std::sync::Arc;

use ethers::{
    abi::{ParamType, Token},
    prelude::abigen,
    providers::Middleware,
    types::Bytes,
};

use crate::{error::CFMMError, pool::Pool};

use super::uniswap_v2::GetUniswapV2PairsBatchRequest;

abigen!(
    GetUniswapV3PoolDataBatchRequest,
    "src/batch_requests/uniswap_v3/GetUniswapV3PoolDataBatchRequest.json";
    // SyncUniswapV3PoolBatchRequest,
    // "src/batch_requests/uniswap_v3/SyncUniswapV3PoolBatchRequest.json";
);

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
                ParamType::Uint(128), // liquidity
                ParamType::Uint(160), // sqrtPrice
                ParamType::Int(24),   // tick
                ParamType::Int(24),   // tickSpacing
                ParamType::Uint(24),  // fee
                ParamType::Int(128),  // liquidityNet
            ],
            data,
        )?;

        //Update the pool data
        if let Pool::UniswapV3(uniswap_v3_pool) = pools.get_mut(pool_idx).unwrap() {
            uniswap_v3_pool.token_a = tokens[0].to_owned().into_address().unwrap();
            uniswap_v3_pool.token_a_decimals =
                tokens[1].to_owned().into_uint().unwrap().as_u32() as u8;
            uniswap_v3_pool.token_b = tokens[2].to_owned().into_address().unwrap();
            uniswap_v3_pool.token_b_decimals =
                tokens[3].to_owned().into_uint().unwrap().as_u32() as u8;
            uniswap_v3_pool.liquidity = tokens[3].to_owned().into_uint().unwrap().as_u128();
            uniswap_v3_pool.sqrt_price = tokens[4].to_owned().into_uint().unwrap();
            uniswap_v3_pool.tick = tokens[5].to_owned().into_uint().unwrap().as_u64() as i32;
            uniswap_v3_pool.tick_spacing =
                tokens[7].to_owned().into_uint().unwrap().as_u64() as i32;
            uniswap_v3_pool.fee = tokens[7].to_owned().into_uint().unwrap().as_u64() as u32;
            uniswap_v3_pool.liquidity_net =
                tokens[8].to_owned().into_uint().unwrap().as_u128() as i128;
        }

        pool_idx += 1;
    }

    Ok(())
}
