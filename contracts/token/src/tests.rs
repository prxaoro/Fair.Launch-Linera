#![cfg(test)]

use fair_launch_abi::{bonding_curve, BondingCurveConfig, TokenMetadata, TokenOperation};
use linera_sdk::linera_base_types::{AccountOwner, ChainId};
use primitive_types::U256;

use crate::state::TokenState;

// Helper to create test metadata
fn test_metadata() -> TokenMetadata {
    TokenMetadata {
        name: "Test Token".to_string(),
        symbol: "TEST".to_string(),
        description: "A test token for unit tests".to_string(),
        image_url: None,
        twitter: None,
        telegram: None,
        website: None,
    }
}

// Helper to create test config
fn test_config() -> BondingCurveConfig {
    BondingCurveConfig {
        k: U256::from(1000),
        scale: U256::from(1_000_000),
        target_raise: U256::from(69_000),
        max_supply: U256::from(1_000_000_000u64),
    }
}

mod bonding_curve_tests {
    use super::*;

    #[test]
    fn test_buy_cost_increases_with_supply() {
        let config = test_config();

        // Cost to buy first 100k tokens
        let cost1 = bonding_curve::calculate_buy_cost(
            U256::zero(),
            U256::from(100_000),
            config.k,
            config.scale,
        );

        // Cost to buy next 100k tokens (at higher supply)
        let cost2 = bonding_curve::calculate_buy_cost(
            U256::from(100_000),
            U256::from(100_000),
            config.k,
            config.scale,
        );

        assert!(cost2 > cost1, "Cost should increase as supply increases");
    }

    #[test]
    fn test_sell_return_less_than_buy_cost() {
        let config = test_config();

        // Buy 100k tokens
        let buy_cost = bonding_curve::calculate_buy_cost(
            U256::zero(),
            U256::from(100_000),
            config.k,
            config.scale,
        );

        // Sell them back immediately
        let sell_return = bonding_curve::calculate_sell_return(
            U256::from(100_000),
            U256::from(100_000),
            config.k,
            config.scale,
        );

        assert!(sell_return < buy_cost, "Sell return should be less than buy cost");
    }

    #[test]
    fn test_price_calculation() {
        let config = test_config();

        let price_at_zero = bonding_curve::calculate_current_price(
            U256::zero(),
            config.k,
            config.scale,
        );

        let price_at_million = bonding_curve::calculate_current_price(
            U256::from(1_000_000),
            config.k,
            config.scale,
        );

        assert!(price_at_million > price_at_zero, "Price should increase with supply");
    }

    #[test]
    fn test_zero_amount_returns_zero_cost() {
        let config = test_config();

        let cost = bonding_curve::calculate_buy_cost(
            U256::from(500_000),
            U256::zero(),
            config.k,
            config.scale,
        );

        assert_eq!(cost, U256::zero());
    }

    #[test]
    fn test_sell_more_than_supply_returns_zero() {
        let config = test_config();

        let return_amount = bonding_curve::calculate_sell_return(
            U256::from(100_000),
            U256::from(200_000),  // Trying to sell more than exists
            config.k,
            config.scale,
        );

        assert_eq!(return_amount, U256::zero());
    }

    #[test]
    fn test_buy_and_sell_round_trip() {
        let config = test_config();
        let amount = U256::from(50_000);

        // Buy at zero supply
        let buy_cost = bonding_curve::calculate_buy_cost(
            U256::zero(),
            amount,
            config.k,
            config.scale,
        );

        // Sell back at new supply
        let sell_return = bonding_curve::calculate_sell_return(
            amount,
            amount,
            config.k,
            config.scale,
        );

        // Sell return should equal buy cost (since we're at same supply points)
        assert_eq!(buy_cost, sell_return);
    }

    #[test]
    fn test_large_buy_increases_price_significantly() {
        let config = test_config();

        let initial_price = bonding_curve::calculate_current_price(
            U256::zero(),
            config.k,
            config.scale,
        );

        let large_amount = U256::from(10_000_000);  // 1% of max supply
        let new_price = bonding_curve::calculate_current_price(
            large_amount,
            config.k,
            config.scale,
        );

        let price_increase = new_price.saturating_sub(initial_price);
        assert!(price_increase > initial_price * U256::from(10),
                "Large buy should significantly increase price");
    }

    #[test]
    fn test_gradual_buys_vs_single_buy() {
        let config = test_config();

        // Single large buy
        let single_cost = bonding_curve::calculate_buy_cost(
            U256::zero(),
            U256::from(300_000),
            config.k,
            config.scale,
        );

        // Three smaller buys
        let cost1 = bonding_curve::calculate_buy_cost(
            U256::zero(),
            U256::from(100_000),
            config.k,
            config.scale,
        );
        let cost2 = bonding_curve::calculate_buy_cost(
            U256::from(100_000),
            U256::from(100_000),
            config.k,
            config.scale,
        );
        let cost3 = bonding_curve::calculate_buy_cost(
            U256::from(200_000),
            U256::from(100_000),
            config.k,
            config.scale,
        );
        let gradual_cost = cost1 + cost2 + cost3;

        assert_eq!(single_cost, gradual_cost, "Total cost should be same regardless of order");
    }
}

mod state_tests {
    use super::*;
    use linera_sdk::linera_base_types::Timestamp;
    use linera_views::memory::MemoryContext;

    #[tokio::test]
    async fn test_initialize_token() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        let token_id = "test-token-123".to_string();
        let creator = AccountOwner::from(ChainId::root(0));
        let metadata = test_metadata();
        let config = test_config();
        let created_at = Timestamp::from(1000);

