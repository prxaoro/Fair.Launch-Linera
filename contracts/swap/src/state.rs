use linera_sdk::{
    linera_base_types::Timestamp,
    views::{MapView, RegisterView, RootView, ViewStorageContext},
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Pool information for a graduated token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    /// Unique pool ID
    pub pool_id: String,

    /// Token ID from the bonding curve
    pub token_id: String,

    /// Total token supply graduated to pool
    pub token_liquidity: U256,

    /// Total base currency raised during bonding curve
    pub base_liquidity: U256,

    /// Initial liquidity ratio (base_per_token)
    pub initial_ratio: U256,

    /// Timestamp when pool was created
    pub created_at: Timestamp,

    /// Whether liquidity is locked (always true for Fair Launch)
    pub is_locked: bool,

    /// Lock expiration timestamp (None = permanent lock)
    pub lock_expires_at: Option<Timestamp>,

    /// Total trades executed (0 for locked pools)
    pub trade_count: u64,

    /// Pool TVL in base currency equivalent
    pub tvl: U256,
}

impl PoolInfo {
    /// Create a new locked pool from graduated token
    pub fn new(
        token_id: String,
        total_supply: U256,
        total_raised: U256,
        created_at: Timestamp,
    ) -> Result<Self, anyhow::Error> {
        // Validate inputs
        if total_supply == U256::zero() {
            anyhow::bail!("Token supply must be greater than zero");
        }
        if total_raised == U256::zero() {
            anyhow::bail!("Total raised must be greater than zero");
        }

        // Generate pool ID from token ID
        let pool_id = format!("pool-{}", token_id);

        // Calculate initial ratio: base_per_token = total_raised / total_supply
        // Use scaled division to preserve precision
        let initial_ratio = (total_raised * U256::from(1_000_000)) / total_supply;

        // Calculate TVL (total value locked) = 2 * total_raised
        // (accounts for both token and base currency sides)
        let tvl = total_raised * U256::from(2);

        Ok(PoolInfo {
            pool_id,
            token_id,
            token_liquidity: total_supply,
            base_liquidity: total_raised,
            initial_ratio,
            created_at,
            is_locked: true,
            lock_expires_at: None, // Permanent lock
            trade_count: 0,
            tvl,
        })
    }

    /// Calculate current token price in base currency
    /// Uses constant product AMM formula: price = base_liquidity / token_liquidity
    pub fn current_price(&self) -> U256 {
        if self.token_liquidity == U256::zero() {
            return U256::zero();
        }
        self.base_liquidity / self.token_liquidity
    }
}

impl From<&PoolInfo> for fair_launch_abi::PoolInfoGQL {
    fn from(pool: &PoolInfo) -> Self {
        Self {
            pool_id: pool.pool_id.clone(),
            token_id: pool.token_id.clone(),
            token_liquidity: pool.token_liquidity.to_string(),
            base_liquidity: pool.base_liquidity.to_string(),
            initial_ratio: pool.initial_ratio.to_string(),
            created_at: pool.created_at.micros().to_string(),
            is_locked: pool.is_locked,
            lock_expires_at: pool.lock_expires_at.map(|t| t.micros().to_string()),
            trade_count: pool.trade_count,
            tvl: pool.tvl.to_string(),
        }
    }
}

/// Swap contract state - manages all graduated token pools
#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct SwapState {
    /// All pools: pool_id → PoolInfo
    pub pools: MapView<String, PoolInfo>,

    /// Token to pool mapping: token_id → pool_id
    pub token_to_pool: MapView<String, String>,

    /// Total number of pools created
    pub total_pools: RegisterView<u64>,

    /// Total value locked across all pools (in base currency)
    pub total_tvl: RegisterView<U256>,

    /// Contract creation timestamp
    pub created_at: RegisterView<Timestamp>,
}

impl SwapState {
    /// Initialize the swap contract
    pub async fn initialize(&mut self, created_at: Timestamp) -> Result<(), anyhow::Error> {
        self.total_pools.set(0);
        self.total_tvl.set(U256::zero());
        self.created_at.set(created_at);
        Ok(())
    }

    /// Create a new pool for a graduated token
    pub async fn create_pool(
        &mut self,
        token_id: String,
        total_supply: U256,
        total_raised: U256,
        created_at: Timestamp,
    ) -> Result<PoolInfo, anyhow::Error> {
        // Check if pool already exists for this token
        if self.token_to_pool.get(&token_id).await?.is_some() {
            anyhow::bail!("Pool already exists for token: {}", token_id);
        }

        // Create new pool
        let pool = PoolInfo::new(
            token_id.clone(),
            total_supply,
            total_raised,
            created_at,
        )?;

        // Store pool
        self.pools.insert(&pool.pool_id, pool.clone())?;
        self.token_to_pool.insert(&token_id, pool.pool_id.clone())?;

        // Update totals
        let current_pools = self.total_pools.get();
        self.total_pools.set(*current_pools + 1);

        let current_tvl = *self.total_tvl.get();
        self.total_tvl.set(current_tvl + pool.tvl);

        Ok(pool)
    }

    /// Get pool by pool ID
    pub async fn get_pool(&self, pool_id: &str) -> Result<Option<PoolInfo>, anyhow::Error> {
        Ok(self.pools.get(pool_id).await?)
    }

