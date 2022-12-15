## ------------------------------------
## Tests and Docs are still being written 🏗️.
This repo is still being tested, assume bugs until tested. If you would like to help contribute on the tests or docs, feel free to open up an issue or make a PR.
## ------------------------------------

# cfmms-rs

Sync pairs simulate swaps, and interact with constant function market makers on Ethereum.

- [Crates.io](https://crates.io/crates/cfmms)
- [Documentation in progress](https://docs.rs/cfmms/0.1.3/cfmms/)


## Supported Dexes

| Dex | Status |
|----------|------|
| UniswapV2 variants  | ✅||
| UniswapV3  | ✅||


## Running Examples

To run any of the examples, supply your node endpoint to the endpoint variable in each example file. For example in `sync-pairs.rs`:

```rust
    //Add rpc endpoint here:
    let rpc_endpoint = "";
```

Once you have supplied a node endpoint, you can simply run `cargo run --example <example_name>`.

