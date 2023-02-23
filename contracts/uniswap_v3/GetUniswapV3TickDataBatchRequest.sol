//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 @dev This contract is not meant to be deployed. Instead, use a static call with the
      deployment bytecode as payload.
 */
contract GetUniswapV3TickDataBatchRequest {
    struct V3TickData {
        uint24[] initializedTicks;
        uint128[] liquidityNets;
    }

    constructor() {
        //Take in some amount of values for:
        // How many words to get
        // Every initialized tick within all of the words returned
        // Array of same size corresponding to the liquidity nets of every initialized tick
    }

    function position(int24 tick)
        private
        pure
        returns (int16 wordPos, uint8 bitPos)
    {
        wordPos = int16(tick >> 8);
        bitPos = uint8(tick % 256);
    }

    function nextInitializedTickWithinOneWord(
        address pool,
        int24 tick,
        int24 tickSpacing,
        bool lte
    ) internal view returns (int24 next, bool initialized) {
        int24 compressed = tick / tickSpacing;
        if (tick < 0 && tick % tickSpacing != 0) compressed--; // round towards negative infinity

        if (lte) {
            (int16 wordPos, uint8 bitPos) = position(compressed);
            // all the 1s at or to the right of the current bitPos
            uint256 mask = (1 << bitPos) - 1 + (1 << bitPos);

            ///TODO: Make an external call to the tickBitmap[wordPos] for this
            uint256 masked = self[wordPos] & mask;

            // if there are no initialized ticks to the right of or at the current tick, return rightmost in the word
            initialized = masked != 0;
            // overflow/underflow is possible, but prevented externally by limiting both tickSpacing and tick
            next = initialized
                ? (compressed -
                    int24(bitPos - BitMath.mostSignificantBit(masked))) *
                    tickSpacing
                : (compressed - int24(bitPos)) * tickSpacing;
        } else {
            // start from the word of the next tick, since the current tick state doesn't matter
            (int16 wordPos, uint8 bitPos) = position(compressed + 1);
            // all the 1s at or to the left of the bitPos
            uint256 mask = ~((1 << bitPos) - 1);
            ///TODO: Make an external call to the tickBitmap[wordPos] for this
            uint256 masked = self[wordPos] & mask;

            // if there are no initialized ticks to the left of the current tick, return leftmost in the word
            initialized = masked != 0;
            // overflow/underflow is possible, but prevented externally by limiting both tickSpacing and tick
            next = initialized
                ? (compressed +
                    1 +
                    int24(BitMath.leastSignificantBit(masked) - bitPos)) *
                    tickSpacing
                : (compressed + 1 + int24(type(uint8).max - bitPos)) *
                    tickSpacing;
        }
    }
}
