use core::panic;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::dex::DexType;

use super::dex::Dex;
use super::pair::Pair;
use ethers::{
    prelude::{abigen, ContractError},
    providers::{Http, Ipc, Middleware, Provider, ProviderError},
    types::{Address, BlockNumber, Filter, ValueOrArray, H160, U256, U64},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

abigen!(
    IUniswapV2Factory,
    r#"[
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256)
    ]"#;

    IUniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function token0() external view returns (address)
    ]"#;

    IUniswapV3Factory,
    r#"[
        event PoolCreated(address indexed token0, address indexed token1, uint24 indexed fee, int24 tickSpacing, address pool)
    ]"#;

    IUniswapV3Pool,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
        ]"#;

    IErc20,
    r#"[
        function balanceOf(address account) external view returns (uint256)
    ]"#;


);

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs(
    dexes: Vec<Dex>,
    provider_endpoint: &str,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a new http provider
    let provider: Provider<Http> = Provider::<Http>::try_from(provider_endpoint)
        .expect("Could not initialize the provider from the supplied endpoint.");

    let current_block = provider.get_block_number().await?;
    let async_provider = Arc::new(provider);

    let mut handles = vec![];

    //Initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    //For each dex supplied, get all pair created events and get reserve values
    for dex in dexes {
        let async_provider = async_provider.clone();

        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn(async move {
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Blocks")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs = get_all_pairs(
                dex,
                async_provider.clone(),
                BlockNumber::Number(current_block),
                progress_bar.clone(),
            )
            .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pairs")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs =
                get_pair_reserves(pairs, dex.factory_address, async_provider, progress_bar).await?;

            Ok::<_, ProviderError>(pairs)
        }));
    }

    //Aggregate the populated pairs from each thread
    let mut aggregated_pairs: Vec<Pair> = vec![];

    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!("Error when syncing pairs: {}", provider_error.to_string());
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the populated aggregated pairs vec
    Ok(aggregated_pairs)
}

//Function to get all pair created events for a given Dex factory address
async fn get_all_pairs(
    dex: Dex,
    provider: Arc<Provider<Http>>,
    current_block: BlockNumber,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Define the step for searching a range of blocks for pair created events
    let step = 100000;
    //Unwrap can be used here because the creation block was verified within `Dex::new()`
    let creation_block = dex.creation_block.as_number().unwrap().as_u64();
    let current_block = current_block.as_number().unwrap().as_u64();

    //Initialize the progress bar message
    progress_bar.set_length(current_block - creation_block);
    progress_bar.set_message(format!("Getting all pairs from: {}", dex.factory_address));

    //Init a new vec to keep track of tasks
    let mut handles = vec![];

    //For each block within the range, get all pairs asynchronously
    for from_block in (creation_block..=current_block).step_by(step) {
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new task to get pair created events from the block range
        handles.push(tokio::spawn(async move {
            let mut pairs = vec![];

            //Get pair created event logs within the block range
            let to_block = from_block + step as u64;
            let logs = provider
                .get_logs(
                    &Filter::new()
                        .topic0(ValueOrArray::Value(dex.pair_created_event_signature()))
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await?;

            //Increment the progres bar by the step
            progress_bar.inc(step as u64);

            //For each pair created log, create a new Pair type and add it to the pairs vec
            for log in logs {
                match dex.dex_type {
                    DexType::UniswapV2 => {
                        let uniswap_v2_factory =
                            IUniswapV2Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, pair_address, _) = match uniswap_v2_factory
                            .decode_event::<(Address, Address, Address, U256)>(
                                "PairCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV2,
                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee: 300,
                        })
                    }
                    DexType::UniswapV3 => {
                        let uniswap_v3_factory =
                            IUniswapV3Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, fee, _, pair_address) = match uniswap_v3_factory
                            .decode_event::<(Address, Address, u128, u128, Address)>(
                                "PoolCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV3,

                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee,
                        })
                    }
                }
            }

            Ok::<Vec<Pair>, ProviderError>(pairs)
        }));
    }

    //Wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    let mut aggregated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!(
                        "Error when getting Pair/Pool Created events: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }
    Ok(aggregated_pairs)
}

