use std::{error::Error, fs::{File, OpenOptions}, io::{Write, prelude}, str::FromStr, sync::Arc};

use ethers::{
    providers::{Http, Provider},
    types::H160,
};

use cfmms::{
    dex::{Dex, DexVariant},
    sync, pool::Pool,
};



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = std::env::var("ETHEREUM_MAINNET_ENDPOINT")
        .expect("Could not get ETHEREUM_MAINNET_ENDPOINT");
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
    // sync::sync_pairs_with_throttle(dexes, 100000, provider, 5, None).await?;
    let pools = sync::sync_pairs_with_throttle(dexes, 100000, provider, 5, None).await?;

    // NEW
    for pool in &pools {
        match pool {
            Pool::UniswapV2(pool) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("UniswapV2Pools.csv")
                    .unwrap();
                let pool_string = format!("{:?}", pool);
                writeln!(file, "{}", pool_string).unwrap();
            }
            Pool::UniswapV3(pool) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("UniswapV3Pools.csv")
                    .unwrap();
                let pool_string = format!("{:?}", pool);
                writeln!(file, "{}", pool_string).unwrap();
            }
        }
    }

    Ok(())


}

