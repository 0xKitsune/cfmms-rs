use ethers::prelude::gas_escalator::GasEscalatorError;
use ethers::prelude::gas_oracle::MiddlewareError;
use ethers::prelude::nonce_manager::NonceManagerError;
use ethers::prelude::policy::PolicyMiddlewareError;
use ethers::prelude::timelag::TimeLagError;
use ethers::prelude::{gas_escalator, AbiError, ContractError, TimeLag};
use ethers::providers::{JsonRpcClient, Middleware, Provider, ProviderError};
use ethers::types::H160;
use thiserror::Error;
use tokio::task::JoinError;
use uniswap_v3_math::error::UniswapV3MathError;

#[derive(Error, Debug)]
pub enum CFMMError<M>
where
    M: Middleware,
{
    #[error("Middleware error")]
    MiddlewareError(<M as Middleware>::Error),
    #[error("Contract error")]
    ContractError(#[from] ContractError<M>),
    #[error("ABI Codec error")]
    ABICodecError(#[from] AbiError),
    #[error("Eth ABI error")]
    EthABIError(#[from] ethers::abi::Error),
    #[error("Join error")]
    JoinError(#[from] JoinError),
    #[error("Uniswap V3 math error")]
    UniswapV3MathError(#[from] UniswapV3MathError),
    #[error("Pair for token_a/token_b does not exist in provided dexes")]
    PairDoesNotExistInDexes(H160, H160),
}
