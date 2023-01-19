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

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];

            if (codeSizeIsZero(poolAddress)) continue;

            PoolData memory poolData;

            try IUniswapV2Pair(poolAddress).token0() returns (address token0) {
                poolData.tokenA = token0;
            } catch {
                continue;
            }

            if (codeSizeIsZero(poolData.tokenA)) continue;

            try IERC20(poolData.tokenA).decimals() returns (
                uint8 tokenADecimals
            ) {
                poolData.tokenADecimals = tokenADecimals;
            } catch {
                continue;
            }

            try IUniswapV2Pair(poolAddress).token1() returns (address token1) {
                poolData.tokenB = token1;
            } catch {
                continue;
            }

            if (codeSizeIsZero(poolData.tokenB)) continue;

            try IERC20(poolData.tokenB).decimals() returns (
                uint8 tokenBDecimals
            ) {
                poolData.tokenBDecimals = tokenBDecimals;
            } catch {
                continue;
            }

            try IUniswapV2Pair(poolAddress).getReserves() returns (
                uint112 reserve0,
                uint112 reserve1,
                uint32 blockTimestampLast
            ) {
                poolData.reserve0 = reserve0;
                poolData.reserve1 = reserve1;
            } catch {
                continue;
            }

            allPoolData[i] = poolData;
        }

        // insure abi encoding, not needed here but increase reusability for different return types
        // note: abi.encode add a first 32 bytes word with the address of the original data
        bytes memory _abiEncodedData = abi.encode(allPoolData);

        assembly {
            // Return from the start of the data (discarding the original data address)
            // up to the end of the memory used
            let dataStart := add(_abiEncodedData, 0x20)
            return(dataStart, sub(msize(), dataStart))
        }
    }

    function codeSizeIsZero(address target) internal view returns (bool) {
        if (target.code.length == 0) {
            return true;
        } else {
            return false;
        }
    }
}
