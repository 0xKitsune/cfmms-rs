use std::sync::Arc;

use ethers::{
    abi::Token,
    prelude::{abigen, ContractError},
    providers::Middleware,
    types::{Bytes, H160, U256},
};

use crate::error::CFMMError;

abigen!(
    GetUniswapV2PairsBatchRequest,
    "src/batch_requests/uniswap_v2/GetUniswapV2PairsBatchRequest.json";
    GetUniswapV2PoolDataBatchRequest,
    "src/batch_requests/uniswap_v2/GetUniswapV2PoolDataBatchRequest.json";
    SyncUniswapV2PoolBatchRequest,
    "src/batch_requests/uniswap_v2/SyncUniswapV2PoolBatchRequest.json";
);

pub async fn get_all_pairs<M: Middleware>(
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
