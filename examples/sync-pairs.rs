use std::{error::Error, str::FromStr, sync::Arc};

use ethers::{
    providers::{Http, Provider},
    types::H160,
};

use pair_sync::{
    dex::{Dex, DexVariant},
    sync,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let mut dexes = vec![];

    //Add UniswapV2
    dexes.push(Dex::new(
        //Specify the factory address
        H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap(),
        //Specify the dex variant
        DexVariant::UniswapV2,
        //Specify the factory contract's creation block number
        2638438,
    ));

    //Add Sushiswap
    dexes.push(Dex::new(
        H160::from_str("0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac").unwrap(),
        DexVariant::UniswapV2,
        10794229,
    ));

    //Add UniswapV3
    dexes.push(Dex::new(
        H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
        DexVariant::UniswapV3,
        12369621,
    ));

    //Sync pairs
    sync::sync_pairs(dexes, provider, false).await?;

    Ok(())
}
