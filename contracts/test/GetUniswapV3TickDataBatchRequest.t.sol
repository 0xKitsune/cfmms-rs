// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./Console.sol";
import "./Test.sol";
import "../uniswap_v3/GetUniswapV3TickDataBatchRequest.sol";

contract GasTest is DSTest {
    function setUp() public {}

    function testBatchContract() public {
        address pool = 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640;
        bool zeroForOne = true;
        int24 currentTick = 202586;
        uint16 numTicks = 1000;
        int24 tickSpacing = 10;
        GetUniswapV3TickDataBatchRequest batchContract = new GetUniswapV3TickDataBatchRequest(
                pool,
                zeroForOne,
                currentTick,
                numTicks,
                tickSpacing
            );

        console.logBytes(address(batchContract).code);
    }
}
