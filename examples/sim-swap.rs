use std::{error::Error, str::FromStr, sync::Arc};

use ethers::{
    prelude::abigen,
    providers::{Http, Provider, Middleware},
    types::{U256, H160},
};

use cfmms::pool::UniswapV3Pool;

abigen!(
    IQuoter,
r#"[
    function quoteExactInputSingle(address tokenIn, address tokenOut,uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut)
]"#;);


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //load rpc endpoint from local environment
    // let rpc_endpoint = std::env::var("ETHEREUM_MAINNET_ENDPOINT")
    //     .expect("Could not get ETHEREUM_MAINNET_ENDPOINT");

    // let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());

    let provider = Arc::new(Provider::<Http>::try_from("https://eth.llamarpc.com").expect("cant load from endpoint"));
    
    //Instantiate Pools and Quoter
    let pool = UniswapV3Pool::new_from_address(
        H160::from_str("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640").unwrap(),  // univ3 usdc/weth pool
        provider.clone(),
    )
    .await // use ? at end of await??
    .unwrap();

    let quoter = IQuoter::new(
        H160::from_str("0xb27308f9f90d607463bb33ea1bebb41c27ce5ab6").unwrap(),
        provider.clone(),
    );

    let amount_in = U256::from_dec_str("1000000000000000000").unwrap(); // 1 WETH

    let current_block = provider.get_block_number().await.unwrap();
    let amount_out = pool
        .simulate_swap(pool.token_b, amount_in, provider.clone())
        .await
        .unwrap();

    let expected_amount_out = quoter
        .quote_exact_input_single(
            pool.token_b,
            pool.token_a,
            pool.fee,
            amount_in,
            U256::zero(),
        )
        .block(current_block)
        .call()
        .await
        .unwrap();

    assert_eq!(amount_out, expected_amount_out);

    println!(
        "Current block: {} Amount in: {} Amount out: {} Expected amount out: {}",
        current_block, amount_in, amount_out, expected_amount_out
    );

    Ok(())

}