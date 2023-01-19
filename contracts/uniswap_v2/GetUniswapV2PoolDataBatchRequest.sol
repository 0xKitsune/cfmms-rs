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
            //Get tokens A and B
            poolData.tokenA = IUniswapV2Pair(poolAddress).token0();
            poolData.tokenB = IUniswapV2Pair(poolAddress).token1();

            //Get tokenA decimals
            (
                bool tokenADecimalsSuccess,
                bytes memory tokenADecimalsData
            ) = poolData.tokenA.call(abi.encodeWithSignature("decimals()"));

            if (tokenADecimalsSuccess) {
                uint256 tokenADecimals;

                assembly {
                    tokenADecimals := mload(add(tokenADecimalsData, 20))
                }

                if (tokenADecimals == 0) {
                    continue;
                } else {
                    poolData.tokenADecimals = uint8(tokenADecimals);
                }
            } else {
                continue;
            }

            //Get tokenB decimals

            (
                bool tokenBDecimalsSuccess,
                bytes memory tokenBDecimalsData
            ) = poolData.tokenB.call(abi.encodeWithSignature("decimals()"));

            if (tokenBDecimalsSuccess) {
                uint256 tokenBDecimals;

                assembly {
                    tokenBDecimals := mload(add(tokenBDecimalsData, 20))
                }
                if (tokenBDecimals == 0) {
                    continue;
                } else {
                    poolData.tokenBDecimals = uint8(tokenBDecimals);
                }
            } else {
                continue;
            }

            // Get reserves
            (poolData.reserve0, poolData.reserve1, ) = IUniswapV2Pair(
                poolAddress
            ).getReserves();

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
