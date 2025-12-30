#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use fair_launch_abi::{FactoryAbi, TokenLaunch, TokenMetadata};
use linera_sdk::{
    abi::WithServiceAbi,
    views::View,
    Service, ServiceRuntime,
};
use primitive_types::U256;
use std::sync::Arc;

use crate::state::FactoryState;

pub struct FactoryService {
    state: Arc<FactoryState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(FactoryService);

impl WithServiceAbi for FactoryService {
    type Abi = FactoryAbi;
}

impl Service for FactoryService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = FactoryState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load factory state");
        FactoryService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: async_graphql::Request) -> async_graphql::Response {
        let schema = Schema::build(
            QueryRoot::default(),
            EmptyMutation,
            EmptySubscription,
        )
        .data(self.state.clone())
        .finish();

        schema.execute(request).await
    }
}

/// GraphQL query root
#[derive(Default)]
struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get total number of tokens created
    async fn token_count(&self, ctx: &Context<'_>) -> u64 {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");
        state.get_token_count()
    }

    /// Get a specific token by its ID (ChainId)
    async fn token(&self, ctx: &Context<'_>, token_id: String) -> Option<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        match state.get_token(&token_id).await {
            Ok(token) => Some(TokenLaunchView::from(token)),
            Err(e) => {
                log::warn!("Failed to get token {}: {}", token_id, e);
                None
            }
        }
    }

    /// Get all tokens with pagination
    async fn tokens(
        &self,
        ctx: &Context<'_>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(20).min(100); // Max 100 per query

        match state.get_all_tokens(offset, limit).await {
            Ok(tokens) => tokens.into_iter().map(TokenLaunchView::from).collect(),
            Err(e) => {
                log::error!("Failed to get tokens: {}", e);
                Vec::new()
            }
        }
    }

    /// Get all tokens created by a specific creator
    async fn tokens_by_creator(
        &self,
        ctx: &Context<'_>,
        creator_json: String,
    ) -> Vec<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        // Parse creator Account from JSON
        let creator_account: linera_sdk::linera_base_types::Account = match serde_json::from_str(&creator_json) {
            Ok(account) => account,
            Err(e) => {
                log::warn!("Invalid creator Account format: {}", e);
                return Vec::new();
            }
        };

        match state.get_tokens_by_creator(&creator_account).await {
            Ok(tokens) => tokens.into_iter().map(TokenLaunchView::from).collect(),
            Err(e) => {
                log::error!("Failed to get tokens by creator: {}", e);
                Vec::new()
            }
        }
    }

    /// Get recent token launches
    async fn recent_tokens(&self, ctx: &Context<'_>, limit: Option<u64>) -> Vec<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        let total_count = state.get_token_count();
        let limit = limit.unwrap_or(10).min(50);

        // Get most recent tokens (from end of list)
        let offset = if total_count > limit {
            total_count - limit
        } else {
            0
        };

        match state.get_all_tokens(offset, limit).await {
            Ok(mut tokens) => {
                // Reverse to get newest first
                tokens.reverse();
                tokens.into_iter().map(TokenLaunchView::from).collect()
            }
            Err(e) => {
                log::error!("Failed to get recent tokens: {}", e);
                Vec::new()
            }
        }
    }

    /// Get graduated tokens (completed bonding curves)
    async fn graduated_tokens(
        &self,
        ctx: &Context<'_>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(20).min(100);

        match state.get_all_tokens(offset, limit * 2).await {
            Ok(tokens) => tokens
                .into_iter()
                .filter(|t| t.is_graduated)
                .map(TokenLaunchView::from)
                .take(limit as usize)
                .collect(),
            Err(e) => {
                log::error!("Failed to get graduated tokens: {}", e);
                Vec::new()
            }
        }
    }

    /// Search tokens by name or symbol
    async fn search_tokens(&self, ctx: &Context<'_>, query: String) -> Vec<TokenLaunchView> {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        let query_lower = query.to_lowercase();

        // Get all tokens and filter
        // In a production system, this would use an index for better performance
        match state.get_all_tokens(0, 1000).await {
            Ok(tokens) => tokens
                .into_iter()
                .filter(|t| {
                    t.metadata.name.to_lowercase().contains(&query_lower)
                        || t.metadata.symbol.to_lowercase().contains(&query_lower)
                })
                .take(20)
                .map(TokenLaunchView::from)
                .collect(),
            Err(e) => {
                log::error!("Failed to search tokens: {}", e);
                Vec::new()
            }
        }
    }

    /// Get factory statistics
    async fn stats(&self, ctx: &Context<'_>) -> FactoryStats {
        let state = ctx.data::<Arc<FactoryState>>().expect("State not found");

        let total_tokens = state.get_token_count();

        // Calculate total value locked by iterating all tokens
        // In production, this should be cached/indexed
        let mut total_value_locked = U256::zero();
        let mut graduated_count = 0;

        if let Ok(tokens) = state.get_all_tokens(0, total_tokens).await {
            for token in tokens {
                total_value_locked += token.total_raised;
                if token.is_graduated {
                    graduated_count += 1;
                }
            }
        }

        FactoryStats {
            total_tokens,
            graduated_count,
            active_count: total_tokens - graduated_count,
            total_value_locked: format!("{}", total_value_locked),
        }
    }
}

/// GraphQL view of TokenLaunch (for serialization compatibility)
#[derive(SimpleObject)]
struct TokenLaunchView {
    token_id: String,
    creator: String,
    metadata: TokenMetadata,
    curve_config: fair_launch_abi::BondingCurveConfigGQL,
    current_supply: String,
    total_raised: String,
    is_graduated: bool,
    created_at: String,
    dex_pool_id: Option<String>,
}

impl From<TokenLaunch> for TokenLaunchView {
    fn from(token: TokenLaunch) -> Self {
        TokenLaunchView {
            token_id: token.token_id,
            creator: format!("{:?}", token.creator),
            metadata: token.metadata,
            curve_config: (&token.curve_config).into(),
            current_supply: format!("{}", token.current_supply),
            total_raised: format!("{}", token.total_raised),
            is_graduated: token.is_graduated,
            created_at: format!("{}", token.created_at.micros()),
            dex_pool_id: token.dex_pool_id,
        }
    }
}

/// Factory statistics
#[derive(SimpleObject)]
struct FactoryStats {
    total_tokens: u64,
    graduated_count: u64,
    active_count: u64,
    total_value_locked: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_launch_view_conversion() {
        use fair_launch_abi::BondingCurveConfig;
        use linera_sdk::linera_base_types::{Account, AccountOwner, ChainId, Timestamp};

        let token = TokenLaunch {
            token_id: "test-123".to_string(),
            creator: Account {
                chain_id: ChainId::root(0),
                owner: AccountOwner::CHAIN,
            },
            metadata: TokenMetadata {
                name: "Test".to_string(),
                symbol: "TEST".to_string(),
                description: "Test token".to_string(),
                image_url: None,
                twitter: None,
                telegram: None,
                website: None,
            },
            curve_config: BondingCurveConfig::default(),
            current_supply: U256::from(1000),
            total_raised: U256::from(500),
            is_graduated: false,
            created_at: Timestamp::from(0),
            dex_pool_id: None,
        };

        let view = TokenLaunchView::from(token);
        assert_eq!(view.token_id, "test-123");
        assert_eq!(view.current_supply, "1000");
        assert_eq!(view.total_raised, "500");
    }
}
