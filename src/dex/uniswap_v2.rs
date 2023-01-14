use std::sync::Arc;

use ethers::{
    abi::ParamType,
    providers::Middleware,
    types::{BlockNumber, Log, H160, H256},
};
use indicatif::ProgressBar;

use crate::{
    abi,
    error::CFMMError,
    pool::{Pool, UniswapV2Pool},
};

use super::DexVariant;

#[derive(Debug, Clone, Copy)]
pub struct UniswapV2Dex {
    pub factory_address: H160,
    pub creation_block: BlockNumber,
}

pub const PAIR_CREATED_EVENT_SIGNATURE: H256 = H256([
    13, 54, 72, 189, 15, 107, 168, 1, 52, 163, 59, 169, 39, 90, 197, 133, 217, 211, 21, 240, 173,
    131, 85, 205, 222, 253, 227, 26, 250, 40, 208, 233,
]);
pub const SYNC_EVENT_SIGNATURE: H256 = H256([
    28, 65, 30, 154, 150, 224, 113, 36, 28, 47, 33, 247, 114, 107, 23, 174, 137, 227, 202, 180,
    199, 139, 229, 14, 6, 43, 3, 169, 255, 251, 186, 209,
]);

impl UniswapV2Dex {
    pub fn new(factory_address: H160, creation_block: BlockNumber) -> UniswapV2Dex {
        UniswapV2Dex {
            factory_address,
            creation_block,
        }
    }

    pub const fn sync_event_signature(&self) -> H256 {
        SYNC_EVENT_SIGNATURE
    }

    pub const fn pool_created_event_signature(&self) -> H256 {
        PAIR_CREATED_EVENT_SIGNATURE
    }

    pub async fn new_pool_from_event<M: Middleware>(
        &self,
        log: Log,
        middleware: Arc<M>,
    ) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Address, ParamType::Uint(256)], &log.data)?;
        let pair_address = tokens[0].to_owned().into_address().unwrap();
        Pool::new_from_address(pair_address, DexVariant::UniswapV2, middleware).await
    }

    pub fn new_empty_pool_from_event<M: Middleware>(&self, log: Log) -> Result<Pool, CFMMError<M>> {
        let tokens = ethers::abi::decode(&[ParamType::Address, ParamType::Uint(256)], &log.data)?;
        let token_a = H160::from(log.topics[0]);
        let token_b = H160::from(log.topics[1]);
        let address = tokens[0].to_owned().into_address().unwrap();

        Ok(Pool::UniswapV2(UniswapV2Pool {
            address,
            token_a,
            token_b,
            token_a_decimals: 0,
            token_b_decimals: 0,
            reserve_0: 0,
            reserve_1: 0,
            fee: 300,
        }))
    }

    pub async fn get_all_pairs<M: 'static + Middleware>(
        self,
        middleware: Arc<M>,
        progress_bar: ProgressBar,
    ) -> Result<Vec<Pool>, CFMMError<M>> {
        let mut aggregated_pairs: Vec<Pool> = vec![];

        let pairs = abi::IUniswapV2Factory::new(self.factory_address, middleware)
            .all_pairs()
            .call()
            .await?;

        //Initialize the progress bar message
        progress_bar.set_length(pairs.len() as u64);
        progress_bar.set_message(format!("Getting all pools from: {}", self.factory_address));

        //Init a new vec to keep track of tasks
        let mut handles = vec![];

        //For each block within the range, get all pairs asynchronously

        //TODO: instead of going through each block, update this to batch everything into one call and get all pool data
        
        here ^^

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
                        &ethers::types::Filter::new()
                            .topic0(ValueOrArray::Value(self.pool_created_event_signature()))
                            .address(self.factory_address)
                            .from_block(BlockNumber::Number(ethers::types::U64([from_block])))
                            .to_block(BlockNumber::Number(ethers::types::U64([to_block]))),
                    )
                    .await
                    .map_err(CFMMError::MiddlewareError)?;

                //For each pair created log, create a new Pair type and add it to the pairs vec
                for log in logs {
                    let pool = self.new_empty_pool_from_event(log)?;
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
}
