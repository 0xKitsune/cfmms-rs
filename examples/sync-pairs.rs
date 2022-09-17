use std::str::FromStr;

use ethers::{
    providers::ProviderError,
    types::{BlockNumber, H160, U64},
};

use pair_sync::{dex::Dex, dex::DexType, sync};

#[tokio::main]
async fn main() -> Result<(), ProviderError> {
    //Add rpc endpoint here:
    let rpc_endpoint = "";

    let mut dexes = vec![];

    //Add UniswapV2
    dexes.push(Dex {
        factory_address: H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap(),
        dex_type: DexType::UniswapV2,
        creation_block: BlockNumber::Number(U64([2638438])),
    });

    //Add Sushiswap
    dexes.push(Dex {
        factory_address: H160::from_str("0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac").unwrap(),
        dex_type: DexType::UniswapV2,
        creation_block: BlockNumber::Number(U64([10794229])),
    });

    //Add UniswapV3
    dexes.push(Dex {
        factory_address: H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
        dex_type: DexType::UniswapV3,
        creation_block: BlockNumber::Number(U64([12369621])),
    });

    //Sync pairs
    sync::sync_pairs(dexes, rpc_endpoint).await?;

    Ok(())
}
