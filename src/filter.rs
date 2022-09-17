use std::collections::HashSet;

use ethers::types::H160;

use crate::pair::Pair;

//Filters out pairs where the blacklisted address is the token_a address or token_b address
pub fn filter_blacklisted_tokens(pairs: Vec<Pair>, blacklisted_addresses: Vec<H160>) -> Vec<Pair> {
    let mut filtered_pairs = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pair in pairs {
        if !blacklist.contains(&pair.token_a) || !blacklist.contains(&pair.token_b) {
            filtered_pairs.push(pair);
        }
    }

    filtered_pairs
}

//Filters out pairs where the blacklisted address is the pair address
pub fn filter_blacklisted_pools(pairs: Vec<Pair>, blacklisted_addresses: Vec<H160>) -> Vec<Pair> {
    let mut filtered_pairs = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pair in pairs {
        if !blacklist.contains(&pair.pair_address) {
            filtered_pairs.push(pair);
        }
    }

    filtered_pairs
}

//Filters out pairs where the blacklisted address is the pair address, token_a address or token_b address
pub fn filter_blacklisted_addresses(
    pairs: Vec<Pair>,
    blacklisted_addresses: Vec<H160>,
) -> Vec<Pair> {
    let mut filtered_pairs = vec![];
    let blacklist: HashSet<H160> = blacklisted_addresses.into_iter().collect();

    for pair in pairs {
        if !blacklist.contains(&pair.pair_address)
            || !blacklist.contains(&pair.token_a)
            || !blacklist.contains(&pair.token_b)
        {
            filtered_pairs.push(pair);
        }
    }

    filtered_pairs
}

#[allow(dead_code)]
fn filter_pools_below_usd_threshold() {}

//Filter to remove tokens that incorporate fees on transfer.
//This filter determines fee on transfer tokens by simulating a transfer and checking if the recieved amount is less
//than the sent amount. It can not be guaranteed that all fee tokens are filtered out. For example,
//if a token has a fee mechanic but the fee is set to 0, this filter will not remove the token.
#[allow(dead_code)]
fn filter_fee_tokens() {}
