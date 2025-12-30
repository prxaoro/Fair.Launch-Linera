#![cfg(test)]

use crate::state::FactoryState;
use fair_launch_abi::{BondingCurveConfig, TokenMetadata};
use linera_sdk::linera_base_types::{AccountOwner, ChainId, Timestamp};
use linera_views::memory::MemoryContext;

/// Helper function to create test metadata
fn create_test_metadata(name: &str, symbol: &str) -> TokenMetadata {
    TokenMetadata {
        name: name.to_string(),
        symbol: symbol.to_string(),
        description: format!("Test token: {}", name),
        image_url: Some("https://example.com/image.png".to_string()),
        twitter: Some(format!("@{}", symbol.to_lowercase())),
        telegram: None,
        website: Some(format!("https://{}.com", symbol.to_lowercase())),
    }
}

#[tokio::test]
async fn test_factory_state_initialization() {
    let context = MemoryContext::default();
    let state = FactoryState::load(context).await.unwrap();

    assert_eq!(state.get_token_count(), 0);
}

#[tokio::test]
async fn test_multiple_token_creation() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);

    // Create 5 tokens
    for i in 0..5 {
        let metadata = create_test_metadata(&format!("Token {}", i), &format!("TKN{}", i));

        state
            .register_token(
                format!("token-{}", i),
                creator,
                metadata,
                curve_config.clone(),
                created_at,
            )
            .await
            .unwrap();
    }

    assert_eq!(state.get_token_count(), 5);

    // Verify all tokens exist
    for i in 0..5 {
        let token = state.get_token(&format!("token-{}", i)).await.unwrap();
        assert_eq!(token.metadata.name, format!("Token {}", i));
    }
}

#[tokio::test]
async fn test_creator_registry() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator1 = AccountOwner::from(ChainId::root(1));
    let creator2 = AccountOwner::from(ChainId::root(2));
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);

    // Creator 1 creates 3 tokens
    for i in 0..3 {
        let metadata = create_test_metadata(&format!("Token {}", i), &format!("TKN{}", i));
        state
            .register_token(
                format!("creator1-token-{}", i),
                creator1,
                metadata,
                curve_config.clone(),
                created_at,
            )
            .await
            .unwrap();
    }

    // Creator 2 creates 2 tokens
    for i in 0..2 {
        let metadata = create_test_metadata(&format!("Token {}", i), &format!("TKN{}", i));
        state
            .register_token(
                format!("creator2-token-{}", i),
                creator2,
                metadata,
                curve_config.clone(),
                created_at,
            )
            .await
            .unwrap();
    }

    // Verify creator 1 has 3 tokens
    let creator1_tokens = state.get_tokens_by_creator(&creator1).await.unwrap();
    assert_eq!(creator1_tokens.len(), 3);

    // Verify creator 2 has 2 tokens
    let creator2_tokens = state.get_tokens_by_creator(&creator2).await.unwrap();
    assert_eq!(creator2_tokens.len(), 2);
}

#[tokio::test]
async fn test_token_metrics_update() {
    use primitive_types::U256;

    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let metadata = create_test_metadata("Test Token", "TEST");
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);
    let token_id = "test-token";

    // Register token
    state
        .register_token(
            token_id.to_string(),
            creator,
            metadata,
            curve_config,
            created_at,
        )
        .await
        .unwrap();

    // Update metrics
    let new_supply = U256::from(1000);
    let new_raised = U256::from(500);

    state
        .update_token_metrics(token_id, new_supply, new_raised)
        .await
        .unwrap();

    // Verify update
    let token = state.get_token(token_id).await.unwrap();
    assert_eq!(token.current_supply, new_supply);
    assert_eq!(token.total_raised, new_raised);
}

#[tokio::test]
async fn test_token_graduation() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let metadata = create_test_metadata("Test Token", "TEST");
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);
    let token_id = "test-token";

    // Register token
    state
        .register_token(
            token_id.to_string(),
            creator,
            metadata,
            curve_config,
            created_at,
        )
        .await
        .unwrap();

    // Initially not graduated
    let token = state.get_token(token_id).await.unwrap();
    assert!(!token.is_graduated);
    assert!(token.dex_pool_id.is_none());

    // Graduate token
    let pool_id = "pool-123".to_string();
    state
        .update_token_status(token_id, true, Some(pool_id.clone()))
        .await
        .unwrap();

    // Verify graduation
    let token = state.get_token(token_id).await.unwrap();
    assert!(token.is_graduated);
    assert_eq!(token.dex_pool_id, Some(pool_id));
}

