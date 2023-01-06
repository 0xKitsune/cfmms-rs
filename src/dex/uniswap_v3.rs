

use ethers::{
    types::{BlockNumber, H160, H256},
};





#[derive(Debug, Clone, Copy)]
pub struct UniswapV3Dex {
    pub factory_address: H160,
    pub creation_block: BlockNumber,
}

pub const POOL_CREATED_EVENT_SIGNATURE: H256 = H256([
    120, 60, 202, 28, 4, 18, 221, 13, 105, 94, 120, 69, 104, 201, 109, 162, 233, 194, 47, 249, 137,
    53, 122, 46, 139, 29, 155, 43, 78, 107, 113, 24,
]);

impl UniswapV3Dex {
    pub fn new(factory_address: H160, creation_block: BlockNumber) -> UniswapV3Dex {
        UniswapV3Dex {
            factory_address,
            creation_block,
        }
    }

    pub const fn pool_created_event_signature(&self) -> H256 {
        POOL_CREATED_EVENT_SIGNATURE
    }
}
