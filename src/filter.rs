use crate::dex::Dex;
use crate::error::CFMMError;
use crate::pool::{Pool, UniswapV2Pool, UniswapV3Pool};
use crate::throttle::RequestThrottle;
use async_trait::async_trait;
use ethers::providers::{JsonRpcClient, Middleware, Provider};
use ethers::types::H160;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use num_bigfloat::BigFloat;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{collections::HashSet, sync::Arc};

#[async_trait]
trait FilteredPool {
    fn tokens(&self) -> Vec<H160>;

    fn address(&self) -> H160;

    async fn get_weth_value_in_pool<M: Middleware>(
        &self,
        weth_address: H160,
        dexes: &[Dex],
        token_weth_pool_min_weth_threshold: u128,
        middleware: Arc<M>,
        token_weth_prices: Arc<Mutex<HashMap<H160, f64>>>,
        request_throttle: Arc<Mutex<RequestThrottle>>,
    ) -> Result<f64, CFMMError<M>>;
}

//Filters out pools where the blacklisted address is the token_a address or token_b address
pub fn filter_blacklisted_tokens(pools: Vec<Pool>, blacklisted_addresses: Vec<H160>) -> Vec<Pool> {
    let mut filtered_pools = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pool in pools {
        let mut blacklisted_token_in_pool = false;
        for token in pool.tokens() {
            if blacklist.contains(&token) {
                blacklisted_token_in_pool = true;
            }
        }

        if !blacklisted_token_in_pool {
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
        if !blacklist.contains(&pool.address()) {
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
        let mut blacklisted_address_in_pool = false;
        for token in pool.tokens() {
            if blacklist.contains(&token) {
                blacklisted_address_in_pool = true;
            }
        }

        if blacklist.contains(&pool.address()) {
            blacklisted_address_in_pool = true;
        }

        if !blacklisted_address_in_pool {
            filtered_pools.push(pool);
        }
    }

    filtered_pools
}

//Filter that removes pools with that contain less than a specified usd value
#[allow(clippy::too_many_arguments)]
pub async fn filter_pools_below_usd_threshold<M: Middleware>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    usd_weth_pool: Pool,
    usd_address: H160,
    weth_address: H160,
    usd_threshold: f64,
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    filter_pools_below_usd_threshold_with_throttle(
        pools,
        dexes,
        usd_weth_pool,
        usd_address,
        weth_address,
        usd_threshold,
        token_weth_pool_min_weth_threshold,
        middleware,
        0,
    )
    .await
}

//Filter that removes pools with that contain less than a specified usd value
#[allow(clippy::too_many_arguments)]
pub async fn filter_pools_below_usd_threshold_with_throttle<M: Middleware>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    usd_weth_pool: Pool,
    usd_address: H160,
    weth_address: H160,
    usd_threshold: f64,
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
) -> Result<Vec<Pool>, CFMMError<M>> {
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
    let usd_price_per_weth = usd_weth_pool.calculate_price(usd_address);

    //Initialize a Hashmap to keep track of token/weth prices already found to avoid unnecessary calls to the node
    let token_weth_prices: Arc<Mutex<HashMap<H160, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    //For each pool, check if the usd value meets the specified threshold
    for pool in pools {
        progress_bar.inc(1);
        //Compare the sum of token_a and token_b usd value against the specified threshold
        let total_usd_value_in_pool = match pool
            .get_weth_value_in_pool(
                weth_address,
                &dexes,
                token_weth_pool_min_weth_threshold,
                middleware.clone(),
                token_weth_prices.clone(),
                request_throttle.clone(),
            )
            .await
        {
            Ok(weth_value_in_pool) => weth_value_in_pool * usd_price_per_weth,
            Err(pair_sync_error) => match pair_sync_error {
                CFMMError::PairDoesNotExistInDexes(token_a, token_b) => {
                    println!("Pair does not exist in dexes: {:?} {:?}", token_a, token_b);
                    0.0
                }
                CFMMError::ContractError(contract_error) => {
                    println!("Contract Error: {:?}", contract_error);

                    0.0
                }
                _ => return Err(pair_sync_error),
            },
        };

        if usd_threshold <= total_usd_value_in_pool {
            filtered_pools.push(pool);
        }
    }

    Ok(filtered_pools)
}

