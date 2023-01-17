//Uniswap V2 Get Parirs Batch request

pub use get_uniswap_v2_pairs_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod get_uniswap_v2_pairs_batch_request {
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
    #[doc = "GetUniswapV2PairsBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"from\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"step\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"factory\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static GETUNISWAPV2PAIRSBATCHREQUEST_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static GETUNISWAPV2PAIRSBATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b506040516104d23803806104d28339818101604052810190610032919061023e565b60008267ffffffffffffffff81111561004e5761004d610291565b5b60405190808252806020026020018201604052801561007c5781602001602082028036833780820191505090505b50905060005b83811015610171578273ffffffffffffffffffffffffffffffffffffffff16631e3dd18b82876100b291906102ef565b6040518263ffffffff1660e01b81526004016100ce9190610332565b6020604051808303816000875af11580156100ed573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610111919061034d565b8282815181106101245761012361037a565b5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff16815250508080610169906103a9565b915050610082565b5060008160405160200161018591906104af565b604051602081830303815290604052905060008151905081600052806000f35b600080fd5b6000819050919050565b6101bd816101aa565b81146101c857600080fd5b50565b6000815190506101da816101b4565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061020b826101e0565b9050919050565b61021b81610200565b811461022657600080fd5b50565b60008151905061023881610212565b92915050565b600080600060608486031215610257576102566101a5565b5b6000610265868287016101cb565b9350506020610276868287016101cb565b925050604061028786828701610229565b9150509250925092565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006102fa826101aa565b9150610305836101aa565b925082820190508082111561031d5761031c6102c0565b5b92915050565b61032c816101aa565b82525050565b60006020820190506103476000830184610323565b92915050565b600060208284031215610363576103626101a5565b5b600061037184828501610229565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60006103b4826101aa565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82036103e6576103e56102c0565b5b600182019050919050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b61042681610200565b82525050565b6000610438838361041d565b60208301905092915050565b6000602082019050919050565b600061045c826103f1565b61046681856103fc565b93506104718361040d565b8060005b838110156104a2578151610489888261042c565b975061049483610444565b925050600181019050610475565b5085935050505092915050565b600060208201905081810360008301526104c98184610451565b90509291505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct GetUniswapV2PairsBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for GetUniswapV2PairsBatchRequest<M> {
        fn clone(&self) -> Self {
            GetUniswapV2PairsBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for GetUniswapV2PairsBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for GetUniswapV2PairsBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(GetUniswapV2PairsBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> GetUniswapV2PairsBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                GETUNISWAPV2PAIRSBATCHREQUEST_ABI.clone(),
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
                GETUNISWAPV2PAIRSBATCHREQUEST_ABI.clone(),
                GETUNISWAPV2PAIRSBATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for GetUniswapV2PairsBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}

//Uniswap V2 Pool Data Batch request

pub use get_uniswap_v2_pool_data_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod get_uniswap_v2_pool_data_batch_request {
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
    #[doc = "GetUniswapV2PoolDataBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"pools\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static GETUNISWAPV2POOLDATABATCHREQUEST_ABI: ethers::contract::Lazy<
        ethers::core::abi::Abi,
    > = ethers::contract::Lazy::new(|| {
        ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    #[doc = r" Bytecode of the #name contract"]
    pub static GETUNISWAPV2POOLDATABATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b5060405161061438038061061483398101604081905261002f916103f0565b600081516001600160401b0381111561004a5761004a6103be565b6040519080825280602002602001820160405280156100aa57816020015b6040805160c08101825260008082526020808301829052928201819052606082018190526080820181905260a082015282526000199092019101816100685790505b50905060005b825181101561038a576040805160c081018252600080825260208201819052918101829052606081018290526080810182905260a08101919091528382815181106100fd576100fd6104b4565b60200260200101516001600160a01b0316630dfe16816040518163ffffffff1660e01b8152600401602060405180830381865afa158015610142573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061016691906104ca565b6001600160a01b03168082526040805163313ce56760e01b8152905163313ce567916004808201926020929091908290030181865afa1580156101ad573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906101d191906104ec565b60ff16602082015283518490839081106101ed576101ed6104b4565b60200260200101516001600160a01b031663d21220a76040518163ffffffff1660e01b8152600401602060405180830381865afa158015610232573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061025691906104ca565b6001600160a01b03166040808301829052805163313ce56760e01b8152905163313ce567916004808201926020929091908290030181865afa1580156102a0573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906102c491906104ec565b60ff16606082015283518490839081106102e0576102e06104b4565b60200260200101516001600160a01b0316630902f1ac6040518163ffffffff1660e01b8152600401606060405180830381865afa158015610325573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906103499190610526565b506001600160701b0390811660a084015216608082015282518190849084908110610376576103766104b4565b6020908102919091010152506001016100b0565b5060008160405160200161039e9190610576565b604051602081830303815290604052905060008151905081600052806000f35b634e487b7160e01b600052604160045260246000fd5b80516001600160a01b03811681146103eb57600080fd5b919050565b6000602080838503121561040357600080fd5b82516001600160401b038082111561041a57600080fd5b818501915085601f83011261042e57600080fd5b815181811115610440576104406103be565b8060051b604051601f19603f83011681018181108582111715610465576104656103be565b60405291825284820192508381018501918883111561048357600080fd5b938501935b828510156104a857610499856103d4565b84529385019392850192610488565b98975050505050505050565b634e487b7160e01b600052603260045260246000fd5b6000602082840312156104dc57600080fd5b6104e5826103d4565b9392505050565b6000602082840312156104fe57600080fd5b815160ff811681146104e557600080fd5b80516001600160701b03811681146103eb57600080fd5b60008060006060848603121561053b57600080fd5b6105448461050f565b92506105526020850161050f565b9150604084015163ffffffff8116811461056b57600080fd5b809150509250925092565b602080825282518282018190526000919060409081850190868401855b8281101561060657815180516001600160a01b0390811686528782015160ff908116898801528783015190911687870152606080830151909116908601526080808201516001600160701b039081169187019190915260a091820151169085015260c09093019290850190600101610593565b509197965050505050505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct GetUniswapV2PoolDataBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for GetUniswapV2PoolDataBatchRequest<M> {
        fn clone(&self) -> Self {
            GetUniswapV2PoolDataBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for GetUniswapV2PoolDataBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for GetUniswapV2PoolDataBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(GetUniswapV2PoolDataBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> GetUniswapV2PoolDataBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                GETUNISWAPV2POOLDATABATCHREQUEST_ABI.clone(),
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
                GETUNISWAPV2POOLDATABATCHREQUEST_ABI.clone(),
                GETUNISWAPV2POOLDATABATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for GetUniswapV2PoolDataBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}

//Uniswap V2 Sync Pool Batch request

pub use sync_uniswap_v2_pool_batch_request::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod sync_uniswap_v2_pool_batch_request {
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
    #[doc = "SyncUniswapV2PoolBatchRequest was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"pools\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static SYNCUNISWAPV2POOLBATCHREQUEST_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static SYNCUNISWAPV2POOLBATCHREQUEST_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b5060405161036f38038061036f83398101604081905261002f916101d1565b600081516001600160401b0381111561004a5761004a61019f565b60405190808252806020026020018201604052801561008f57816020015b60408051808201909152600080825260208201528152602001906001900390816100685790505b50905060005b825181101561016b5760408051808201909152600080825260208201528382815181106100c4576100c4610295565b60200260200101516001600160a01b0316630902f1ac6040518163ffffffff1660e01b8152600401606060405180830381865afa158015610109573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061012d91906102c2565b506001600160701b0390811660208401521681528251819084908490811061015757610157610295565b602090810291909101015250600101610095565b5060008160405160200161017f9190610312565b604051602081830303815290604052905060008151905081600052806000f35b634e487b7160e01b600052604160045260246000fd5b80516001600160a01b03811681146101cc57600080fd5b919050565b600060208083850312156101e457600080fd5b82516001600160401b03808211156101fb57600080fd5b818501915085601f83011261020f57600080fd5b8151818111156102215761022161019f565b8060051b604051601f19603f830116810181811085821117156102465761024661019f565b60405291825284820192508381018501918883111561026457600080fd5b938501935b828510156102895761027a856101b5565b84529385019392850192610269565b98975050505050505050565b634e487b7160e01b600052603260045260246000fd5b80516001600160701b03811681146101cc57600080fd5b6000806000606084860312156102d757600080fd5b6102e0846102ab565b92506102ee602085016102ab565b9150604084015163ffffffff8116811461030757600080fd5b809150509250925092565b602080825282518282018190526000919060409081850190868401855b8281101561036157815180516001600160701b039081168652908701511686850152928401929085019060010161032f565b509197965050505050505056fe" . parse () . expect ("invalid bytecode")
    });
    pub struct SyncUniswapV2PoolBatchRequest<M>(ethers::contract::Contract<M>);
    impl<M> Clone for SyncUniswapV2PoolBatchRequest<M> {
        fn clone(&self) -> Self {
            SyncUniswapV2PoolBatchRequest(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for SyncUniswapV2PoolBatchRequest<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for SyncUniswapV2PoolBatchRequest<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(SyncUniswapV2PoolBatchRequest))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> SyncUniswapV2PoolBatchRequest<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                SYNCUNISWAPV2POOLBATCHREQUEST_ABI.clone(),
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
                SYNCUNISWAPV2POOLBATCHREQUEST_ABI.clone(),
                SYNCUNISWAPV2POOLBATCHREQUEST_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for SyncUniswapV2PoolBatchRequest<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
}
