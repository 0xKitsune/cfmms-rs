//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */

pragma solidity ^0.8.0;

interface IUniswapV3Pool {
    function token0() external view returns (address);

    function token1() external view returns (address);

    function fee() external view returns (uint24);

    function tickSpacing() external view returns (int24);

    function liquidity() external view returns (uint128);

    function slot0()
        external
        view
        returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );

    function ticks(int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128,
            int56 tickCumulativeOutside,
            uint160 secondsPerLiquidityOutsideX128,
            uint32 secondsOutside,
            bool initialized
        );
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract GetUniswapV3PoolDataBatchRequest {
    struct PoolData {
        address tokenA;
        uint8 tokenADecimals;
        address tokenB;
        uint8 tokenBDecimals;
        uint128 liquidity;
        uint160 sqrtPrice;
        int24 tick;
        int24 tickSpacing;
        uint24 fee;
        int128 liquidityNet;
    }

    constructor(address[] memory pools) {
        PoolData[] memory allPoolData = new PoolData[](pools.length);

        for (uint256 i = 0; i < pools.length; ) {
            address poolAddress = pools[i];

            if (poolAddress.code.length == 0) {
                unchecked {
                    ++i;
                }

                continue;
            }
            PoolData memory poolData;

            (uint160 sqrtPriceX96, int24 tick, , , , , ) = IUniswapV3Pool(
                poolAddress
            ).slot0();

            (, int128 liquidityNet, , , , , , ) = IUniswapV3Pool(poolAddress)
                .ticks(tick);

            poolData.tokenA = IUniswapV3Pool(poolAddress).token0();
            poolData.tokenADecimals = IERC20(poolData.tokenA).decimals();
            poolData.tokenB = IUniswapV3Pool(poolAddress).token1();
            poolData.tokenBDecimals = IERC20(poolData.tokenB).decimals();
            poolData.liquidity = IUniswapV3Pool(poolAddress).liquidity();
            poolData.sqrtPrice = sqrtPriceX96;
            poolData.tick = tick;
            poolData.tickSpacing = IUniswapV3Pool(poolAddress).tickSpacing();
            poolData.fee = IUniswapV3Pool(poolAddress).fee();
            poolData.liquidityNet = liquidityNet;

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