//Filter that removes pools with that contain less than a specified weth value
//
pub async fn filter_pools_below_weth_threshold<M: Middleware>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    weth_address: H160,
    weth_threshold: f64,
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    filter_pools_below_weth_threshold_with_throttle(
        pools,
        dexes,
        weth_address,
        weth_threshold,
        token_weth_pool_min_weth_threshold,
        middleware,
        0,
    )
    .await
}

pub async fn filter_pools_below_weth_threshold_with_throttle<M: Middleware>(
    pools: Vec<Pool>,
    dexes: Vec<Dex>,
    weth_address: H160,
    weth_threshold: f64,
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
) -> Result<Vec<Pool>, CFMMError<M>> {
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

    //Initialize a Hashmap to keep track of token/weth prices already found to avoid unnecessary calls to the node
    let token_weth_prices: Arc<Mutex<HashMap<H160, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    //For each pool, check if the usd value meets the specified threshold
    for pool in pools {
        let token_weth_prices = token_weth_prices.clone();
        let request_throttle = request_throttle.clone();
        let middleware = middleware.clone();
        let dexes = dexes.clone();
        let progress_bar = progress_bar.clone();

        progress_bar.inc(1);
        //Compare the sum of token_a and token_b usd value against the specified threshold
        let total_weth_value_in_pool = match pool
            .get_weth_value_in_pool(
                weth_address,
                &dexes,
                token_weth_pool_min_weth_threshold,
                middleware.clone(),
                token_weth_prices.clone(),
                request_throttle.clone(),
            )
            .await
        {
            Ok(weth_value_in_pool) => weth_value_in_pool,
            Err(pair_sync_error) => match pair_sync_error {
                CFMMError::PairDoesNotExistInDexes(_, _) | CFMMError::ContractError(_) => 0.0,
                _ => return Err(pair_sync_error),
            },
        };

        if weth_threshold <= total_weth_value_in_pool {
            filtered_pools.push(pool);
        }
    }

    Ok(filtered_pools)
}

async fn get_price_of_token_per_weth<M: Middleware>(
    token_address: H160,
    weth_address: H160,
    dexes: &[Dex],
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
) -> Result<f64, CFMMError<M>> {
    if token_address == weth_address {
        return Ok(1.0);
    }

    //Get token_a/weth price
    let token_weth_pool = get_token_to_weth_pool(
        token_address,
        weth_address,
        dexes,
        token_weth_pool_min_weth_threshold,
        middleware.clone(),
    )
    .await?;

    let token_price_per_weth = token_weth_pool.calculate_price(token_address);

    Ok(token_price_per_weth)
}

