use std::{error::Error, str::FromStr, sync::Arc};

use ethers::{
    providers::{Http, Provider},
    types::H160,
};

use cfmms::{
    dex::{Dex, DexVariant},
    sync,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let dexes = vec![
        //Add UniswapV3
        Dex::new(
            H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            DexVariant::UniswapV3,
            12369621,
        ),
    ];

    //Sync pairs
    sync::sync_pairs_with_throttle(dexes, provider, 5, false).await?;
    Ok(())
}
