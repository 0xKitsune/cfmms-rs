## ------------------------------------
## Tests and Docs are still being written 🏗️.
Tests are still being written, assume bugs until tested. If you would like to help contribute on the tests or docs, feel free to open up an issue or make a PR.
## ------------------------------------

# cfmms-rs

Sync pairs simulate swaps, and interact with constant function market makers on Ethereum.

- [Crates.io](https://crates.io/crates/cfmms)
- [Documentation in progress](https://docs.rs/cfmms/0.1.3/cfmms/)

## Project Layout
```./
├── src/
│   ├── batch_requests/
│   ├── dex/
│   ├── pool/
│   ├── abi.rs
│   ├── checkpoint.rs
│   ├── errors.rs
│   ├── lib.rs
│   ├── sync.rs
│   └── throttle.rs
├── Cargo.lock
├── Cargo.toml
├── foundry.toml
└── README.md
```



## Supported Dexes

| Dex | Status |
|----------|------|
| UniswapV2 variants  | ✅||
| UniswapV3  | ✅||


## Running Examples

To run any of the examples, first set a local environment variable called `ETHEREUM_MAINNET_ENDPOINT`. Then you can simply run `cargo run --example <example_name>`.