//Gets the best token to weth pairing from the dexes provided
async fn get_token_to_weth_pool<M: Middleware>(
    token_a: H160,
    weth_address: H160,
    dexes: &[Dex],
    token_weth_pool_min_weth_threshold: u128,
    middleware: Arc<M>,
) -> Result<Pool, CFMMError<M>> {
    let _pair_address = H160::zero();
    let mut _pool: Pool;

    let mut best_pool: Option<Pool> = None;
    let mut best_weth_reserves = 0_u128;

    for dex in dexes {
        match dex
            .get_pool_with_best_liquidity(token_a, weth_address, middleware.clone())
            .await
        {
            Ok(pool) => {
                if pool.is_some() {
                    match pool.unwrap() {
                        Pool::UniswapV2(univ2_pool) => {
                            if univ2_pool.token_a == weth_address {
                                if univ2_pool.reserve_0 > best_weth_reserves {
                                    best_weth_reserves = univ2_pool.reserve_0;
                                    best_pool = pool;
                                } else if univ2_pool.reserve_1 > best_weth_reserves {
                                    best_weth_reserves = univ2_pool.reserve_1;
                                    best_pool = pool;
                                }
                            } else if univ2_pool.reserve_1 > best_weth_reserves {
                                best_weth_reserves = univ2_pool.reserve_1;
                                best_pool = pool;
                            } else if univ2_pool.reserve_0 > best_weth_reserves {
                                best_weth_reserves = univ2_pool.reserve_0;
                                best_pool = pool;
                            }
                        }

                        Pool::UniswapV3(univ3_pool) => {
                            let (reserve_0, reserve_1) = univ3_pool.calculate_virtual_reserves();

                            if univ3_pool.token_a == weth_address {
                                if reserve_0 > best_weth_reserves {
                                    best_weth_reserves = reserve_0;
                                    best_pool = pool;
                                } else if reserve_1 > best_weth_reserves {
                                    best_weth_reserves = reserve_1;
                                    best_pool = pool;
                                }
                            } else if reserve_1 > best_weth_reserves {
                                best_weth_reserves = reserve_1;
                                best_pool = pool;
                            } else if reserve_0 > best_weth_reserves {
                                best_weth_reserves = reserve_0;
                                best_pool = pool;
                            }
                        }
                    }
                }
            }

            Err(pair_sync_error) => match pair_sync_error {
                CFMMError::ContractError(_) => continue,
                other => return Err(other),
            },
        };
    }

    //If the pool getting the price doesnt have at least x eth, return no pair
    if best_weth_reserves >= token_weth_pool_min_weth_threshold {
        Ok(best_pool.unwrap())
    } else {
        Err(CFMMError::PairDoesNotExistInDexes(token_a, weth_address))
    }
}

#[async_trait]
impl FilteredPool for Pool {
    fn address(&self) -> H160 {
        match self {
            Pool::UniswapV2(pool) => pool.address(),
            Pool::UniswapV3(pool) => pool.address(),
        }
    }

    fn tokens(&self) -> Vec<H160> {
        match self {
            Pool::UniswapV2(pool) => pool.tokens(),
            Pool::UniswapV3(pool) => pool.tokens(),
        }
    }

    async fn get_weth_value_in_pool<M: Middleware>(
        &self,
        weth_address: H160,
        dexes: &[Dex],
        token_weth_pool_min_weth_threshold: u128,
        middleware: Arc<M>,
        token_weth_prices: Arc<Mutex<HashMap<H160, f64>>>,
        request_throttle: Arc<Mutex<RequestThrottle>>,
    ) -> Result<f64, CFMMError<M>> {
        match self {
            Pool::UniswapV2(pool) => {
                pool.get_weth_value_in_pool(
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware,
                    token_weth_prices,
                    request_throttle,
                )
                .await
            }
            Pool::UniswapV3(pool) => {
                pool.get_weth_value_in_pool(
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware,
                    token_weth_prices,
                    request_throttle,
                )
                .await
            }
        }
    }
}

#[async_trait]
impl FilteredPool for UniswapV2Pool {
    fn address(&self) -> H160 {
        self.address
    }

    fn tokens(&self) -> Vec<H160> {
        vec![self.token_a, self.token_b]
    }

