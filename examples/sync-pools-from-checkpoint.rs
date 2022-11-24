use std::{error::Error, sync::Arc};

use ethers::providers::{Http, Provider};

use pair_sync::checkpoint::sync_pools_from_checkpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let _pools = sync_pools_from_checkpoint("./pool_sync_checkpoint.json".into(), provider).await?;

    Ok(())
}