    /// Get pool by token ID
    pub async fn get_pool_by_token(&self, token_id: &str) -> Result<Option<PoolInfo>, anyhow::Error> {
        if let Some(pool_id) = self.token_to_pool.get(token_id).await? {
            Ok(self.pools.get(&pool_id).await?)
        } else {
            Ok(None)
        }
    }

    /// Get all pools (paginated)
    pub async fn get_all_pools(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<PoolInfo>, anyhow::Error> {
        let mut pools = Vec::new();
        let mut count = 0;
        let mut skipped = 0;

        for pool_id in self.pools.indices().await? {
            if skipped < offset {
                skipped += 1;
                continue;
            }

            if count >= limit {
                break;
            }

            if let Some(pool) = self.pools.get(&pool_id).await? {
                pools.push(pool);
                count += 1;
            }
        }

        Ok(pools)
    }

    /// Check if token has graduated (has a pool)
    pub async fn has_pool(&self, token_id: &str) -> Result<bool, anyhow::Error> {
        Ok(self.token_to_pool.get(token_id).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_views::memory::MemoryContext;

    #[tokio::test]
    async fn test_pool_creation() {
        let token_id = "test-token-123".to_string();
        let total_supply = U256::from(1_000_000_000u64);
        let total_raised = U256::from(69_000);
        let created_at = Timestamp::from(1234567890);

        let pool = PoolInfo::new(
            token_id.clone(),
            total_supply,
            total_raised,
            created_at,
        ).unwrap();

        assert_eq!(pool.token_id, token_id);
        assert_eq!(pool.pool_id, format!("pool-{}", token_id));
        assert_eq!(pool.token_liquidity, total_supply);
        assert_eq!(pool.base_liquidity, total_raised);
        assert!(pool.is_locked);
        assert_eq!(pool.lock_expires_at, None);
        assert_eq!(pool.trade_count, 0);
        assert_eq!(pool.tvl, total_raised * U256::from(2));
    }

    #[tokio::test]
    async fn test_pool_creation_validation() {
        let token_id = "test-token".to_string();
        let created_at = Timestamp::from(0);

        // Test zero supply
        let result = PoolInfo::new(
            token_id.clone(),
            U256::zero(),
            U256::from(1000),
            created_at,
        );
        assert!(result.is_err());

        // Test zero raised
        let result = PoolInfo::new(
            token_id.clone(),
            U256::from(1000),
            U256::zero(),
            created_at,
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_swap_state_initialization() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        assert_eq!(*state.total_pools.get(), 0);
        assert_eq!(*state.total_tvl.get(), U256::zero());
        assert_eq!(*state.created_at.get(), created_at);
    }

    #[tokio::test]
    async fn test_create_and_get_pool() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "test-token-456".to_string();
        let total_supply = U256::from(1_000_000);
        let total_raised = U256::from(10_000);

        // Create pool
        let pool = state.create_pool(
            token_id.clone(),
            total_supply,
            total_raised,
            created_at,
        ).await.unwrap();

        assert_eq!(pool.token_id, token_id);
        assert_eq!(*state.total_pools.get(), 1);

        // Get pool by ID
        let retrieved = state.get_pool(&pool.pool_id).await.unwrap().unwrap();
        assert_eq!(retrieved.token_id, token_id);

        // Get pool by token
        let retrieved = state.get_pool_by_token(&token_id).await.unwrap().unwrap();
        assert_eq!(retrieved.pool_id, pool.pool_id);

        // Check has_pool
        assert!(state.has_pool(&token_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_duplicate_pool_prevention() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "test-token-duplicate".to_string();
        let total_supply = U256::from(1_000_000);
        let total_raised = U256::from(10_000);

        // Create first pool
        state.create_pool(
            token_id.clone(),
            total_supply,
            total_raised,
            created_at,
        ).await.unwrap();

        // Try to create duplicate
        let result = state.create_pool(
            token_id.clone(),
            total_supply,
            total_raised,
            created_at,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_pools_pagination() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Create 5 pools
        for i in 0..5 {
            let token_id = format!("token-{}", i);
            state.create_pool(
                token_id,
                U256::from(1_000_000),
                U256::from(10_000),
                created_at,
            ).await.unwrap();
        }

        // Test pagination
        let pools = state.get_all_pools(0, 3).await.unwrap();
        assert_eq!(pools.len(), 3);

        let pools = state.get_all_pools(3, 5).await.unwrap();
        assert_eq!(pools.len(), 2);

        let pools = state.get_all_pools(0, 100).await.unwrap();
        assert_eq!(pools.len(), 5);
    }

    #[test]
    fn test_pool_price_calculation() {
        let token_id = "test-token".to_string();
        let total_supply = U256::from(1_000_000_000u64); // 1B tokens
        let total_raised = U256::from(69_000); // 69k base currency
        let created_at = Timestamp::from(0);

        let pool = PoolInfo::new(
            token_id,
            total_supply,
            total_raised,
            created_at,
        ).unwrap();

        let price = pool.current_price();
        assert!(price > U256::zero());

        // Price should be approximately 69000 / 1000000000 = 0.000069 (in scaled form)
        let expected_ratio = (total_raised * U256::from(1_000_000)) / total_supply;
        assert_eq!(price, expected_ratio);
    }
}
