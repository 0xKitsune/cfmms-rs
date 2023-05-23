//! # Sync
//!
//! Syncs multiple pool states between all dexes.
//! Contains logic for managing adding checkpoints during a sync,
//! endpoint throttling requests, and removing inactive pools.

use crate::{checkpoint, errors::CFMMError};

use super::dex::Dex;
use super::pool::Pool;
use super::throttle::RequestThrottle;
use ethers::providers::Middleware;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    panic::resume_unwind,
    sync::{Arc, Mutex},
};

/// Synchronizes all pairs and sync reserve values for each DEX in `Vec<Dex>` with
/// **fixed** step size of `100000`. Step is the block range used to get all pools
/// from a dex if syncing from event logs. Use `sync_pairs_with_step` to specify a
/// custom step size. Sync pairs with throttle but the throttle is disabled because
/// the default variable is fixed at 0.
///
/// This function synchronizes the pairs and reserve values for each DEX in `Vec<Dex>`.
/// It utilizes the specified `middleware` for performing the synchronization. An
/// optional `checkpoint_path` can be provided to resume the synchronization from a
/// previously saved checkpoint.

pub async fn sync_pairs<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    middleware: Arc<M>,
    checkpoint_path: Option<&str>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //throttle is disabled with a default value of 0
    sync_pairs_with_throttle(dexes, 100000, middleware, 0, checkpoint_path).await
}

/// Synchronizes all pairs and sync reserve values for each DEX in `Vec<Dex>`
/// with **variable** step size. Step is the block range used to get all pools
/// from a dex if syncing from event logs. Sync pairs with throttle but the
/// throttle is disabled because the default variable is fixed at 0.
///
/// This function synchronizes the pairs and reserve values for each DEX
/// in `Vec<Dex>`. It utilizes the specified `middleware` for performing
/// the synchronization. An optional `checkpoint_path` can be provided to
/// resume the synchronization from a previously saved checkpoint.

pub async fn sync_pairs_with_step<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    step: usize,
    middleware: Arc<M>,
    checkpoint_path: Option<&str>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //throttle is disabled with a default value of 0
    sync_pairs_with_throttle(dexes, step, middleware, 0, checkpoint_path).await
}

/// Get all pairs and sync reserve values for each DEX in `Vec<Dex>` with a throttle.
///
/// This function asynchronously retrieves all pairs and synchronizes the reserve values
/// for each DEX in `Vec<Dex>`. It uses a specified `step` to define the block range
/// when syncing from event logs. The synchronization is performed using the given
/// `middleware` and a `requests_per_second_limit` is applied to limit the number of
/// requests per second. An optional `checkpoint_path` can be provided to save a
/// checkpoint for resuming the synchronization from a specific point.

pub async fn sync_pairs_with_throttle<M: 'static + Middleware>(
    dexes: Vec<Dex>,
    step: usize,
    middleware: Arc<M>,
    requests_per_second_limit: usize,
    checkpoint_path: Option<&str>,
) -> Result<Vec<Pool>, CFMMError<M>> {
    //Get the current block number
    let current_block = middleware
        .get_block_number()
        .await
        .map_err(CFMMError::MiddlewareError)?;

    //Initialize a new request throttle
    let request_throttle = Arc::new(Mutex::new(RequestThrottle::new(requests_per_second_limit)));

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
            progress_bar.set_message(format!("Getting all pools from: {}", dex.factory_address()));

            let mut pools = dex
                .get_all_pools(
                    request_throttle.clone(),
                    step,
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

            //Get all of the pool data and sync the pool
            progress_bar.set_message(format!(
                "Getting all pool data for: {}",
                dex.factory_address()
            ));
            progress_bar.set_length(pools.len() as u64);

            dex.get_all_pool_data(
                &mut pools,
                request_throttle.clone(),
                progress_bar.clone(),
                middleware.clone(),
            )
            .await?;

            //Clean empty pools
            pools = remove_empty_pools(pools);

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

    //Save a checkpoint if a path is provided
    if checkpoint_path.is_some() {
        let checkpoint_path = checkpoint_path.unwrap();

        checkpoint::construct_checkpoint(
            dexes,
            &aggregated_pools,
            current_block.as_u64(),
            checkpoint_path,
        )
    }

    //Return the populated aggregated pools vec
    Ok(aggregated_pools)
}

/// Removes empty pools with empty `token_a` values from the `pools` vector.
///
/// This function iterates over the provided `pools` vector and removes any pools
/// that have an empty `token_a` value. The cleaned vector is then returned.
pub fn remove_empty_pools(pools: Vec<Pool>) -> Vec<Pool> {
    let mut cleaned_pools = vec![];

    for pool in pools {
        match pool {
            Pool::UniswapV2(uniswap_v2_pool) => {
                if !uniswap_v2_pool.token_a.is_zero() {
                    cleaned_pools.push(pool)
                }
            }
            Pool::UniswapV3(uniswap_v3_pool) => {
                if !uniswap_v3_pool.token_a.is_zero() {
                    cleaned_pools.push(pool)
                }
            }
        }
    }

    cleaned_pools
}
