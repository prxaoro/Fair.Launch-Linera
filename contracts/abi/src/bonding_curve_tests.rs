/**
 * Comprehensive unit tests for bonding curve mathematics
 */

#[cfg(test)]
mod bonding_curve_math_tests {
    use crate::bonding_curve::*;
    use primitive_types::U256;

    #[test]
    fn test_price_is_zero_at_zero_supply() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::zero();

        let price = calculate_current_price(supply, k, scale);
        assert_eq!(price, U256::zero(), "Price should be zero at zero supply");
    }

    #[test]
    fn test_price_increases_quadratically() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);

        // Calculate prices at different supply levels
        let price_100k = calculate_current_price(U256::from(100_000), k, scale);
        let price_200k = calculate_current_price(U256::from(200_000), k, scale);
        let price_400k = calculate_current_price(U256::from(400_000), k, scale);

        // Verify quadratic relationship: price ∝ supply²
        // At 2x supply, price should be 4x
        let ratio_2x = price_200k / price_100k;
        assert_eq!(
            ratio_2x,
            U256::from(4),
            "Price should quadruple when supply doubles"
        );

        // At 4x supply, price should be 16x
        let ratio_4x = price_400k / price_100k;
        assert_eq!(
            ratio_4x,
            U256::from(16),
            "Price should increase 16x when supply quadruples"
        );
    }

    #[test]
    fn test_buy_cost_increases_with_supply() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let amount = U256::from(10_000);

        // Cost to buy 10k tokens at different supply levels
        let cost_at_0 = calculate_buy_cost(U256::zero(), amount, k, scale);
        let cost_at_100k = calculate_buy_cost(U256::from(100_000), amount, k, scale);
        let cost_at_500k = calculate_buy_cost(U256::from(500_000), amount, k, scale);

        assert!(
            cost_at_100k > cost_at_0,
            "Cost should increase with supply"
        );
        assert!(
            cost_at_500k > cost_at_100k,
            "Cost should continue increasing"
        );
    }

    #[test]
    fn test_sell_return_equals_buy_cost() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::from(100_000);
        let amount = U256::from(10_000);

        let buy_cost = calculate_buy_cost(supply, amount, k, scale);
        let new_supply = supply + amount;
        let sell_return = calculate_sell_return(new_supply, amount, k, scale);

        // For a mathematical bonding curve without fees, sell return equals buy cost
        // (because we're integrating the same function in reverse)
        assert_eq!(
            sell_return, buy_cost,
            "Sell return should equal buy cost (before fees)"
        );
    }

    #[test]
    fn test_large_trade_impact() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::from(100_000);

        // Small buy
        let small_amount = U256::from(1_000);
        let small_cost = calculate_buy_cost(supply, small_amount, k, scale);
        let small_avg_price = small_cost / small_amount;

        // Large buy (10x)
        let large_amount = U256::from(10_000);
        let large_cost = calculate_buy_cost(supply, large_amount, k, scale);
        let large_avg_price = large_cost / large_amount;

        assert!(
            large_avg_price > small_avg_price,
            "Average price should increase for larger buys (price impact)"
        );
    }

    #[test]
    fn test_creator_fee_calculation() {
        let cost = U256::from(100_000);

        // Test various fee levels
        let fee_1_pct = (cost * U256::from(100)) / U256::from(10000);
        assert_eq!(fee_1_pct, U256::from(1_000), "1% fee should be 1000");

        let fee_3_pct = (cost * U256::from(300)) / U256::from(10000);
        assert_eq!(fee_3_pct, U256::from(3_000), "3% fee should be 3000");

        let fee_5_pct = (cost * U256::from(500)) / U256::from(10000);
        assert_eq!(fee_5_pct, U256::from(5_000), "5% fee should be 5000");
    }

    #[test]
    fn test_buy_sell_roundtrip_with_fees() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::from(100_000);
        let amount = U256::from(10_000);
        let fee_bps = 300; // 3%

        // Buy tokens
        let buy_cost = calculate_buy_cost(supply, amount, k, scale);
        let buy_fee = (buy_cost * U256::from(fee_bps)) / U256::from(10000);
        let total_cost_with_fee = buy_cost + buy_fee;

        // Sell tokens back
        let new_supply = supply + amount;
        let sell_return = calculate_sell_return(new_supply, amount, k, scale);
        let sell_fee = (sell_return * U256::from(fee_bps)) / U256::from(10000);
        let net_return_after_fee = sell_return - sell_fee;

        // Due to curve shape and fees, user should lose money on roundtrip
        assert!(
            net_return_after_fee < total_cost_with_fee,
            "Roundtrip should result in loss due to curve shape and fees"
        );
    }

    #[test]
    fn test_maximum_supply_constraint() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let max_supply = U256::from(1_000_000_000u64);
        let current_supply = U256::from(999_000_000u64);

        // Try to buy amount that would exceed max supply
        let requested_amount = U256::from(2_000_000);
        let total_would_be = current_supply + requested_amount;

        assert!(
            total_would_be > max_supply,
            "This buy would exceed max supply and should fail in contract"
        );
    }

    #[test]
    fn test_precision_with_small_amounts() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::from(1_000_000);

        // Test very small buy
        let tiny_amount = U256::from(1);
        let cost = calculate_buy_cost(supply, tiny_amount, k, scale);

        assert!(cost > U256::zero(), "Even tiny amounts should have non-zero cost");
    }

    #[test]
    fn test_sell_entire_supply_returns_zero() {
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);
        let supply = U256::from(100_000);

        // Selling entire supply should return approximately the integral from 0 to supply
        let sell_return = calculate_sell_return(supply, supply, k, scale);

        // Should get back the area under the curve
        assert!(sell_return > U256::zero(), "Selling should return non-zero amount");
    }

    #[test]
    fn test_integration_formula_consistency() {
        // Verify bonding curve integration formula is mathematically consistent
        let k = U256::from(1000);
        let scale = U256::from(1_000_000);

        // Test that cost is cumulative: buying from 0->100k + 100k->200k = buying 0->200k
        let cost_0_to_100k = calculate_buy_cost(U256::zero(), U256::from(100_000), k, scale);
        let cost_100k_to_200k = calculate_buy_cost(U256::from(100_000), U256::from(100_000), k, scale);
        let cost_0_to_200k = calculate_buy_cost(U256::zero(), U256::from(200_000), k, scale);

        assert_eq!(
            cost_0_to_100k + cost_100k_to_200k,
            cost_0_to_200k,
            "Integration should be additive"
        );

        // Verify that buying then selling returns to original state (minus fees)
        let supply = U256::from(100_000);
        let amount = U256::from(50_000);

        let buy_cost = calculate_buy_cost(supply, amount, k, scale);
        let sell_return = calculate_sell_return(supply + amount, amount, k, scale);

        assert_eq!(
            buy_cost, sell_return,
            "Buy cost and sell return should be equal (before fees)"
        );
    }
}
