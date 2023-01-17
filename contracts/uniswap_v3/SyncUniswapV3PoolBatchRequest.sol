//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFactory {
    function allPairs(uint256 idx) external returns (address);
}

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract SyncUniswapV3PoolBatchRequest {
    constructor(address[] memory pools) {
        // assembly {
        //     // Return from the start of the data (discarding the original data address)
        //     // up to the end of the memory used
        //     mstore(0x00, returnData)
        //     return(0x00, returnDataLength)
        // }
    }
}
