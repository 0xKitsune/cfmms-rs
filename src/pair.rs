use crate::dex::DexType;
use ethers::types::H160;

#[derive(Debug)]
pub struct Pair {
    pub pair_address: H160,
    pub token_a: H160,
    pub token_b: H160,
    pub a_to_b: bool,
    pub reserve_0: u128,
    pub reserve_1: u128,
    pub fee: u128,
    pub dex_type: DexType,
}

impl Pair {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pair_address: H160,
        token_a: H160,
        token_b: H160,
        a_to_b: bool,
        reserve_0: u128,
        reserve_1: u128,
        fee: u128,
        dex_type: DexType,
    ) -> Pair {
        Pair {
            pair_address,
            token_a,
            token_b,
            a_to_b,
            reserve_0,
            reserve_1,
            fee,
            dex_type,
        }
    }

    pub fn empty_pair(dex_type: DexType) -> Pair {
        Pair {
            pair_address: H160::zero(),
            token_a: H160::zero(),
            token_b: H160::zero(),
            a_to_b: false,
            reserve_0: 0,
            reserve_1: 0,
            fee: 0,
            dex_type,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.token_a == H160::zero()
    }
}
