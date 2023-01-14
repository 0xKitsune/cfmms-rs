//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {}


interface IERC20 {
    function decimals() external view returns (uint256);
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
        uint128 reserve0;
        uint128 reserve1; 
        
        
    }

    constructor(address[] calldata pools) {
        // There is a max number of pool as a too big returned data times out the rpc
        PoolData[] memory poolData = new address[](pools.length);

        // Query every pool balance
        for (uint256 i = 0; i < pools.length; ) {
            allPairs[i] = IUniswapV2Pair(pools[i]).

            unchecked {
                ++i;
            }
        }

        // insure abi encoding, not needed here but increase reusability for different return types
        // note: abi.encode add a first 32 bytes word with the address of the original data
        bytes memory returnData = abi.encode(poolData);
        uint256 returnDataLength = returnData.length;

        assembly {
            // Return from the start of the data (discarding the original data address)
            // up to the end of the memory used
            mstore(0x00, returnData)
            return(0x00, returnDataLength)
        }
    }
}
