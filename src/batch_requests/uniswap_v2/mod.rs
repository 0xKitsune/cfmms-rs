use std::sync::Arc;

use ethers::{
    abi::Token,
    prelude::{abigen, ContractError},
    providers::Middleware,
    types::{H160, U256},
};

use crate::{error::CFMMError, pool::Pool};

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
    let pairs = vec![];

    //TODO: FIXME: Just using unwrap right now to debug, DecodingError(InvalidData)
    let deployer = GetUniswapV2PairsBatchRequest::deploy(
        middleware,
        vec![
            Token::Uint(from),
            Token::Uint(step),
            Token::Address(factory),
        ],
    )
    .unwrap();

    let x = deployer.send().await.unwrap();

    Ok(pairs)
}