//Function to get reserves for each pair in the `pairs` vec.
async fn get_pair_reserves(
    pairs: Vec<Pair>,
    dex_factory_address: H160,
    provider: Arc<Provider<Http>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a vec to track each async task.
    let mut handles = vec![];

    //Initialize the progress bar message
    progress_bar.set_length(pairs.len() as u64);
    progress_bar.set_message(format!(
        "Syncing reserves for pairs from: {}",
        dex_factory_address
    ));

    //For each pair in the pairs vec, get the reserves asyncrhonously
    for mut pair in pairs {
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new thread to get the reserves for the pair
        handles.push(tokio::spawn(async move {
            progress_bar.inc(1);

            //Match the DexType and fetch the reserves for the pair.
            match pair.dex_type {
                DexType::UniswapV2 => {
                    //Initialize a new instance of the Pool
                    let v2_pair = IUniswapV2Pair::new(pair.pair_address, provider.clone());

                    // Make a call to get the reserves
                    let (reserve_0, reserve_1, _timestamp) =
                        match v2_pair.get_reserves().call().await {
                            Ok(result) => result,
                            Err(contract_error) => match contract_error {
                                ContractError::ProviderError(provider_error) => {
                                    return Err(provider_error)
                                }
                                _ => {
                                    return Ok(Pair::empty_pair(DexType::UniswapV2));
                                }
                            },
                        };

                    //set the pair reserves
                    pair.reserve_0 = reserve_0;
                    pair.reserve_1 = reserve_1;

                    // Make a call to get token0 to initialize a_to_b
                    let token0 = match v2_pair.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                progress_bar.inc(1);
                                return Ok(Pair::empty_pair(DexType::UniswapV2));
                            }
                        },
                    };

                    //Update a_to_b
                    pair.a_to_b = pair.token_a == token0;

                    Ok(pair)
                }
                DexType::UniswapV3 => {
                    //Initialize a new instance of the Pool
                    let v3_pool = IUniswapV3Pool::new(pair.pair_address, provider.clone());

                    // Make a call to get token0 and initialize a_to_b
                    let token0 = match v3_pool.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    pair.a_to_b = pair.token_a == token0;

                    //Initialize a new instance of token_a
                    let token_a = IErc20::new(pair.token_a, provider.clone());

                    // Make a call to get the Pool's balance of token_a
                    let reserve_0 = match token_a.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Initialize a new instance of token_b
                    let token_b = IErc20::new(pair.token_b, provider.clone());

                    // Make a call to get the Pool's balance of token_b
                    let reserve_1 = match token_b.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Set the pair reserves
                    pair.reserve_0 = reserve_0.as_u128();
                    pair.reserve_1 = reserve_1.as_u128();

                    Ok(pair)
                }
            }
        }));
    }

    //Create a new vec to aggregate the pairs and populate the vec.
    let mut updated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pair) => {
                    if !pair.is_empty() {
                        updated_pairs.push(pair)
                    }
                }
                Err(provider_error) => {
                    panic!(
                        "Error when getting pair reserves: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the vec of pairs with updated reserve values
    Ok(updated_pairs)
}

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs_with_ipc(
    dexes: Vec<Dex>,
    ipc_endpoint: &str,
    interval: Duration,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a new http provider
    let provider: Provider<Ipc> = Provider::connect_ipc(ipc_endpoint)
        .await?
        .interval(interval);

    let current_block = provider.get_block_number().await?;
    let async_provider = Arc::new(provider);

    let mut handles = vec![];

    //Initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    //For each dex supplied, get all pair created events and get reserve values
    for dex in dexes {
        let async_provider = async_provider.clone();

        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn(async move {
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Blocks")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs = get_all_pairs_with_ipc(
                dex,
                async_provider.clone(),
                BlockNumber::Number(current_block),
                progress_bar.clone(),
            )
            .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pairs")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs = get_pair_reserves_with_ipc(
                pairs,
                dex.factory_address,
                async_provider,
                progress_bar,
            )
            .await?;

            Ok::<_, ProviderError>(pairs)
        }));
    }

    //Aggregate the populated pairs from each thread
    let mut aggregated_pairs: Vec<Pair> = vec![];

    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!("Error when syncing pairs: {}", provider_error.to_string());
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the populated aggregated pairs vec
    Ok(aggregated_pairs)
}

