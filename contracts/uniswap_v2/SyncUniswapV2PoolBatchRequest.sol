//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function getReserves()
        external
        view
        returns (
            uint112 reserve0,
            uint112 reserve1,
            uint32 blockTimestampLast
        );
}

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract SyncUniswapV2PoolBatchRequest {
    struct Reserves {
        uint112 reserve0;
        uint112 reserve1;
    }

    constructor(address[] memory pools) {
        Reserves[] memory allReserves = new Reserves[](pools.length);

        for (uint256 i = 0; i < pools.length; ) {
            Reserves memory reserves;

            (reserves.reserve0, reserves.reserve1, ) = IUniswapV2Pair(pools[i])
                .getReserves();

            allReserves[i] = reserves;
            unchecked {
                ++i;
            }
        }

        bytes memory returnData = abi.encode(allReserves);
        uint256 returnDataLength = returnData.length;

        assembly {
            mstore(0x00, returnData)
            return(0x00, returnDataLength)
        }
    }
}
