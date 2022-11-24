use std::{error::Error, str::FromStr, sync::Arc};

use ethers::{
    providers::{Http, Provider},
    types::H160,
};

use pool_sync::{
    dex::{Dex, DexVariant},
    filter,
    pool::{Pool, UniswapV2Pool},
    sync,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let mut dexes = vec![];

    //Add UniswapV3
    dexes.push(Dex::new(
        H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
        DexVariant::UniswapV3,
        12369621,
    ));

    //Sync pools
    let pools = sync::sync_pairs_with_throttle(dexes.clone(), provider.clone(), 10, false).await?;

    //Create a list of blacklisted tokens
    let blacklisted_tokens =
        vec![H160::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984").unwrap()];

    //Filter out blacklisted tokens
    let filtered_pools = filter::filter_blacklisted_tokens(pools, blacklisted_tokens);

    //UniswapV2 usdc weth pool on Eth mainnet
    let _uniswap_v2_usdc_weth_pool = Pool::UniswapV2(
        UniswapV2Pool::new_from_address(
            H160::from_str("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc").unwrap(),
            provider.clone(),
        )
        .await?,
    );

    let weth_address = H160::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
    let usdc_address = H160::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    let usd_weth_pair_address =
        H160::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let usd_weth_pool = Pool::UniswapV2(
        UniswapV2Pool::new_from_address(usd_weth_pair_address, provider.clone()).await?,
    );

    let _filtered_pools = filter::filter_pools_below_usd_threshold_with_throttle(
        filtered_pools,
        dexes,
        usd_weth_pool,
        usdc_address,
        weth_address,
        100000.00, //Setting usd_threshold to 100000.00 filters out any pool that contains less than $100k USD
        // When getting token to weth price to determine weth value in pool, dont use price with weth reserves with less than 2 eth
        2000000000000000000 as u128,
        provider.clone(),
        10,
    )
    .await?;

    Ok(())
}
