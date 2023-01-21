use std::{error::Error, sync::Arc};

use ethers::providers::{Http, Provider};

use cfmms::checkpoint::sync_pools_from_checkpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = std::env::var("ETHEREUM_MAINNET_ENDPOINT")
        .expect("Could not get ETHEREUM_MAINNET_ENDPOINT");
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let _pools = sync_pools_from_checkpoint("./pool_sync_checkpoint.json".into(), provider).await?;

    Ok(())
}