        state.initialize(
            token_id.clone(),
            creator,
            metadata.clone(),
            config.clone(),
            created_at,
        ).await.unwrap();

        assert_eq!(state.token_id.get().as_str(), "test-token-123");
        assert_eq!(state.metadata.get().name, "Test Token");
        assert_eq!(*state.current_supply.get(), U256::zero());
        assert_eq!(*state.is_graduated.get(), false);
    }

    #[tokio::test]
    async fn test_balance_operations() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        let account1 = AccountOwner::from(ChainId::root(1));
        let account2 = AccountOwner::from(ChainId::root(2));

        // Initial balance should be zero
        assert_eq!(state.get_balance(&account1).await, U256::zero());

        // Set balance for account1
        state.set_balance(account1, U256::from(1000)).await.unwrap();
        assert_eq!(state.get_balance(&account1).await, U256::from(1000));

        // Set balance for account2
        state.set_balance(account2, U256::from(500)).await.unwrap();
        assert_eq!(state.get_balance(&account2).await, U256::from(500));

        // Update account1 balance
        state.set_balance(account1, U256::from(2000)).await.unwrap();
        assert_eq!(state.get_balance(&account1).await, U256::from(2000));
    }

    #[tokio::test]
    async fn test_holder_count() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        assert_eq!(*state.holder_count.get(), 0);

        // Add first holder
        let account1 = AccountOwner::from(ChainId::root(1));
        state.set_balance(account1, U256::from(100)).await.unwrap();
        assert_eq!(*state.holder_count.get(), 1);

        // Add second holder
        let account2 = AccountOwner::from(ChainId::root(2));
        state.set_balance(account2, U256::from(200)).await.unwrap();
        assert_eq!(*state.holder_count.get(), 2);

        // Update existing holder (count shouldn't change)
        state.set_balance(account1, U256::from(300)).await.unwrap();
        assert_eq!(*state.holder_count.get(), 2);

        // Remove holder
        state.set_balance(account1, U256::zero()).await.unwrap();
        assert_eq!(*state.holder_count.get(), 1);
    }

    #[tokio::test]
    async fn test_record_trade() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        // Initialize state
        let token_id = "test-token".to_string();
        let creator = AccountOwner::from(ChainId::root(0));
        state.initialize(
            token_id.clone(),
            creator,
            test_metadata(),
            test_config(),
            Timestamp::from(0),
        ).await.unwrap();

        let trader = AccountOwner::from(ChainId::root(1));

        let trade = fair_launch_abi::Trade {
            token_id: token_id.clone(),
            trader,
            is_buy: true,
            token_amount: U256::from(1000),
            currency_amount: U256::from(100),
            price: U256::from(1),
            timestamp: Timestamp::from(1000),
        };

        state.record_trade("trade-1".to_string(), trade).await.unwrap();

        assert_eq!(*state.trade_count.get(), 1);

        // Check user position was created
        let position = state.user_positions.get(&trader).await.unwrap().unwrap();
        assert_eq!(position.balance, U256::from(1000));
        assert_eq!(position.total_invested, U256::from(100));
        assert_eq!(position.trades_count, 1);
    }

    #[tokio::test]
    async fn test_is_curve_complete() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        let token_id = "test-token".to_string();
        let creator = AccountOwner::from(ChainId::root(0));
        let config = test_config();

        state.initialize(
            token_id,
            creator,
            test_metadata(),
            config.clone(),
            Timestamp::from(0),
        ).await.unwrap();

        // Initially not complete
        assert!(!state.is_curve_complete());

        // Set supply to max
        state.current_supply.set(config.max_supply);
        assert!(state.is_curve_complete());

        // Set supply beyond max
        state.current_supply.set(config.max_supply + U256::from(1));
        assert!(state.is_curve_complete());
    }
}

// Add more integration-style tests here
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_realistic_token_launch_scenario() {
        let config = test_config();

        // Simulate a realistic launch:
        // 1. 10 buyers each buy 10k tokens
        // 2. Total: 100k tokens sold
        // 3. Calculate total raised

        let mut current_supply = U256::zero();
        let mut total_raised = U256::zero();

        for _ in 0..10 {
            let cost = bonding_curve::calculate_buy_cost(
                current_supply,
                U256::from(10_000),
                config.k,
                config.scale,
            );

            current_supply += U256::from(10_000);
            total_raised += cost;
        }

        assert_eq!(current_supply, U256::from(100_000));
        assert!(total_raised > U256::zero());

        // Verify final price is higher than initial
        let final_price = bonding_curve::calculate_current_price(
            current_supply,
            config.k,
            config.scale,
        );
        let initial_price = bonding_curve::calculate_current_price(
            U256::zero(),
            config.k,
            config.scale,
        );

        assert!(final_price > initial_price);
    }

    #[test]
    fn test_full_curve_completion() {
        let config = test_config();

        // Calculate cost to buy entire supply
        let total_cost = bonding_curve::calculate_buy_cost(
            U256::zero(),
            config.max_supply,
            config.k,
            config.scale,
        );

        // Should be around target raise
        // Allow 10% variance due to curve math
        let target = config.target_raise;
        let lower_bound = (target * U256::from(90)) / U256::from(100);
        let upper_bound = (target * U256::from(110)) / U256::from(100);

        assert!(total_cost >= lower_bound && total_cost <= upper_bound,
                "Total cost should be close to target raise");
    }
}
