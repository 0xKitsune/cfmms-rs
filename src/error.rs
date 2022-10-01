use ethers::prelude::{AbiError, ContractError};
use ethers::providers::{JsonRpcClient, Provider, ProviderError};
use ethers::types::H160;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum PairSyncError<P>
where
    P: JsonRpcClient,
{
    #[error("Provider error")]
    ProviderError(#[from] ProviderError),
    #[error("Contract error")]
    ContractError(#[from] ContractError<Provider<P>>),
    #[error("ABI error")]
    ABIError(#[from] AbiError),
    #[error("Join error")]
    JoinError(#[from] JoinError),
    #[error("Pair for token_a/token_b does not exist in provided dexes")]
    PairDoesNotExistInDexes(H160, H160),
}
