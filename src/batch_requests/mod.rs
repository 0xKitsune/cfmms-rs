pub use get_all_uniswap_v2_pairs_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod get_all_uniswap_v2_pairs_batch_request {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "GetAllUniswapV2PairsBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"from\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"step\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"factory\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static GETALLUNISWAPV2PAIRSBATCHREQUEST_ABI: ethers::contract::Lazy<
        ethers::core::abi::Abi,
    > = ethers::contract::Lazy::new(|| {
        ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    #[doc = r" Bytecode of the #name contract"]
    pub static GETALLUNISWAPV2PAIRSBATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b5060405161029f38038061029f83398101604081905261002f91610186565b6000826001600160401b03811115610049576100496101bb565b604051908082528060200260200182016040528015610072578160200160208202803683370190505b50905060005b83811015610136576001600160a01b038316631e3dd18b61009983886101e7565b6040518263ffffffff1660e01b81526004016100b791815260200190565b6020604051808303816000875af11580156100d6573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906100fa9190610200565b82828151811061010c5761010c610222565b6001600160a01b03909216602092830291909101909101528061012e81610238565b915050610078565b5060008160405160200161014a9190610251565b604051602081830303815290604052905060008151905081600052806000f35b80516001600160a01b038116811461018157600080fd5b919050565b60008060006060848603121561019b57600080fd5b83519250602084015191506101b26040850161016a565b90509250925092565b634e487b7160e01b600052604160045260246000fd5b634e487b7160e01b600052601160045260246000fd5b808201808211156101fa576101fa6101d1565b92915050565b60006020828403121561021257600080fd5b61021b8261016a565b9392505050565b634e487b7160e01b600052603260045260246000fd5b60006001820161024a5761024a6101d1565b5060010190565b6020808252825182820181905260009190848201906040850190845b818110156102925783516001600160a01b03168352928401929184019160010161026d565b5090969550505050505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct GetAllUniswapV2PairsBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for GetAllUniswapV2PairsBatchRequest<M> {
        fn clone(&self) -> Self {
            GetAllUniswapV2PairsBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for GetAllUniswapV2PairsBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for GetAllUniswapV2PairsBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(GetAllUniswapV2PairsBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> GetAllUniswapV2PairsBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                GETALLUNISWAPV2PAIRSBATCHREQUEST_ABI.clone(),
                client,
            )
            .into()
        }
        #[doc = r" Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it."]
        #[doc = r" Returns a new instance of a deployer that returns an instance of this contract after sending the transaction"]
        #[doc = r""]
        #[doc = r" Notes:"]
        #[doc = r" 1. If there are no constructor arguments, you should pass `()` as the argument."]
        #[doc = r" 1. The default poll duration is 7 seconds."]
        #[doc = r" 1. The default number of confirmations is 1 block."]
        #[doc = r""]
        #[doc = r""]
        #[doc = r" # Example"]
        #[doc = r""]
        #[doc = r" Generate contract bindings with `abigen!` and deploy a new contract instance."]
        #[doc = r""]
        #[doc = r" *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact."]
        #[doc = r""]
        #[doc = r" ```ignore"]
        #[doc = r" # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {"]
        #[doc = r#"     abigen!(Greeter,"../greeter.json");"#]
        #[doc = r""]
        #[doc = r#"    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();"#]
        #[doc = r"    let msg = greeter_contract.greet().call().await.unwrap();"]
        #[doc = r" # }"]
        #[doc = r" ```"]
        pub fn deploy<T: ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ethers::contract::builders::ContractDeployer<M, Self>,
            ethers::contract::ContractError<M>,
        > {
            let factory = ethers::contract::ContractFactory::new(
                GETALLUNISWAPV2PAIRSBATCHREQUEST_ABI.clone(),
                GETALLUNISWAPV2PAIRSBATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for GetAllUniswapV2PairsBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}
