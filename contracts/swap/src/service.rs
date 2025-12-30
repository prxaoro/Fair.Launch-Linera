#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use fair_launch_abi::SwapAbi;
use linera_sdk::{abi::WithServiceAbi, views::View, Service, ServiceRuntime};
use primitive_types::U256;
use std::sync::Arc;

use crate::state::SwapState;

/// GraphQL service for querying swap pools
pub struct SwapService {
    state: Arc<SwapState>,
}

linera_sdk::service!(SwapService);

impl WithServiceAbi for SwapService {
    type Abi = SwapAbi;
}

impl Service for SwapService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = SwapState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load swap state");
        SwapService {
            state: Arc::new(state),
        }
    }

    async fn handle_query(&self, request: async_graphql::Request) -> async_graphql::Response {
        let schema = Schema::build(
            QueryRoot {
                state: self.state.clone(),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish();

        schema.execute(request).await
    }
}

pub struct QueryRoot {
    state: Arc<SwapState>,
}

#[derive(SimpleObject)]
pub struct SwapStats {
    /// Total number of pools
    pub total_pools: u64,

    /// Total value locked across all pools
    pub total_tvl: String,

    /// Average pool size
    pub average_pool_tvl: String,
}

#[derive(SimpleObject)]
pub struct PoolDetails {
    /// Pool information
    pub pool: fair_launch_abi::PoolInfoGQL,

    /// Whether pool is active
    pub is_active: bool,

    /// Pool age in seconds
    pub age_seconds: u64,
}

#[Object]
impl QueryRoot {
    /// Get overall swap statistics
    async fn stats(&self) -> SwapStats {
        let total_pools = *self.state.total_pools.get();
        let total_tvl = *self.state.total_tvl.get();

        let average_pool_tvl = if total_pools > 0 {
            total_tvl / U256::from(total_pools)
        } else {
            U256::zero()
        };

        SwapStats {
            total_pools,
            total_tvl: total_tvl.to_string(),
            average_pool_tvl: average_pool_tvl.to_string(),
        }
    }

    /// List all pools with pagination
    async fn pools(&self, offset: Option<i32>, limit: Option<i32>) -> Vec<fair_launch_abi::PoolInfoGQL> {
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(20).max(1).min(100) as usize;

        self.state
            .get_all_pools(offset, limit)
            .await
            .unwrap_or_default()
            .iter()
            .map(|p| p.into())
            .collect()
    }

    /// Get pool by pool ID
    async fn pool(&self, pool_id: String) -> Option<PoolDetails> {
        let pool = self.state.get_pool(&pool_id).await.ok()??;

        let current_time = linera_sdk::linera_base_types::Timestamp::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        );

        let age_seconds = if current_time.micros() >= pool.created_at.micros() {
            (current_time.micros() - pool.created_at.micros()) / 1_000_000
        } else {
            0
        };

        Some(PoolDetails {
            pool: (&pool).into(),
            is_active: true, // All pools are always active (locked)
            age_seconds,
        })
    }

    /// Get pool by token ID
    async fn pool_by_token(&self, token_id: String) -> Option<PoolDetails> {
        let pool = self.state.get_pool_by_token(&token_id).await.ok()??;

        let current_time = linera_sdk::linera_base_types::Timestamp::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        );

        let age_seconds = if current_time.micros() >= pool.created_at.micros() {
            (current_time.micros() - pool.created_at.micros()) / 1_000_000
        } else {
            0
        };

        Some(PoolDetails {
            pool: (&pool).into(),
            is_active: true,
            age_seconds,
        })
    }

    /// Check if token has graduated (has a pool)
    async fn has_graduated(&self, token_id: String) -> bool {
        self.state.has_pool(&token_id).await.unwrap_or(false)
    }

    /// Get top pools by TVL
    async fn top_pools_by_tvl(&self, limit: Option<i32>) -> Vec<fair_launch_abi::PoolInfoGQL> {
        let limit = limit.unwrap_or(10).max(1).min(50) as usize;

        // Get all pools (in production, this would use an indexed view)
        let mut pools = self
            .state
            .get_all_pools(0, 1000)
            .await
            .unwrap_or_default();

        // Sort by TVL descending
        pools.sort_by(|a, b| b.tvl.cmp(&a.tvl));

        // Take top N
        pools.truncate(limit);

        pools.iter().map(|p| p.into()).collect()
    }

    /// Get recently created pools
    async fn recent_pools(&self, limit: Option<i32>) -> Vec<fair_launch_abi::PoolInfoGQL> {
        let limit = limit.unwrap_or(10).max(1).min(50) as usize;

        // Get pools (already in creation order from MapView)
        let mut pools = self
            .state
            .get_all_pools(0, limit)
            .await
            .unwrap_or_default();

        // Reverse to get most recent first
        pools.reverse();

        pools.iter().map(|p| p.into()).collect()
    }

    /// Get locked liquidity summary
    async fn locked_liquidity_summary(&self) -> LockedLiquiditySummary {
        let total_pools = *self.state.total_pools.get();
        let total_tvl = *self.state.total_tvl.get();

        // All pools in Fair Launch are permanently locked
        LockedLiquiditySummary {
            total_locked_pools: total_pools,
            total_locked_tvl: total_tvl.to_string(),
            permanently_locked_pools: total_pools,
            temporarily_locked_pools: 0,
        }
    }
}