    async fn get_weth_value_in_pool<M: Middleware>(
        &self,
        weth_address: H160,
        dexes: &[Dex],
        token_weth_pool_min_weth_threshold: u128,
        middleware: Arc<M>,
        token_weth_prices: Arc<Mutex<HashMap<H160, f64>>>,
        request_throttle: Arc<Mutex<RequestThrottle>>,
    ) -> Result<f64, CFMMError<M>> {
        let token_a_price_per_weth = token_weth_prices
            .lock()
            .unwrap()
            .get(&self.token_a)
            .map(|price| price.to_owned());

        let token_a_price_per_weth = match token_a_price_per_weth {
            Some(price) => price,
            None => {
                request_throttle.lock().unwrap().increment_or_sleep(1);
                let price = get_price_of_token_per_weth(
                    self.token_a,
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware.clone(),
                )
                .await?;

                token_weth_prices
                    .lock()
                    .unwrap()
                    .insert(self.token_a, price);

                price
            }
        };

        //Get weth value of token a in pool
        let token_a_weth_value_in_pool = BigFloat::from(self.reserve_0).to_f64()
            / 10f64.powf(self.token_a_decimals.into())
            / token_a_price_per_weth;

        let token_b_price_per_weth = token_weth_prices
            .lock()
            .unwrap()
            .get(&self.token_b)
            .map(|price| price.to_owned());

        let token_b_price_per_weth = match token_b_price_per_weth {
            Some(price) => price.to_owned(),
            None => {
                request_throttle.lock().unwrap().increment_or_sleep(1);
                let price = get_price_of_token_per_weth(
                    self.token_b,
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware.clone(),
                )
                .await?;

                token_weth_prices
                    .lock()
                    .unwrap()
                    .insert(self.token_b, price);

                price
            }
        };

        //Get weth value of token a in pool
        let token_b_weth_value_in_pool = BigFloat::from(self.reserve_1).to_f64()
            / 10f64.powf(self.token_b_decimals.into())
            / token_b_price_per_weth;

        //Return weth value in pool
        Ok(token_a_weth_value_in_pool + token_b_weth_value_in_pool)
    }
}

#[async_trait]
impl FilteredPool for UniswapV3Pool {
    fn address(&self) -> H160 {
        self.address
    }

    fn tokens(&self) -> Vec<H160> {
        vec![self.token_a, self.token_b]
    }

    async fn get_weth_value_in_pool<M: Middleware>(
        &self,
        weth_address: H160,
        dexes: &[Dex],
        token_weth_pool_min_weth_threshold: u128,
        middleware: Arc<M>,
        token_weth_prices: Arc<Mutex<HashMap<H160, f64>>>,
        request_throttle: Arc<Mutex<RequestThrottle>>,
    ) -> Result<f64, CFMMError<M>> {
        let (reserve_0, reserve_1) = self.calculate_virtual_reserves();

        let token_a_price_per_weth = token_weth_prices
            .lock()
            .unwrap()
            .get(&self.token_a)
            .map(|price| price.to_owned());

        let token_a_price_per_weth = match token_a_price_per_weth {
            Some(price) => price,
            None => {
                request_throttle.lock().unwrap().increment_or_sleep(1);
                let price = get_price_of_token_per_weth(
                    self.token_a,
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware.clone(),
                )
                .await?;

                token_weth_prices
                    .lock()
                    .unwrap()
                    .insert(self.token_a, price);

                price
            }
        };

        //Get weth value of token a in pool
        let token_a_weth_value_in_pool = BigFloat::from(reserve_0).to_f64()
            / 10f64.powf(self.token_a_decimals.into())
            / token_a_price_per_weth;

        let token_b_price_per_weth = token_weth_prices
            .lock()
            .unwrap()
            .get(&self.token_b)
            .map(|price| price.to_owned());

        let token_b_price_per_weth = match token_b_price_per_weth {
            Some(price) => price.to_owned(),
            None => {
                request_throttle.lock().unwrap().increment_or_sleep(1);
                let price = get_price_of_token_per_weth(
                    self.token_b,
                    weth_address,
                    dexes,
                    token_weth_pool_min_weth_threshold,
                    middleware.clone(),
                )
                .await?;

                token_weth_prices
                    .lock()
                    .unwrap()
                    .insert(self.token_b, price);

                price
            }
        };

        //Get weth value of token a in pool
        let token_b_weth_value_in_pool = BigFloat::from(reserve_1).to_f64()
            / 10f64.powf(self.token_b_decimals.into())
            / token_b_price_per_weth;

        //Return weth value in pool
        Ok(token_a_weth_value_in_pool + token_b_weth_value_in_pool)
    }
}
