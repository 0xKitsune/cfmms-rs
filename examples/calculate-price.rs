use std::{error::Error, str::FromStr, sync::Arc};

use ethers::{
    prelude::abigen,
    providers::{Http, Provider, Middleware},
    types::{H160},
};

use cfmms::pool::UniswapV3Pool;
use cfmms::abi::IUniswapV3Pool;

abigen!(
    IQuoter,
r#"[
    function quoteExactInputSingle(address tokenIn, address tokenOut,uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut)
]"#;);


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load rpc endpoint from local environment
    let rpc_endpoint = std::env::var("ETHEREUM_MAINNET_ENDPOINT")
        .expect("Could not get ETHEREUM_MAINNET_ENDPOINT");

    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    //Instantiate Pools and Quoter
    let mut pool = UniswapV3Pool::new_from_address(
        H160::from_str("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640").unwrap(),  // univ3 usdc/weth pool
        provider.clone(),
    )
    .await // use ? at end of await??
    .unwrap();

    pool.get_pool_data(provider.clone()).await.unwrap();

    let block_pool = IUniswapV3Pool::new(
        H160::from_str("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640").unwrap(),
        provider.clone(),
    );

    let current_block = provider.get_block_number().await.unwrap();

    let sqrt_price = block_pool.slot_0().block(current_block).call().await.unwrap().0;
    pool.sqrt_price = sqrt_price;

    let float_price_a = pool.calculate_price(pool.token_a);

    let float_price_b = pool.calculate_price(pool.token_b);

    dbg!(pool);

    println!("Current Block: {current_block}");
    println!("Price A: {float_price_a}");
    println!("Price B: {float_price_b}");



    Ok(())

}
