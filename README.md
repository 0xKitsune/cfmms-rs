## ------------------------------------
## Tests and Docs are still being written 🏗️.
Tests are still being written, assume bugs until tested. If you would like to help contribute on the tests or docs, feel free to open up an issue or make a PR.
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

Run any of the examples with make `make <example_name>` eg:
```bash
make create-new-pool
```

You can also export your RPC endpoint.
```bash
export ETHEREUM_MAINNET_ENDPOINT=https://eth.llamarpc.com
cargo run --example <example_name>
```
