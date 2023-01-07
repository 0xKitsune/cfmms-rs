use ethers::types::{BlockNumber, H160, H256};

#[derive(Debug, Clone, Copy)]
pub struct UniswapV2Dex {
    pub factory_address: H160,
    pub creation_block: BlockNumber,
}

pub const PAIR_CREATED_EVENT_SIGNATURE: H256 = H256([
    13, 54, 72, 189, 15, 107, 168, 1, 52, 163, 59, 169, 39, 90, 197, 133, 217, 211, 21, 240, 173,
    131, 85, 205, 222, 253, 227, 26, 250, 40, 208, 233,
]);

impl UniswapV2Dex {
    pub fn new(factory_address: H160, creation_block: BlockNumber) -> UniswapV2Dex {
        UniswapV2Dex {
            factory_address,
            creation_block,
        }
    }

    pub const fn pool_created_event_signature(&self) -> H256 {
        PAIR_CREATED_EVENT_SIGNATURE
    }
}
