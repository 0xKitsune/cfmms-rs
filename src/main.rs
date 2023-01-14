use ethers::prelude::Abigen;

fn main() {
    Abigen::new("GetAllUniswapV2PairsBatchRequest", "./x.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("token.rs")
        .unwrap()
}
