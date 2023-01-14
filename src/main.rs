use ethers::prelude::Abigen;

fn main() {
    Abigen::new("SyncUniswapV2PoolBatchRequest", "./x.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("x.rs")
        .unwrap()
}
