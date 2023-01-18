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
    save_checkpoint: Option<String>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Sync pairs with throttle but set the requests per second limit to 0, disabling the throttle.
    sync_pairs_with_throttle(dexes, middleware, 0, save_checkpoint).await
}

//Get all pairs and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_pairs_with_throttle<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
    save_checkpoint: Option<String>,
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
        let middleware = middleware.clone();
        let request_throttle = request_throttle.clone();
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        //Spawn a new thread to get all pools and sync data for each dex
        handles.push(tokio::spawn(async move {
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            //Get all of the pools from the dex
            let mut pools = dex
                .get_all_pools(
                    request_throttle.clone(),
                    progress_bar.clone(),
                    middleware.clone(),
                )
                .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            dex.get_all_pool_data(
                &mut pools,
                request_throttle.clone(),
                progress_bar.clone(),
                middleware.clone(),
            )
            .await?;

            progress_bar.reset();
            progress_bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} Pairs")
                    .expect("Error when setting progress bar style")
                    .progress_chars("##-"),
            );

            //TODO: sync reserves
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

                pool.sync_pool(middleware.clone()).await?;
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

    //If
    if save_checkpoint.is_some() {
        let save_checkpoint = save_checkpoint.unwrap();

        let latest_block = middleware
            .get_block_number()
            .await
            .map_err(CFMMError::MiddlewareError)?;

        checkpoint::construct_checkpoint(
            dexes,
            &aggregated_pools,
            latest_block.as_u64(),
            save_checkpoint,
        )
    }

    //Return the populated aggregated pools vec
    Ok(aggregated_pools)
}
