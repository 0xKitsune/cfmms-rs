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
        address[] memory allPairs = new address[](step);

        for (uint256 i = 0; i < step; i++) {
            allPairs[i] = IFactory(factory).allPairs(from + i);
        }

        bytes memory returnData = abi.encode(allPairs);
        uint256 returnDataLength = returnData.length;

        assembly {
            mstore(0x00, returnData)
            return(0x00, returnDataLength)
        }
    }
}