//Function to get all pair created events for a given Dex factory address
async fn get_all_pairs_with_ipc(
    dex: Dex,
    provider: Arc<Provider<Ipc>>,
    current_block: BlockNumber,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Define the step for searching a range of blocks for pair created events
    let step = 100000;
    //Unwrap can be used here because the creation block was verified within `Dex::new()`
    let creation_block = dex.creation_block.as_number().unwrap().as_u64();
    let current_block = current_block.as_number().unwrap().as_u64();

    //Initialize the progress bar message
    progress_bar.set_length(current_block - creation_block);
    progress_bar.set_message(format!("Getting all pairs from: {}", dex.factory_address));

    //Init a new vec to keep track of tasks
    let mut handles = vec![];

    //For each block within the range, get all pairs asynchronously
    for from_block in (creation_block..=current_block).step_by(step) {
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new task to get pair created events from the block range
        handles.push(tokio::spawn(async move {
            let mut pairs = vec![];

            //Get pair created event logs within the block range
            let to_block = from_block + step as u64;
            let logs = provider
                .get_logs(
                    &Filter::new()
                        .topic0(ValueOrArray::Value(dex.pair_created_event_signature()))
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await?;

            //Increment the progres bar by the step
            progress_bar.inc(step as u64);

            //For each pair created log, create a new Pair type and add it to the pairs vec
            for log in logs {
                match dex.dex_type {
                    DexType::UniswapV2 => {
                        let uniswap_v2_factory =
                            IUniswapV2Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, pair_address, _) = match uniswap_v2_factory
                            .decode_event::<(Address, Address, Address, U256)>(
                                "PairCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV2,
                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee: 300,
                        })
                    }
                    DexType::UniswapV3 => {
                        let uniswap_v3_factory =
                            IUniswapV3Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, fee, _, pair_address) = match uniswap_v3_factory
                            .decode_event::<(Address, Address, u128, u128, Address)>(
                                "PoolCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV3,

                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee,
                        })
                    }
                }
            }

            Ok::<Vec<Pair>, ProviderError>(pairs)
        }));
    }

    //Wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    let mut aggregated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!(
                        "Error when getting Pair/Pool Created events: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }
    Ok(aggregated_pairs)
}

//Function to get reserves for each pair in the `pairs` vec.
async fn get_pair_reserves_with_ipc(
    pairs: Vec<Pair>,
    dex_factory_address: H160,
    provider: Arc<Provider<Ipc>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a vec to track each async task.
    let mut handles = vec![];

    //Initialize the progress bar message
    progress_bar.set_length(pairs.len() as u64);
    progress_bar.set_message(format!(
        "Syncing reserves for pairs from: {}",
        dex_factory_address
    ));

    //For each pair in the pairs vec, get the reserves asyncrhonously
    for mut pair in pairs {
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new thread to get the reserves for the pair
        handles.push(tokio::spawn(async move {
            progress_bar.inc(1);

            //Match the DexType and fetch the reserves for the pair.
            match pair.dex_type {
                DexType::UniswapV2 => {
                    //Initialize a new instance of the Pool
                    let v2_pair = IUniswapV2Pair::new(pair.pair_address, provider.clone());

                    // Make a call to get the reserves
                    let (reserve_0, reserve_1, _timestamp) =
                        match v2_pair.get_reserves().call().await {
                            Ok(result) => result,
                            Err(contract_error) => match contract_error {
                                ContractError::ProviderError(provider_error) => {
                                    return Err(provider_error)
                                }
                                _ => {
                                    return Ok(Pair::empty_pair(DexType::UniswapV2));
                                }
                            },
                        };

                    //set the pair reserves
                    pair.reserve_0 = reserve_0;
                    pair.reserve_1 = reserve_1;

                    // Make a call to get token0 to initialize a_to_b
                    let token0 = match v2_pair.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                progress_bar.inc(1);
                                return Ok(Pair::empty_pair(DexType::UniswapV2));
                            }
                        },
                    };

                    //Update a_to_b
                    pair.a_to_b = pair.token_a == token0;

                    Ok(pair)
                }
                DexType::UniswapV3 => {
                    //Initialize a new instance of the Pool
                    let v3_pool = IUniswapV3Pool::new(pair.pair_address, provider.clone());

                    // Make a call to get token0 and initialize a_to_b
                    let token0 = match v3_pool.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    pair.a_to_b = pair.token_a == token0;

                    //Initialize a new instance of token_a
                    let token_a = IErc20::new(pair.token_a, provider.clone());

                    // Make a call to get the Pool's balance of token_a
                    let reserve_0 = match token_a.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Initialize a new instance of token_b
                    let token_b = IErc20::new(pair.token_b, provider.clone());

                    // Make a call to get the Pool's balance of token_b
                    let reserve_1 = match token_b.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Set the pair reserves
                    pair.reserve_0 = reserve_0.as_u128();
                    pair.reserve_1 = reserve_1.as_u128();

                    Ok(pair)
                }
            }
        }));
    }

    //Create a new vec to aggregate the pairs and populate the vec.
    let mut updated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pair) => {
                    if !pair.is_empty() {
                        updated_pairs.push(pair)
                    }
                }
                Err(provider_error) => {
                    panic!(
                        "Error when getting pair reserves: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the vec of pairs with updated reserve values
    Ok(updated_pairs)
}

use super::throttle::RequestThrottle;
//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs_with_throttle(
    dexes: Vec<Dex>,
    provider_endpoint: &str,
    requests_per_second_limit: usize,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a new http provider

    let provider: Provider<Http> = Provider::<Http>::try_from(provider_endpoint)
        .expect("Could not initialize the provider from the supplied endpoint.");

    //Initalize a new request throttle
    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));

    let current_block = provider.get_block_number().await?;
    let async_provider = Arc::new(provider);

    let mut handles = vec![];

    //Initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    //For each dex supplied, get all pair created events and get reserve values
    for dex in dexes {
        let async_provider = async_provider.clone();
        let request_throttle = request_throttle.clone();
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn(async move {
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Blocks")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs = get_all_pairs_with_throttle(
                dex,
                async_provider.clone(),
                BlockNumber::Number(current_block),
                request_throttle.clone(),
                progress_bar.clone(),
            )
            .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pairs")
                    .unwrap()
                    .progress_chars("##-"),
            );

            let pairs = get_pair_reserves_with_throttle(
                pairs,
                dex.factory_address,
                async_provider,
                request_throttle,
                progress_bar,
            )
            .await?;

            Ok::<_, ProviderError>(pairs)
        }));
    }

    //Aggregate the populated pairs from each thread
    let mut aggregated_pairs: Vec<Pair> = vec![];

    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!("Error when syncing pairs: {}", provider_error.to_string());
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the populated aggregated pairs vec
    Ok(aggregated_pairs)
}

