//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFactory {
    function allPairs(uint256 idx) external returns (address);
}

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract GetUniswapV3PoolDataBatchRequest {
    constructor(
        uint256 from,
        uint256 step,
        address factory
    ) {
        // There is a max number of pool as a too big returned data times out the rpc
        address[] memory allPairs = new address[](step);

        // Query every pool balance
        for (uint256 i = 0; i < step; i++) {
            allPairs[i] = IFactory(factory).allPairs(from + i);
        }

        // insure abi encoding, not needed here but increase reusability for different return types
        // note: abi.encode add a first 32 bytes word with the address of the original data
        bytes memory returnData = abi.encode(allPairs);
        uint256 returnDataLength = returnData.length;

        assembly {
            // Return from the start of the data (discarding the original data address)
            // up to the end of the memory used
            mstore(0x00, returnData)
            return(0x00, returnDataLength)
        }
    }
}
