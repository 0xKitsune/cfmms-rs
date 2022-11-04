use crate::dex::Dex;
use crate::error::PairSyncError;
use crate::pool::{Pool, PoolVariant};
use crate::throttle::RequestThrottle;
use ethers::providers::{JsonRpcClient, Provider};
use ethers::types::H160;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Mutex;
use std::{collections::HashSet, sync::Arc};

//Filters out pools where the blacklisted address is the token_a address or token_b address
pub fn filter_blacklisted_tokens(pools: Vec<Pool>, blacklisted_addresses: Vec<H160>) -> Vec<Pool> {
    let mut filtered_pools = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pool in pools {
        if !blacklist.contains(&pool.token_a) || !blacklist.contains(&pool.token_b) {
            filtered_pools.push(pool);
        }
    }

    filtered_pools
}

//Filters out pools where the blacklisted address is the pair address
pub fn filter_blacklisted_pools(pools: Vec<Pool>, blacklisted_addresses: Vec<H160>) -> Vec<Pool> {
    let mut filtered_pools = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pool in pools {
        if !blacklist.contains(&pool.address) {
            filtered_pools.push(pool);
        }
    }

    filtered_pools
}

//Filters out pools where the blacklisted address is the pair address, token_a address or token_b address
pub fn filter_blacklisted_addresses(
    pools: Vec<Pool>,
    blacklisted_addresses: Vec<H160>,
) -> Vec<Pool> {
    let mut filtered_pools = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pool in pools {
        if !blacklist.contains(&pool.address)
            || !blacklist.contains(&pool.token_a)
            || !blacklist.contains(&pool.token_b)
        {
            filtered_pools.push(pool);
        }
    }

    filtered_pools
}

//Filter that removes pools with that contain less than a specified usd value
pub async fn filter_pools_below_usd_threshold<P: 'static + JsonRpcClient>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    usd_weth_pool: Pool,
    weth_address: H160,
    usd_threshold: f64,
    provider: Arc<Provider<P>>,
) -> Result<Vec<Pool>, PairSyncError<P>> {
    filter_pools_below_usd_threshold_with_throttle(
        pools,
        dexes,
        usd_weth_pool,
        weth_address,
        usd_threshold,
        provider,
        0,
    )
    .await
}

