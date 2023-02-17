use std::{error::Error, fs::OpenOptions, io::{Write}, str::FromStr, sync::Arc};

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
        //UniswapV2
        // Dex::new(
        //     H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap(),
        //     DexVariant::UniswapV2,
        //     2638438,
        // ),
        //Add Sushiswap
        // Dex::new(
        //     H160::from_str("0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac").unwrap(),
        //     DexVariant::UniswapV2,
        //     10794229,
        // ),
        //Add UniswapV3
        Dex::new(
            H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            DexVariant::UniswapV3,
            12369621,
        ),
    ];

    //Sync pairs 
    let pools = sync::sync_pairs_with_throttle(dexes, 100000, provider, 5, None).await?;
    
    // CSV file logic
    for pool in &pools {
        match pool {
            Pool::UniswapV2(pool) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("UniswapV2Pools.csv")
                    .unwrap();
                // Serialize pool enum to a string (OLD WAY)
                let pool_string = format!("{:?}", pool); // We convert the pool enum to a string using format! macro
                writeln!(file, "{}", pool_string).unwrap();
            }
            Pool::UniswapV3(pool) => {
                // this is where the new csv file is written
                let file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("UniswapV3Pools.csv")
                    .unwrap();
                                
                // add header vector to file
                let mut wtr = csv::WriterBuilder::new()
                    .has_headers(false)
                    .from_writer(file);

                // write pool values to file
                wtr.serialize(pool)?;
                wtr.flush()?;
            }
        }
    }
    Ok(())
}

