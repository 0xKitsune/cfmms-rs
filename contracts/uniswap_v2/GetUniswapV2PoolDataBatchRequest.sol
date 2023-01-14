//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function token0() external view returns (address);

    function token1() external view returns (address);

    function getReserves()
        external
        view
        returns (
            uint112 reserve0,
            uint112 reserve1,
            uint32 blockTimestampLast
        );
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract GetUniswapV2PoolDataBatchRequest {
    struct PoolData {
        address tokenA;
        uint8 tokenADecimals;
        address tokenB;
        uint8 tokenBDecimals;
        uint112 reserve0;
        uint112 reserve1;
    }

    constructor(address[] memory pools) {
        PoolData[] memory allPoolData = new PoolData[](pools.length);

        for (uint256 i = 0; i < pools.length; ) {
            PoolData memory poolData;

            poolData.tokenA = IUniswapV2Pair(pools[i]).token0();
            poolData.tokenADecimals = IERC20(poolData.tokenA).decimals();
            poolData.tokenB = IUniswapV2Pair(pools[i]).token1();
            poolData.tokenBDecimals = IERC20(poolData.tokenB).decimals();

            (poolData.reserve0, poolData.reserve1, ) = IUniswapV2Pair(pools[i])
                .getReserves();

            allPoolData[i] = poolData;
            unchecked {
                ++i;
            }
        }

        bytes memory returnData = abi.encode(allPoolData);
        uint256 returnDataLength = returnData.length;

        assembly {
            mstore(0x00, returnData)
            return(0x00, returnDataLength)
        }
    }
}