//Filter that removes pools with that contain less than a specified usd value
pub async fn filter_pools_below_usd_threshold_with_throttle<P: 'static + JsonRpcClient>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    usd_weth_pool: Pool,
    weth_address: H160,
    usd_threshold: f64,
    provider: Arc<Provider<P>>,
    requests_per_second_limit: usize,
) -> Result<Vec<Pool>, PairSyncError<P>> {
    let multi_progress_bar = MultiProgress::new();
    let progress_bar = multi_progress_bar.add(ProgressBar::new(0));
    progress_bar.set_style(
        ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pools Filtered")
            .unwrap()
            .progress_chars("##-"),
    );

    progress_bar.set_length(pools.len() as u64);
    progress_bar.set_message("Filtering pools: ");

    //Init a new vec to hold the filtered pools
    let mut filtered_pools = vec![];

    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));

    //Get price of weth in USD
    let usd_price_per_weth = usd_weth_pool
        .get_price(usd_weth_pool.a_to_b, provider.clone())
        .await?;

    //Initialize a Hashmap to keep track of token/weth prices already found to avoid unnecessary calls to the node
    let token_weth_prices: Arc<Mutex<HashMap<H160, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    //For each pool, check if the usd value meets the specified threshold
    for pool in pools {
        let token_weth_prices = token_weth_prices.clone();
        let request_throttle = request_throttle.clone();
        let provider = provider.clone();
        let dexes = dexes.clone();
        let progress_bar = progress_bar.clone();

        handles.push(tokio::spawn(async move {
            let (token_a_reserves, token_b_reserves) = if pool.a_to_b {
                (pool.reserve_0, pool.reserve_1)
            } else {
                (pool.reserve_1, pool.reserve_0)
            };

            let token_a_price_per_weth = token_weth_prices
                .lock()
                .unwrap()
                .get(&pool.token_a)
                .map(|price| price.to_owned());

            progress_bar.inc(1);

            let token_a_price_per_weth = match token_a_price_per_weth {
                Some(price) => price,
                None => {
                    request_throttle.lock().unwrap().increment_or_sleep(1);
                    let price = get_price_of_token_per_weth(
                        pool.token_a,
                        weth_address,
                        &dexes,
                        provider.clone(),
                    )
                    .await?;

                    token_weth_prices
                        .lock()
                        .unwrap()
                        .insert(pool.token_a, price);

                    price
                }
            };

            //Get weth value of token a in pool
            let token_a_weth_value_in_pool = token_a_reserves as f64
                / 10f64.powf(pool.token_a_decimals.into())
                / token_a_price_per_weth;

            //Calculate token_a usd value
            let token_a_usd_value_in_pool = token_a_weth_value_in_pool * usd_price_per_weth;

            let token_b_price_per_weth = token_weth_prices
                .lock()
                .unwrap()
                .get(&pool.token_b)
                .map(|price| price.to_owned());

            let token_b_price_per_weth = match token_b_price_per_weth {
                Some(price) => price.to_owned(),
                None => {
                    request_throttle.lock().unwrap().increment_or_sleep(1);
                    let price = get_price_of_token_per_weth(
                        pool.token_b,
                        weth_address,
                        &dexes,
                        provider.clone(),
                    )
                    .await?;

                    token_weth_prices
                        .lock()
                        .unwrap()
                        .insert(pool.token_b, price);

                    price
                }
            };

            //Get weth value of token a in pool
            let token_b_weth_value_in_pool = token_b_reserves as f64
                * 10f64.powf(pool.token_b_decimals.into())
                / token_b_price_per_weth;

            //Calculate token_b usd value
            let token_b_usd_value_in_pool = token_b_weth_value_in_pool * usd_price_per_weth;

            //Compare the sum of token_a and token_b usd value against the specified threshold
            let total_usd_value_in_pool = token_a_usd_value_in_pool + token_b_usd_value_in_pool;

            Ok::<_, PairSyncError<P>>((total_usd_value_in_pool, pool))
        }));
    }

    for handle in handles {
        match handle.await {
            Ok(filter_result) => match filter_result {
                Ok((total_usd_value_in_pool, pool)) => {
                    if usd_threshold <= total_usd_value_in_pool {
                        filtered_pools.push(pool);
                    }
                }
                Err(pair_sync_error) => match pair_sync_error {
                    PairSyncError::PairDoesNotExistInDexes(_, _) => {}
                    _ => return Err(pair_sync_error),
                },
            },

            Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
        }
    }

    Ok(filtered_pools)
}

async fn get_price_of_token_per_weth<P: 'static + JsonRpcClient>(
    token_address: H160,
    weth_address: H160,
    dexes: &Vec<Dex>,
    provider: Arc<Provider<P>>,
) -> Result<f64, PairSyncError<P>> {
    if token_address == weth_address {
        return Ok(1.0);
    }

    //Get token_a/weth price
    let token_a_weth_pool =
        get_token_to_weth_pool(token_address, weth_address, dexes, provider.clone()).await?;

    let token_a_price_per_weth = token_a_weth_pool
        .get_price(token_a_weth_pool.token_a == weth_address, provider.clone())
        .await?;

    Ok(token_a_price_per_weth)
}

//Gets the best token to weth pairing from the dexes provided
async fn get_token_to_weth_pool<P: 'static + JsonRpcClient>(
    token_a: H160,
    weth_address: H160,
    dexes: &Vec<Dex>,
    provider: Arc<Provider<P>>,
) -> Result<Pool, PairSyncError<P>> {
    let mut token_a_weth_pool = Pool::empty_pool(PoolVariant::UniswapV2);

    for dex in dexes {
        (token_a_weth_pool.address, token_a_weth_pool.fee) = dex
            .get_pool_with_best_liquidity(token_a, weth_address, provider.clone())
            .await?;

        if !token_a_weth_pool.is_empty() {
            break;
        }
    }

    if !token_a_weth_pool.is_empty() {
        token_a_weth_pool.update_a_to_b(provider.clone()).await?;
        token_a_weth_pool.update_reserves(provider).await?;
    } else {
        return Err(PairSyncError::PairDoesNotExistInDexes(
            token_a,
            weth_address,
        ));
    }

    Ok(token_a_weth_pool)
}

