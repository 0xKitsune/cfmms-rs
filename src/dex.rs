use std::str::FromStr;

use ethers::types::{BlockNumber, H160, H256};

#[derive(Debug, Clone, Copy)]
pub struct Dex {
    pub factory_address: H160,
    pub dex_type: DexType,
    pub creation_block: BlockNumber,
}

#[derive(Debug, Clone, Copy)]
pub enum DexType {
    UniswapV2,
    UniswapV3,
}

impl Dex {
    pub fn new(address: String, dex_type: DexType, creation_block: BlockNumber) -> Dex {
        if creation_block.as_number().is_none() {
            panic!("A valid BlockNumber::Number must be supplied as the creation block when initializing a new Dex")
        }

        Dex {
            factory_address: H160::from_slice(address.as_bytes()),
            dex_type,
            creation_block,
        }
    }

    pub fn pair_created_event_signature(&self) -> H256 {
        match self.dex_type {
            DexType::UniswapV2 => {
                H256::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9")
                    .unwrap()
            }
            DexType::UniswapV3 => {
                H256::from_str("0x783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118")
                    .unwrap()
            }
        }
    }
}
