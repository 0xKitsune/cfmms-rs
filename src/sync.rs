use crate::{batch_requests, checkpoint, dex::DexVariant, error::CFMMError};

use super::dex::Dex;
use super::pool::Pool;
use super::throttle::RequestThrottle;
use ethers::{
    providers::Middleware,
    types::{BlockNumber, Filter, ValueOrArray, H160, U64},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    borrow::BorrowMut,
    panic::resume_unwind,
    sync::{Arc, Mutex},
};

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.

pub async fn sync_pairs<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    batched_calls: bool,
    save_checkpoint: bool,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Sync pairs with throttle but set the requests per second limit to 0, disabling the throttle.
    sync_pairs_with_throttle(dexes, middleware, 0, batched_calls, save_checkpoint).await
}

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs_with_throttle<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
    batched_calls: bool,
    save_checkpoint: bool,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Initialize a new request throttle
    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));

    let current_block = middleware
        .get_block_number()
        .await
        .map_err(CFMMError::MiddlewareError)?;

    //Aggregate the populated pools from each thread
    let mut aggregated_pools: Vec<Pool> = vec![];
    let mut handles = vec![];

    //Initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    //For each dex supplied, get all pair created events and get reserve values
    for dex in dexes.clone() {
        let async_provider = middleware.clone();
        let request_throttle = request_throttle.clone();
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn(async move {
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            let pools = dex
                .get_all_pools(
                    async_provider.clone(),
                    current_block.into(),
                    request_throttle.clone(),
                    progress_bar.clone(),
                )
                .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            let mut pools = get_all_pool_data(
                pools,
                dex.factory_address(),
                async_provider.clone(),
                request_throttle.clone(),
                progress_bar.clone(),
            )
            .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pairs")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            progress_bar.set_length(pools.len() as u64);

            progress_bar.set_message(format!(
                "Syncing reserves for pools from: {}",
                dex.factory_address()
            ));

            for pool in pools.iter_mut() {
                let request_throttle = request_throttle.clone();
                request_throttle
                    .lock()
                    .expect("Error when acquiring request throttle mutex lock")
                    .increment_or_sleep(1);

                pool.sync_pool(async_provider.clone()).await?;
            }

            Ok::<_, CFMMError<M>>(pools)
        }));
    }

    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_pools.extend(sync_result?),
            Err(err) => {
                {
                    if err.is_panic() {
                        // Resume the panic on the main task
                        resume_unwind(err.into_panic());
                    }
                }
            }
        }
    }

    if save_checkpoint {
        let latest_block = middleware
            .get_block_number()
            .await
            .map_err(CFMMError::MiddlewareError)?;

        checkpoint::construct_checkpoint(
            dexes,
            &aggregated_pools,
            latest_block.as_u64(),
            String::from("pool_sync_checkpoint.json"),
        )
    }

    //Return the populated aggregated pools vec
    Ok(aggregated_pools)
}

//Get all pairs
pub async fn get_all_pools<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Initalize a new request throttle
    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));
    let current_block = middleware
        .get_block_number()
        .await
        .map_err(CFMMError::MiddlewareError)?;

    let mut handles = vec![];

    //Initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    //For each dex supplied, get all pair created events and get reserve values
    for dex in dexes {
        let async_provider = middleware.clone();
        let request_throttle = request_throttle.clone();
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn(async move {
            let pools = dex
                .get_all_pools(
                    async_provider.clone(),
                    current_block.into(),
                    request_throttle.clone(),
                    progress_bar.clone(),
                )
                .await?;

            Ok::<_, CFMMError<M>>(pools)
        }));
    }

    //Aggregate the populated pools from each thread
    let mut aggregated_pools: Vec<Pool> = vec![];

    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_pools.extend(sync_result?),
            Err(err) => {
                {
                    if err.is_panic() {
                        // Resume the panic on the main task
                        resume_unwind(err.into_panic());
                    }
                }
            }
        }
    }

    //Return the populated aggregated pools vec
    Ok(aggregated_pools)
}

//Function to get reserves for each pair in the `pairs` vec. for a given dex variant
pub async fn get_all_pool_data<M: Middleware>(
    mut pools: Vec<Pool>,
    dex_factory_address: H160,
    dex_variant: DexVariant,
    middleware: Arc<M>,
    request_throttle: Arc<Mutex<RequestThrottle>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Initialize a vec to track each async task.
    // let mut handles: Vec<tokio::task::JoinHandle<Result<Pool, _>>> = vec![];
    //Create a new vec to aggregate the pools and populate the vec.
    let mut updated_pools: Vec<Pool> = vec![];

    //Initialize the progress bar message
    progress_bar.set_length(pools.len() as u64);
    progress_bar.set_message(format!(
        "Syncing pool data for pairs from: {}",
        dex_factory_address
    ));

    match dex_variant {
        DexVariant::UniswapV2 => {
            let step = 400;
            for pools in pools.chunks_mut(step) {
                request_throttle
                    .lock()
                    .expect("Error when acquiring request throttle mutex lock")
                    .increment_or_sleep(4);

                batch_requests::uniswap_v2::get_pool_data_batch_request(pools, middleware.clone())
                    .await?;

                progress_bar.inc(step as u64);
            }
        }
        DexVariant::UniswapV3 => {}
    }

    //For each pair in the pairs vec, get the pool data

    Ok(updated_pools)
}
