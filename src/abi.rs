use ethers::prelude::abigen;

abigen!(
    IUniswapV2Factory,
    r#"[
        function getPair(address tokenA, address tokenB) external view returns (address pair)
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256)
    ]"#;

    IUniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function token0() external view returns (address)
    ]"#;

    IUniswapV3Factory,
    r#"[
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool)
        event PoolCreated(address indexed token0, address indexed token1, uint24 indexed fee, int24 tickSpacing, address pool)
    ]"#;

    IUniswapV3Pool,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
        function liquidity() external view returns (uint128)
        function slot0() external view returns (uint160, int24, uint16, uint16, uint16, uint8, bool)
        ]"#;

    IErc20,
    r#"[
        function balanceOf(address account) external view returns (uint256)
        function decimals() external view returns (uint8)
    ]"#;


);
