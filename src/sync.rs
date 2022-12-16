use crate::{checkpoint, error::CFMMError};

use super::dex::Dex;
use super::pool::Pool;
use super::throttle::RequestThrottle;
use ethers::{
    providers::Middleware,
    types::{BlockNumber, Filter, ValueOrArray, H160, U64},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    panic::resume_unwind,
    sync::{Arc, Mutex},
};

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    save_checkpoint: bool,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Sync pairs with throttle but set the requests per second limit to 0, disabling the throttle.
    sync_pairs_with_throttle(dexes, middleware, 0, save_checkpoint).await
}

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs_with_throttle<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
    save_checkpoint: bool,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Initalize a new request throttle
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
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Blocks")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            let pools = get_all_pools_from_dex(
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
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Blocks")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            let pools = get_all_pools_from_dex(
                dex,
                async_provider.clone(),
                BlockNumber::Number(current_block),
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

//Function to get all pair created events for a given Dex factory address and sync pool data
pub async fn get_all_pools_from_dex<M: 'static + Middleware>(
    dex: Dex,
    middleware: Arc<M>,
    current_block: BlockNumber,
    request_throttle: Arc<Mutex<RequestThrottle>>,
    progress_bar: ProgressBar,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Define the step for searching a range of blocks for pair created events
    let step = 100000;
    //Unwrap can be used here because the creation block was verified within `Dex::new()`
    let from_block = dex
        .creation_block()
        .as_number()
        .expect("Error using converting creation block as number")
        .as_u64();
    let current_block = current_block
        .as_number()
        .expect("Error using converting current block as number")
        .as_u64();

    let mut aggregated_pairs: Vec<Pool> = vec![];

    //Initialize the progress bar message
    progress_bar.set_length(current_block - from_block);
    progress_bar.set_message(format!("Getting all pools from: {}", dex.factory_address()));

    //Init a new vec to keep track of tasks
    let mut handles = vec![];

    //For each block within the range, get all pairs asynchronously
    for from_block in (from_block..=current_block).step_by(step) {
        let request_throttle = request_throttle.clone();
        let provider = middleware.clone();
        let progress_bar = progress_bar.clone();

        //Spawn a new task to get pair created events from the block range
        handles.push(tokio::spawn(async move {
            let mut pools = vec![];

            //Get pair created event logs within the block range
            let to_block = from_block + step as u64;

            //Update the throttle
            request_throttle
                .lock()
                .expect("Error when acquiring request throttle mutex lock")
                .increment_or_sleep(1);

            let logs = provider
                .get_logs(
                    &Filter::new()
                        .topic0(ValueOrArray::Value(dex.pool_created_event_signature()))
                        .address(dex.factory_address())
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await
                .map_err(CFMMError::MiddlewareError)?;

            //For each pair created log, create a new Pair type and add it to the pairs vec
            for log in logs {
                let pool = dex.new_empty_pool_from_event(log)?;
                pools.push(pool);
            }

            //Increment the progress bar by the step
            progress_bar.inc(step as u64);

            Ok::<Vec<Pool>, CFMMError<M>>(pools)
        }));
    }

    //Wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_pairs.extend(sync_result?),

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

    Ok(aggregated_pairs)
}

//Function to get reserves for each pair in the `pairs` vec.
pub async fn get_all_pool_data<M: Middleware>(
    pools: Vec<Pool>,
    dex_factory_address: H160,
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

    //For each pair in the pairs vec, get the reserves asyncrhonously
    for mut pool in pools {
        let request_throttle = request_throttle.clone();
        let provider = middleware.clone();
        let progress_bar = progress_bar.clone();

        request_throttle
            .lock()
            .expect("Error when acquiring request throttle mutex lock")
            .increment_or_sleep(4);

        match pool.get_pool_data(provider.clone()).await {
            Ok(_) => updated_pools.push(pool),

            Err(cfmm_error) => {
                if let CFMMError::MiddlewareError(_) = &cfmm_error {
                    return Err(cfmm_error);
                }
            }
        }

        progress_bar.inc(1);
    }

    Ok(updated_pools)
}