#[tokio::test]
async fn test_pagination_boundary_cases() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);

    // Create 3 tokens
    for i in 0..3 {
        let metadata = create_test_metadata(&format!("Token {}", i), &format!("TKN{}", i));
        state
            .register_token(
                format!("token-{}", i),
                creator,
                metadata,
                curve_config.clone(),
                created_at,
            )
            .await
            .unwrap();
    }

    // Test offset beyond total
    let tokens = state.get_all_tokens(10, 5).await.unwrap();
    assert_eq!(tokens.len(), 0);

    // Test limit larger than remaining
    let tokens = state.get_all_tokens(1, 10).await.unwrap();
    assert_eq!(tokens.len(), 2); // Only 2 tokens after offset 1

    // Test zero limit
    let tokens = state.get_all_tokens(0, 0).await.unwrap();
    assert_eq!(tokens.len(), 0);
}

#[tokio::test]
async fn test_metadata_validation_edge_cases() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);

    // Test name with only whitespace
    let mut metadata = create_test_metadata("   ", "TEST");
    let result = state
        .register_token(
            "token-1".to_string(),
            creator,
            metadata.clone(),
            curve_config.clone(),
            created_at,
        )
        .await;
    assert!(result.is_err());

    // Test very long name
    metadata = create_test_metadata(&"A".repeat(101), "TEST");
    let result = state
        .register_token(
            "token-2".to_string(),
            creator,
            metadata.clone(),
            curve_config.clone(),
            created_at,
        )
        .await;
    assert!(result.is_err());

    // Test very long symbol
    metadata = create_test_metadata("Test", &"T".repeat(21));
    let result = state
        .register_token(
            "token-3".to_string(),
            creator,
            metadata,
            curve_config,
            created_at,
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_url_formats() {
    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);

    // Test invalid image URL
    let mut metadata = create_test_metadata("Test", "TEST");
    metadata.image_url = Some("not-a-url".to_string());

    let result = state
        .register_token(
            "token-1".to_string(),
            creator,
            metadata.clone(),
            curve_config.clone(),
            created_at,
        )
        .await;
    assert!(result.is_err());

    // Test invalid website URL
    metadata = create_test_metadata("Test", "TEST");
    metadata.website = Some("ftp://invalid.com".to_string());

    let result = state
        .register_token("token-2".to_string(), creator, metadata, curve_config, created_at)
        .await;
    assert!(result.is_err());
}

/// Integration test: Simulate complete token lifecycle
#[tokio::test]
async fn test_token_lifecycle() {
    use primitive_types::U256;

    let context = MemoryContext::default();
    let mut state = FactoryState::load(context).await.unwrap();

    let creator = AccountOwner::from(ChainId::root(0));
    let metadata = create_test_metadata("Lifecycle Token", "LIFE");
    let curve_config = BondingCurveConfig::default();
    let created_at = Timestamp::from(0);
    let token_id = "lifecycle-token";

    // Step 1: Create token
    state
        .register_token(
            token_id.to_string(),
            creator,
            metadata.clone(),
            curve_config.clone(),
            created_at,
        )
        .await
        .unwrap();

    let token = state.get_token(token_id).await.unwrap();
    assert_eq!(token.current_supply, U256::zero());
    assert_eq!(token.total_raised, U256::zero());
    assert!(!token.is_graduated);

    // Step 2: Simulate trading (update metrics)
    state
        .update_token_metrics(token_id, U256::from(500_000), U256::from(10_000))
        .await
        .unwrap();

    let token = state.get_token(token_id).await.unwrap();
    assert_eq!(token.current_supply, U256::from(500_000));
    assert_eq!(token.total_raised, U256::from(10_000));

    // Step 3: More trading
    state
        .update_token_metrics(token_id, U256::from(1_000_000), U256::from(69_000))
        .await
        .unwrap();

    // Step 4: Graduate to DEX
    state
        .update_token_status(token_id, true, Some("dex-pool-xyz".to_string()))
        .await
        .unwrap();

    let token = state.get_token(token_id).await.unwrap();
    assert_eq!(token.current_supply, U256::from(1_000_000));
    assert_eq!(token.total_raised, U256::from(69_000));
    assert!(token.is_graduated);
    assert_eq!(token.dex_pool_id, Some("dex-pool-xyz".to_string()));

    // Verify creator registry
    let creator_tokens = state.get_tokens_by_creator(&creator).await.unwrap();
    assert_eq!(creator_tokens.len(), 1);
    assert_eq!(creator_tokens[0].token_id, token_id);
}