//Function to get all pair created events for a given Dex factory address
async fn get_all_pairs_with_throttle(
    dex: Dex,
    provider: Arc<Provider<Http>>,
    current_block: BlockNumber,
    request_throttle: Arc<Mutex<RequestThrottle>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Define the step for searching a range of blocks for pair created events
    let step = 100000;
    //Unwrap can be used here because the creation block was verified within `Dex::new()`
    let creation_block = dex.creation_block.as_number().unwrap().as_u64();
    let current_block = current_block.as_number().unwrap().as_u64();

    //Initialize the progress bar message
    progress_bar.set_length(current_block - creation_block);
    progress_bar.set_message(format!("Getting all pairs from: {}", dex.factory_address));

    //Init a new vec to keep track of tasks
    let mut handles = vec![];

    //For each block within the range, get all pairs asynchronously
    for from_block in (creation_block..=current_block).step_by(step) {
        let request_throttle = request_throttle.clone();
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new task to get pair created events from the block range
        handles.push(tokio::spawn(async move {
            let mut pairs = vec![];

            //Update the throttle
            request_throttle.lock().unwrap().increment_or_sleep();

            //Get pair created event logs within the block range
            let to_block = from_block + step as u64;
            let logs = provider
                .get_logs(
                    &Filter::new()
                        .topic0(ValueOrArray::Value(dex.pair_created_event_signature()))
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await?;

            //Increment the progres bar by the step
            progress_bar.inc(step as u64);

            //For each pair created log, create a new Pair type and add it to the pairs vec
            for log in logs {
                match dex.dex_type {
                    DexType::UniswapV2 => {
                        let uniswap_v2_factory =
                            IUniswapV2Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, pair_address, _) = match uniswap_v2_factory
                            .decode_event::<(Address, Address, Address, U256)>(
                                "PairCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV2,
                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee: 300,
                        })
                    }
                    DexType::UniswapV3 => {
                        let uniswap_v3_factory =
                            IUniswapV3Factory::new(dex.factory_address, provider.clone());

                        let (token_a, token_b, fee, _, pair_address) = match uniswap_v3_factory
                            .decode_event::<(Address, Address, u128, u128, Address)>(
                                "PoolCreated",
                                log.topics,
                                log.data,
                            ) {
                            Ok(result) => result,
                            Err(_) => {
                                //If there was an abi error, continue without adding the pair
                                continue;
                            }
                        };

                        pairs.push(Pair {
                            dex_type: DexType::UniswapV3,

                            pair_address,
                            token_a,
                            token_b,
                            //Initialize the following variables as zero values
                            //They will be populated when getting pair reserves
                            a_to_b: false,
                            reserve_0: 0,
                            reserve_1: 0,
                            fee,
                        })
                    }
                }
            }

            Ok::<Vec<Pair>, ProviderError>(pairs)
        }));
    }

    //Wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    let mut aggregated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pairs) => aggregated_pairs.extend(pairs),
                Err(provider_error) => {
                    panic!(
                        "Error when getting Pair/Pool Created events: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }
    Ok(aggregated_pairs)
}

//Function to get reserves for each pair in the `pairs` vec.
async fn get_pair_reserves_with_throttle(
    pairs: Vec<Pair>,
    dex_factory_address: H160,
    provider: Arc<Provider<Http>>,
    request_throttle: Arc<Mutex<RequestThrottle>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pair>, ProviderError> {
    //Initialize a vec to track each async task.
    let mut handles = vec![];

    //Initialize the progress bar message
    progress_bar.set_length(pairs.len() as u64);
    progress_bar.set_message(format!(
        "Syncing reserves for pairs from: {}",
        dex_factory_address
    ));

    //For each pair in the pairs vec, get the reserves asyncrhonously
    for mut pair in pairs {
        let request_throttle = request_throttle.clone();
        let provider = provider.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new thread to get the reserves for the pair
        handles.push(tokio::spawn(async move {
            progress_bar.inc(1);

            //Match the DexType and fetch the reserves for the pair.
            match pair.dex_type {
                DexType::UniswapV2 => {
                    //Initialize a new instance of the Pool
                    let v2_pair = IUniswapV2Pair::new(pair.pair_address, provider.clone());

                    request_throttle.lock().unwrap().increment_or_sleep();
                    // Make a call to get the reserves
                    let (reserve_0, reserve_1, _timestamp) =
                        match v2_pair.get_reserves().call().await {
                            Ok(result) => result,
                            Err(contract_error) => match contract_error {
                                ContractError::ProviderError(provider_error) => {
                                    return Err(provider_error)
                                }
                                _ => {
                                    return Ok(Pair::empty_pair(DexType::UniswapV2));
                                }
                            },
                        };

                    //set the pair reserves
                    pair.reserve_0 = reserve_0;
                    pair.reserve_1 = reserve_1;
                    request_throttle.lock().unwrap().increment_or_sleep();
                    // Make a call to get token0 to initialize a_to_b
                    let token0 = match v2_pair.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                progress_bar.inc(1);
                                return Ok(Pair::empty_pair(DexType::UniswapV2));
                            }
                        },
                    };

                    //Update a_to_b
                    pair.a_to_b = pair.token_a == token0;

                    Ok(pair)
                }
                DexType::UniswapV3 => {
                    //Initialize a new instance of the Pool
                    let v3_pool = IUniswapV3Pool::new(pair.pair_address, provider.clone());

                    request_throttle.lock().unwrap().increment_or_sleep();
                    // Make a call to get token0 and initialize a_to_b
                    let token0 = match v3_pool.token_0().call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    pair.a_to_b = pair.token_a == token0;

                    //Initialize a new instance of token_a
                    let token_a = IErc20::new(pair.token_a, provider.clone());

                    request_throttle.lock().unwrap().increment_or_sleep();
                    // Make a call to get the Pool's balance of token_a
                    let reserve_0 = match token_a.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Initialize a new instance of token_b
                    let token_b = IErc20::new(pair.token_b, provider.clone());

                    request_throttle.lock().unwrap().increment_or_sleep();
                    // Make a call to get the Pool's balance of token_b
                    let reserve_1 = match token_b.balance_of(pair.pair_address).call().await {
                        Ok(result) => result,
                        Err(contract_error) => match contract_error {
                            ContractError::ProviderError(provider_error) => {
                                return Err(provider_error)
                            }
                            _ => {
                                return Ok(Pair::empty_pair(DexType::UniswapV3));
                            }
                        },
                    };

                    //Set the pair reserves
                    pair.reserve_0 = reserve_0.as_u128();
                    pair.reserve_1 = reserve_1.as_u128();

                    Ok(pair)
                }
            }
        }));
    }

    //Create a new vec to aggregate the pairs and populate the vec.
    let mut updated_pairs: Vec<Pair> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => match sync_result {
                Ok(pair) => {
                    if !pair.is_empty() {
                        updated_pairs.push(pair)
                    }
                }
                Err(provider_error) => {
                    panic!(
                        "Error when getting pair reserves: {}",
                        provider_error.to_string()
                    );
                }
            },

            Err(join_error) => {
                panic!("Error when joining handles: {}", join_error.to_string());
            }
        }
    }

    //Return the vec of pairs with updated reserve values
    Ok(updated_pairs)
}
