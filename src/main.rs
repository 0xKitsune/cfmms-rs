use ethers::providers::{Middleware, Provider};

fn main() {
    let provider = Arc::new(Provider::<Http>::try_from(rpc_endpoint).unwrap());
}

fn quick_test<M: Middleware>(x: Arc<M>) {}
