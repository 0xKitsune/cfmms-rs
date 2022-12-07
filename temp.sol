contract A {
    function simulateAmountOutOnSqrtPriceX96(
        address token0,
        address tokenIn,
        address lpAddressAToWeth,
        uint256 amountIn,
        int24 tickSpacing,
        uint128 liquidity,
        uint24 fee
    ) internal returns (int256 amountOut) {
        ///@notice If token0 in the pool is tokenIn then set zeroForOne to true.
        bool zeroForOne = token0 == tokenIn ? true : false;

        ///@notice Grab the current price and the current tick in the pool.
        (uint160 sqrtPriceX96, int24 initialTick, , , , , ) = IUniswapV3Pool(
            lpAddressAToWeth
        ).slot0();

        ///@notice Set the sqrtPriceLimit to Min or Max sqrtRatio
        uint160 sqrtPriceLimitX96 = zeroForOne
            ? TickMath.MIN_SQRT_RATIO + 1
            : TickMath.MAX_SQRT_RATIO - 1;

        ///@notice Initialize the initial simulation state
        CurrentState memory currentState = CurrentState({
            sqrtPriceX96: sqrtPriceX96,
            amountCalculated: 0,
            amountSpecifiedRemaining: int256(amountIn),
            tick: initialTick,
            liquidity: liquidity
        });

        ///@notice While the current state still has an amount to swap continue.
        while (currentState.amountSpecifiedRemaining != 0) {
            ///@notice Initialize step structure.
            StepComputations memory step;
            ///@notice Set sqrtPriceStartX96.
            step.sqrtPriceStartX96 = currentState.sqrtPriceX96;
            ///@notice Set the tickNext, and if the tick is initialized.
            (step.tickNext, step.initialized) = tickBitmap
                .nextInitializedTickWithinOneWord(
                    currentState.tick,
                    tickSpacing,
                    zeroForOne
                );
            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            if (step.tickNext < TickMath.MIN_TICK) {
                step.tickNext = TickMath.MIN_TICK;
            } else if (step.tickNext > TickMath.MAX_TICK) {
                step.tickNext = TickMath.MAX_TICK;
            }
            ///@notice Set the next sqrtPrice of the step.
            step.sqrtPriceNextX96 = TickMath.getSqrtRatioAtTick(step.tickNext);
            ///@notice Perform the swap step on the current tick.
            (
                currentState.sqrtPriceX96,
                step.amountIn,
                step.amountOut,
                step.feeAmount
            ) = SwapMath.computeSwapStep(
                currentState.sqrtPriceX96,
                (
                    zeroForOne
                        ? step.sqrtPriceNextX96 < sqrtPriceLimitX96
                        : step.sqrtPriceNextX96 > sqrtPriceLimitX96
                )
                    ? sqrtPriceLimitX96
                    : step.sqrtPriceNextX96,
                currentState.liquidity,
                currentState.amountSpecifiedRemaining,
                fee
            );
            ///@notice Decrement the remaining amount to be swapped by the amount available within the tick range.
            currentState.amountSpecifiedRemaining -= (step.amountIn +
                step.feeAmount).toInt256();
            ///@notice Increment amountCalculated by the amount recieved in the tick range.
            currentState.amountCalculated -= step.amountOut.toInt256();
            ///@notice If the swap step crossed into the next tick, and that tick is initialized.
            if (currentState.sqrtPriceX96 == step.sqrtPriceNextX96) {
                if (step.initialized) {
                    ///@notice Get the net liquidity after crossing the tick.
                    int128 liquidityNet = ticks.cross(step.tickNext);
                    ///@notice If swapping token0 for token1 then negate the liquidtyNet.

                    if (zeroForOne) liquidityNet = -liquidityNet;

                    currentState.liquidity = LiquidityMath.addDelta(
                        currentState.liquidity,
                        liquidityNet
                    );
                }
                ///@notice Update the currentStates tick.
                unchecked {
                    currentState.tick = zeroForOne
                        ? step.tickNext - 1
                        : step.tickNext;
                }
                ///@notice If sqrtPriceX96 in the currentState is not equal to the projected next tick, then recompute the currentStates tick.
            } else if (currentState.sqrtPriceX96 != step.sqrtPriceStartX96) {
                currentState.tick = TickMath.getTickAtSqrtRatio(
                    currentState.sqrtPriceX96
                );
            }
        }
        ///@notice Return the simulated amount out as a negative value representing the amount recieved in the swap.
        return currentState.amountCalculated;
    }
}