#[derive(SimpleObject)]
pub struct LockedLiquiditySummary {
    /// Total number of locked pools
    pub total_locked_pools: u64,

    /// Total TVL in locked pools
    pub total_locked_tvl: String,

    /// Number of permanently locked pools
    pub permanently_locked_pools: u64,

    /// Number of temporarily locked pools
    pub temporarily_locked_pools: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SwapState;
    use linera_sdk::linera_base_types::Timestamp;
    use linera_views::memory::MemoryContext;

    #[tokio::test]
    async fn test_stats_query() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Create some pools
        for i in 0..3 {
            let token_id = format!("token-{}", i);
            state
                .create_pool(
                    token_id,
                    U256::from(1_000_000),
                    U256::from(10_000),
                    created_at,
                )
                .await
                .unwrap();
        }

        let query_root = QueryRoot {
            state: Arc::new(state),
        };

        let stats = query_root.stats().await;
        assert_eq!(stats.total_pools, 3);
        assert!(U256::from_dec_str(&stats.total_tvl).unwrap() > U256::zero());
    }

    #[tokio::test]
    async fn test_pool_queries() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "test-token-query".to_string();
        let pool = state
            .create_pool(
                token_id.clone(),
                U256::from(1_000_000),
                U256::from(10_000),
                created_at,
            )
            .await
            .unwrap();

        let query_root = QueryRoot {
            state: Arc::new(state),
        };

        // Test pool by ID
        let result = query_root.pool(pool.pool_id.clone()).await;
        assert!(result.is_some());
        let details = result.unwrap();
        assert_eq!(details.pool.token_id, token_id);
        assert!(details.is_active);

        // Test pool by token
        let result = query_root.pool_by_token(token_id.clone()).await;
        assert!(result.is_some());
        let details = result.unwrap();
        assert_eq!(details.pool.pool_id, pool.pool_id);

        // Test has_graduated
        assert!(query_root.has_graduated(token_id).await);
        assert!(!query_root.has_graduated("non-existent".to_string()).await);
    }

    #[tokio::test]
    async fn test_pool_listing() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Create 5 pools
        for i in 0..5 {
            let token_id = format!("token-{}", i);
            state
                .create_pool(
                    token_id,
                    U256::from(1_000_000 * (i + 1)),
                    U256::from(10_000 * (i + 1)),
                    created_at,
                )
                .await
                .unwrap();
        }

        let query_root = QueryRoot {
            state: Arc::new(state),
        };

        // Test pagination
        let pools = query_root.pools(Some(0), Some(3)).await;
        assert_eq!(pools.len(), 3);

        let pools = query_root.pools(Some(3), Some(10)).await;
        assert_eq!(pools.len(), 2);

        // Test all pools
        let pools = query_root.pools(None, Some(100)).await;
        assert_eq!(pools.len(), 5);
    }

    #[tokio::test]
    async fn test_top_pools_by_tvl() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Create pools with different TVLs
        let tvls = vec![100_000, 50_000, 200_000, 75_000, 150_000];
        for (i, &tvl) in tvls.iter().enumerate() {
            let token_id = format!("token-{}", i);
            state
                .create_pool(
                    token_id,
                    U256::from(1_000_000),
                    U256::from(tvl / 2), // TVL = 2 * total_raised
                    created_at,
                )
                .await
                .unwrap();
        }

        let query_root = QueryRoot {
            state: Arc::new(state),
        };

        let top_pools = query_root.top_pools_by_tvl(Some(3)).await;
        assert_eq!(top_pools.len(), 3);

        // Verify sorted by TVL descending
        assert!(top_pools[0].tvl >= top_pools[1].tvl);
        assert!(top_pools[1].tvl >= top_pools[2].tvl);
    }

    #[tokio::test]
    async fn test_locked_liquidity_summary() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Create pools
        for i in 0..3 {
            let token_id = format!("token-{}", i);
            state
                .create_pool(
                    token_id,
                    U256::from(1_000_000),
                    U256::from(10_000),
                    created_at,
                )
                .await
                .unwrap();
        }

        let query_root = QueryRoot {
            state: Arc::new(state),
        };

        let summary = query_root.locked_liquidity_summary().await;
        assert_eq!(summary.total_locked_pools, 3);
        assert_eq!(summary.permanently_locked_pools, 3);
        assert_eq!(summary.temporarily_locked_pools, 0);
        assert!(U256::from_dec_str(&summary.total_locked_tvl).unwrap() > U256::zero());
    }
}
