//Uniswap V3 Pool Data Batch request

pub use get_uniswap_v3_pool_data_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod get_uniswap_v3_pool_data_batch_request {
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
    #[doc = "GetUniswapV3PoolDataBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"from\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"step\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"factory\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static GETUNISWAPV3POOLDATABATCHREQUEST_ABI: ethers::contract::Lazy<
        ethers::core::abi::Abi,
    > = ethers::contract::Lazy::new(|| {
        ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    #[doc = r" Bytecode of the #name contract"]
    pub static GETUNISWAPV3POOLDATABATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b506040516104d23803806104d28339818101604052810190610032919061023e565b60008267ffffffffffffffff81111561004e5761004d610291565b5b60405190808252806020026020018201604052801561007c5781602001602082028036833780820191505090505b50905060005b83811015610171578273ffffffffffffffffffffffffffffffffffffffff16631e3dd18b82876100b291906102ef565b6040518263ffffffff1660e01b81526004016100ce9190610332565b6020604051808303816000875af11580156100ed573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610111919061034d565b8282815181106101245761012361037a565b5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff16815250508080610169906103a9565b915050610082565b5060008160405160200161018591906104af565b604051602081830303815290604052905060008151905081600052806000f35b600080fd5b6000819050919050565b6101bd816101aa565b81146101c857600080fd5b50565b6000815190506101da816101b4565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061020b826101e0565b9050919050565b61021b81610200565b811461022657600080fd5b50565b60008151905061023881610212565b92915050565b600080600060608486031215610257576102566101a5565b5b6000610265868287016101cb565b9350506020610276868287016101cb565b925050604061028786828701610229565b9150509250925092565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006102fa826101aa565b9150610305836101aa565b925082820190508082111561031d5761031c6102c0565b5b92915050565b61032c816101aa565b82525050565b60006020820190506103476000830184610323565b92915050565b600060208284031215610363576103626101a5565b5b600061037184828501610229565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60006103b4826101aa565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82036103e6576103e56102c0565b5b600182019050919050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b61042681610200565b82525050565b6000610438838361041d565b60208301905092915050565b6000602082019050919050565b600061045c826103f1565b61046681856103fc565b93506104718361040d565b8060005b838110156104a2578151610489888261042c565b975061049483610444565b925050600181019050610475565b5085935050505092915050565b600060208201905081810360008301526104c98184610451565b90509291505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct GetUniswapV3PoolDataBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for GetUniswapV3PoolDataBatchRequest<M> {
        fn clone(&self) -> Self {
            GetUniswapV3PoolDataBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for GetUniswapV3PoolDataBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for GetUniswapV3PoolDataBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(GetUniswapV3PoolDataBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> GetUniswapV3PoolDataBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                GETUNISWAPV3POOLDATABATCHREQUEST_ABI.clone(),
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
                GETUNISWAPV3POOLDATABATCHREQUEST_ABI.clone(),
                GETUNISWAPV3POOLDATABATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for GetUniswapV3PoolDataBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}

//Uniswap V3 Sync Pool Batch request
pub use sync_uniswap_v3_pool_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod sync_uniswap_v3_pool_batch_request {
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
    #[doc = "SyncUniswapV3PoolBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"from\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"step\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"factory\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static SYNCUNISWAPV3POOLBATCHREQUEST_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static SYNCUNISWAPV3POOLBATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b506040516104d23803806104d28339818101604052810190610032919061023e565b60008267ffffffffffffffff81111561004e5761004d610291565b5b60405190808252806020026020018201604052801561007c5781602001602082028036833780820191505090505b50905060005b83811015610171578273ffffffffffffffffffffffffffffffffffffffff16631e3dd18b82876100b291906102ef565b6040518263ffffffff1660e01b81526004016100ce9190610332565b6020604051808303816000875af11580156100ed573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610111919061034d565b8282815181106101245761012361037a565b5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff16815250508080610169906103a9565b915050610082565b5060008160405160200161018591906104af565b604051602081830303815290604052905060008151905081600052806000f35b600080fd5b6000819050919050565b6101bd816101aa565b81146101c857600080fd5b50565b6000815190506101da816101b4565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061020b826101e0565b9050919050565b61021b81610200565b811461022657600080fd5b50565b60008151905061023881610212565b92915050565b600080600060608486031215610257576102566101a5565b5b6000610265868287016101cb565b9350506020610276868287016101cb565b925050604061028786828701610229565b9150509250925092565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006102fa826101aa565b9150610305836101aa565b925082820190508082111561031d5761031c6102c0565b5b92915050565b61032c816101aa565b82525050565b60006020820190506103476000830184610323565b92915050565b600060208284031215610363576103626101a5565b5b600061037184828501610229565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60006103b4826101aa565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82036103e6576103e56102c0565b5b600182019050919050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b61042681610200565b82525050565b6000610438838361041d565b60208301905092915050565b6000602082019050919050565b600061045c826103f1565b61046681856103fc565b93506104718361040d565b8060005b838110156104a2578151610489888261042c565b975061049483610444565b925050600181019050610475565b5085935050505092915050565b600060208201905081810360008301526104c98184610451565b90509291505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct SyncUniswapV3PoolBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for SyncUniswapV3PoolBatchRequest<M> {
        fn clone(&self) -> Self {
            SyncUniswapV3PoolBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for SyncUniswapV3PoolBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for SyncUniswapV3PoolBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(SyncUniswapV3PoolBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> SyncUniswapV3PoolBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                SYNCUNISWAPV3POOLBATCHREQUEST_ABI.clone(),
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
                SYNCUNISWAPV3POOLBATCHREQUEST_ABI.clone(),
                SYNCUNISWAPV3POOLBATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for SyncUniswapV3PoolBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}
