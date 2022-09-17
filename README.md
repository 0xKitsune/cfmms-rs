# pair_sync

Note: This repo is not finished yet. The progress bar needs patching, I am adding filters and additional Dex variants.

A simple library to get all pairs from any Dex and sync reserves. 

```rust
//Initialize a vec to hold all the dexes
let mut dexes: Vec<Dex> = vec![];

//Add UniswapV3
dexes.push(Dex::new(
    //Specify the factory address
    "0x1F98431c8aD98523631AE4a59f267346ea31F984",
    //Specify the dex variant
    DexType::UniswapV3,
    //Specify the factory contract's creation block number
    12369621,
));

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




