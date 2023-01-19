use std::{io::Read, sync::Arc, thread::sleep, time::Duration};

use ethers::{
    abi::{Param, ParamType, Token},
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

    let return_data_tokens = ethers::abi::decode(
        &vec![ParamType::Array(Box::new(ParamType::Address))],
        &return_data,
    )?;

    for token_array in return_data_tokens {
        if let Some(arr) = token_array.into_array() {
            for token in arr {
                match token.into_address() {
                    Some(addr) => {
                        if !addr.is_zero() {
                            pairs.push(addr);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(pairs)
}

pub async fn get_pool_data_batch_request<M: Middleware>(
    pools: &mut [Pool],
    middleware: Arc<M>,
) -> Result<(), CFMMError<M>> {
    let mut target_addresses = vec![];
    for pool in pools.iter() {
        println!("addr: {:?}", pool.address());
        target_addresses.push(Token::Address(pool.address()));
    }

    println!("pool len: {:?}", pools.len());

    let constructor_args = Token::Tuple(vec![Token::Array(target_addresses)]);

    let deployer =
        GetUniswapV2PoolDataBatchRequest::deploy(middleware.clone(), constructor_args).unwrap();

    let return_data: Bytes = deployer.call_raw().await?;
    println!("datalen :{:?}", return_data.len());
    println!("data :{:?}", return_data);

    let return_data_tokens = ethers::abi::decode(
        &vec![ParamType::Array(Box::new(ParamType::Tuple(vec![
            ParamType::Address,   // addr
            ParamType::Address,   // token a
            ParamType::Uint(8),   // token a decimals
            ParamType::Address,   // token b
            ParamType::Uint(8),   // token b decimals
            ParamType::Uint(112), // reserve 0
            ParamType::Uint(112), // reserve 1
        ])))],
        &return_data,
    );
    println!("");

    println!("retdatatokens: {:?}", return_data_tokens);

    // if return_data.len() > 0 {
    //     let mut pool_idx = 0;
    //     //Chunk the return data, populate the pools,
    //     for data in return_data.chunks(192) {
    //         let tokens = ethers::abi::decode(
    //             &vec![
    //                 ParamType::Address,   // token a
    //                 ParamType::Uint(8),   // token a decimals
    //                 ParamType::Address,   // token b
    //                 ParamType::Uint(8),   // token b decimals
    //                 ParamType::Uint(112), // reserve 0
    //                 ParamType::Uint(112), // reserve 1
    //             ],
    //             data,
    //         )?;
    //         println!("here");

    //         // //Update the pool data
    //         // if let Pool::UniswapV2(uniswap_v2_pool) = pools.get_mut(pool_idx).unwrap() {
    //         //     uniswap_v2_pool.token_a = tokens[0].to_owned().into_address().unwrap();
    //         //     uniswap_v2_pool.token_a_decimals =
    //         //         tokens[1].to_owned().into_uint().unwrap().as_u32() as u8;
    //         //     uniswap_v2_pool.token_b = tokens[2].to_owned().into_address().unwrap();
    //         //     uniswap_v2_pool.token_b_decimals =
    //         //         tokens[3].to_owned().into_uint().unwrap().as_u32() as u8;
    //         //     uniswap_v2_pool.reserve_0 = tokens[3].to_owned().into_uint().unwrap().as_u128();
    //         //     uniswap_v2_pool.reserve_1 = tokens[4].to_owned().into_uint().unwrap().as_u128();
    //         // }

    //         pool_idx += 1;
    //     }
    // }

    Ok(())
}
