use std::str::FromStr;

use ethers::{providers::ProviderError, types::H160};

use pair_sync::{
    dex::{Dex, DexType},
    filter, sync,
};

#[tokio::main]
async fn main() -> Result<(), ProviderError> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";

    let mut dexes = vec![];

    //Add UniswapV3
    dexes.push(Dex::new(
        "0x1F98431c8aD98523631AE4a59f267346ea31F984",
        DexType::UniswapV3,
        12369621,
    ));

    //Sync pairs
    let pairs = sync::sync_pairs_with_throttle(dexes, rpc_endpoint, 10).await?;

    //Create a list of blacklisted tokens
    let blacklisted_tokens =
        vec![H160::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984").unwrap()];

    //Filter out blacklisted tokens
    let _filtered_pairs = filter::filter_blacklisted_tokens(pairs, blacklisted_tokens);

    Ok(())
}
