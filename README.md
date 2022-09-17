# pair_sync

Note: This repo is not finished yet. The progress bar needs patching, I am adding filters and additional Dex variants.

A simple library to get all pairs from any Dex and sync reserves. 

```rust
//Initialize a vec to hold all the dexes
let mut dexes: Vec<Dex> = vec![];

//Add UniswapV3
dexes.push(Dex {
    //Specify the factory address
    factory_address: H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
    //Specify the dex variant
    dex_type: DexType::UniswapV3,
    //Specify the factory contract's creation block number
    creation_block: BlockNumber::Number(U64([12369621])),
});

//Sync all pairs from Univ3
let pairs: Vec<Pair> = sync::sync_pairs(dexes, rpc_endpoint).await?;
```


### Supported Dexes

| Dex | Status |
|----------|------|
| UniswapV2 variants  | ✅||
| UniswapV3  | ✅||



### Running Examples



### Filters




