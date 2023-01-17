use ethers::prelude::Abigen;

fn main() {
    Abigen::new("SyncUniswapV3PoolBatchRequest", "./x.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("x.rs")
        .unwrap()
}