//Filter that removes pools with that contain less than a specified weth value
//
pub async fn filter_pools_below_weth_threshold<P: 'static + JsonRpcClient>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    weth_address: H160,
    weth_threshold: f64,
    provider: Arc<Provider<P>>,
) -> Result<Vec<Pool>, PairSyncError<P>> {
    filter_pools_below_weth_threshold_with_throttle(
        pools,
        dexes,
        weth_address,
        weth_threshold,
        provider,
        0,
    )
    .await
}

pub async fn filter_pools_below_weth_threshold_with_throttle<P: 'static + JsonRpcClient>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    weth_address: H160,
    weth_threshold: f64,
    provider: Arc<Provider<P>>,
    requests_per_second_limit: usize,
) -> Result<Vec<Pool>, PairSyncError<P>> {
    //TODO: add progress bar

    //Init a new vec to hold the filtered pools
    let mut filtered_pools = vec![];

    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));

    //Initialize a Hashmap to keep track of token/weth prices already found to avoid unnecessary calls to the node
    let token_weth_prices: Arc<Mutex<HashMap<H160, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    //For each pool, check if the usd value meets the specified threshold
    for pool in pools {
        let token_weth_prices = token_weth_prices.clone();
        let request_throttle = request_throttle.clone();
        let provider = provider.clone();
        let dexes = dexes.clone();

        handles.push(tokio::spawn(async move {
            let (token_a_reserves, token_b_reserves) = if pool.a_to_b {
                (pool.reserve_0, pool.reserve_1)
            } else {
                (pool.reserve_1, pool.reserve_0)
            };

            let token_a_price_per_weth = token_weth_prices
                .lock()
                .unwrap()
                .get(&pool.token_a)
                .map(|price| price.to_owned());

            let token_a_price_per_weth = match token_a_price_per_weth {
                Some(price) => price,
                None => {
                    request_throttle.lock().unwrap().increment_or_sleep(1);
                    let price = get_price_of_token_per_weth(
                        pool.token_a,
                        weth_address,
                        &dexes,
                        provider.clone(),
                    )
                    .await?;

                    token_weth_prices
                        .lock()
                        .unwrap()
                        .insert(pool.token_a, price);

                    price
                }
            };

            //Get weth value of token a in pool
            let token_a_weth_value_in_pool = token_a_reserves as f64
                / 10f64.powf(pool.token_a_decimals.into())
                / token_a_price_per_weth;

            let token_b_price_per_weth = token_weth_prices
                .lock()
                .unwrap()
                .get(&pool.token_b)
                .map(|price| price.to_owned());

            let token_b_price_per_weth = match token_b_price_per_weth {
                Some(price) => price.to_owned(),
                None => {
                    request_throttle.lock().unwrap().increment_or_sleep(1);
                    let price = get_price_of_token_per_weth(
                        pool.token_b,
                        weth_address,
                        &dexes,
                        provider.clone(),
                    )
                    .await?;

                    token_weth_prices
                        .lock()
                        .unwrap()
                        .insert(pool.token_b, price);

                    price
                }
            };

            //Get weth value of token a in pool
            let token_b_weth_value_in_pool = token_b_reserves as f64
                / 10f64.powf(pool.token_b_decimals.into())
                / token_b_price_per_weth;

            //Compare the sum of token_a and token_b usd value against the specified threshold
            let total_weth_value_in_pool = token_a_weth_value_in_pool + token_b_weth_value_in_pool;

            Ok::<_, PairSyncError<P>>((total_weth_value_in_pool, pool))
        }));
    }

    for handle in handles {
        match handle.await {
            Ok(filter_result) => match filter_result {
                Ok((total_weth_value_in_pool, pool)) => {
                    if weth_threshold <= total_weth_value_in_pool {
                        filtered_pools.push(pool);
                    }
                }
                Err(pair_sync_error) => match pair_sync_error {
                    PairSyncError::PairDoesNotExistInDexes(_, _) => {}
                    _ => return Err(pair_sync_error),
                },
            },

            Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
        }
    }

    Ok(filtered_pools)
}

//Filter to remove tokens that incorporate fees on transfer.
//This filter determines fee on transfer tokens by simulating a transfer and checking if the recieved amount is less
//than the sent amount. It can not be guaranteed that all fee tokens are filtered out. For example,
//if a token has a fee mechanic but the fee is set to 0, this filter will not remove the token.
#[allow(dead_code)]
fn filter_fee_tokens<P: 'static + JsonRpcClient>(_provider: Arc<Provider<P>>) {}
