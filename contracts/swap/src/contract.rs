#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
use fair_launch_abi::{Message, SwapAbi, SwapOperation};
use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::ChainId,
    views::View,
    Contract, ContractRuntime,
};
use primitive_types::U256;
use thiserror::Error;

use crate::state::SwapState;

#[derive(Debug, Error)]
pub enum SwapError {
    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Pool is locked")]
    PoolLocked,

    #[error("Insufficient liquidity")]
    InsufficientLiquidity,

    #[error("Slippage exceeded: got {got}, min {min}")]
    SlippageExceeded { got: U256, min: U256 },

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,
}

/// Swap contract - creates and manages locked liquidity pools for graduated tokens
pub struct SwapContract {
    state: SwapState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(SwapContract);

impl WithContractAbi for SwapContract {
    type Abi = SwapAbi;
}

impl Contract for SwapContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = SwapState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load swap state");
        SwapContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        let created_at = self.runtime.system_time();
        self.state
            .initialize(created_at)
            .await
            .expect("Failed to initialize swap contract");
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            SwapOperation::AddLiquidity {
                pool_id,
                token_amount,
                base_amount,
            } => {
                self.add_liquidity(pool_id, token_amount, base_amount)
                    .await
                    .expect("Failed to add liquidity");
            }
            SwapOperation::Swap {
                pool_id,
                token_in,
                amount_in,
                min_amount_out,
            } => {
                self.execute_swap(pool_id, token_in, amount_in, min_amount_out)
                    .await
                    .expect("Swap failed");
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::GraduateToken {
                token_id,
                total_supply,
                total_raised,
            } => {
                self.handle_graduation(token_id, total_supply, total_raised)
                    .await;
            }

            _ => {
                // Ignore other message types
            }
        }
    }

    async fn store(self) {
        // State is automatically persisted by linera-views
    }
}

impl SwapContract {
    /// Handle token graduation - create locked liquidity pool
    async fn handle_graduation(
        &mut self,
        token_id: String,
        total_supply: U256,
        total_raised: U256,
    ) {
        // Log graduation event
        self.log_event(&format!(
            "Graduation request received for token {}",
            token_id
        ));

        // Validate inputs
        if total_supply == U256::zero() {
            self.log_error(&format!(
                "Invalid graduation: token {} has zero supply",
                token_id
            ));
            return;
        }

        if total_raised == U256::zero() {
            self.log_error(&format!(
                "Invalid graduation: token {} has zero raised amount",
                token_id
            ));
            return;
        }

        // Check if pool already exists (idempotency check)
        match self.state.has_pool(&token_id).await {
            Ok(true) => {
                self.log_event(&format!(
                    "Pool already exists for token {}, ignoring duplicate graduation",
                    token_id
                ));

                // Still send PoolCreated message back (idempotent)
                if let Ok(Some(pool)) = self.state.get_pool_by_token(&token_id).await {
                    let chain_id = self.runtime.chain_id();
                    self.send_pool_created_message(token_id, pool.pool_id, chain_id);
                }
                return;
            }
            Err(e) => {
                self.log_error(&format!(
                    "Failed to check pool existence for token {}: {}",
                    token_id, e
                ));
                return;
            }
            _ => {}
        }

        // Create pool
        let created_at = self.runtime.system_time();
        match self
            .state
            .create_pool(token_id.clone(), total_supply, total_raised, created_at)
            .await
        {
            Ok(pool) => {
                self.log_event(&format!(
                    "Pool created successfully: {} for token {} with {} tokens and {} base currency (locked permanently)",
                    pool.pool_id, token_id, total_supply, total_raised
                ));

                // Send PoolCreated message back to token contract
                let chain_id = self.runtime.chain_id();
                self.send_pool_created_message(token_id, pool.pool_id, chain_id);
            }
            Err(e) => {
                self.log_error(&format!(
                    "Failed to create pool for token {}: {}",
                    token_id, e
                ));
            }
        }
    }

    /// Add liquidity to an existing pool
    async fn add_liquidity(
        &mut self,
        pool_id: String,
        token_amount: U256,
        base_amount: U256,
    ) -> Result<(), SwapError> {
        // Validate amounts
        if token_amount == U256::zero() || base_amount == U256::zero() {
            return Err(SwapError::InvalidAmount);
        }

        // Get pool
        let pool = self
            .state
            .get_pool(&pool_id)
            .await
            .map_err(|_| SwapError::PoolNotFound(pool_id.clone()))?
            .ok_or_else(|| SwapError::PoolNotFound(pool_id.clone()))?;

        // Check if pool is locked (fair launch pools are permanently locked)
        if pool.is_locked {
            return Err(SwapError::PoolLocked);
        }

        // NOTE: For fair launch pools, initial liquidity is added during graduation
        // and pools are permanently locked to prevent rug pulls. Additional liquidity
        // is not supported to maintain the anti-rug guarantee.
        return Err(SwapError::PoolLocked);
    }

    /// Execute a swap using constant product AMM formula
    async fn execute_swap(
        &mut self,
        pool_id: String,
        _token_in: String,
        amount_in: U256,
        min_amount_out: U256,
    ) -> Result<(), SwapError> {
        // Validate amount
        if amount_in == U256::zero() {
            return Err(SwapError::InvalidAmount);
        }

        // Get pool
        let mut pool = self
            .state
            .get_pool(&pool_id)
            .await
            .map_err(|_| SwapError::PoolNotFound(pool_id.clone()))?
            .ok_or_else(|| SwapError::PoolNotFound(pool_id.clone()))?;

        // Calculate output using constant product formula: x * y = k
        // amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
        let amount_out = (amount_in * pool.base_liquidity) / (pool.token_liquidity + amount_in);

        // Check slippage protection
        if amount_out < min_amount_out {
            return Err(SwapError::SlippageExceeded {
                got: amount_out,
                min: min_amount_out,
            });
        }

        // Update pool reserves
        pool.token_liquidity = pool.token_liquidity + amount_in;
        pool.base_liquidity = pool.base_liquidity - amount_out;
        pool.trade_count += 1;

        // Update pool in state
        self.state
            .pools
            .insert(&pool_id, pool)
            .expect("Failed to update pool");

        // NOTE: Token transfers are handled through the token contract's
        // approve/transferFrom operations. Users must:
        // 1. Approve swap contract to spend their tokens
        // 2. Call this swap operation
        // 3. Token contract transfers tokens via transferFrom
        // This maintains security by keeping token logic in token contract.

        Ok(())
    }

    /// Send PoolCreated message back to token contract
    fn send_pool_created_message(&mut self, token_id: String, pool_id: String, target_chain: ChainId) {
        self.runtime
            .prepare_message(Message::PoolCreated {
                token_id: token_id.clone(),
                pool_id: pool_id.clone(),
            })
            .with_tracking()
            .send_to(target_chain);

        self.log_event(&format!(
            "Sent PoolCreated message for token {} to chain {}",
            token_id, target_chain
        ));
    }

    /// Log an event (would integrate with Linera logging in production)
    fn log_event(&self, message: &str) {
        #[cfg(debug_assertions)]
        eprintln!("[SWAP-EVENT] {}", message);

        // In production, this would use proper logging
        let _ = message; // Suppress unused warning in release builds
    }

    /// Log an error (would integrate with Linera logging in production)
    fn log_error(&self, message: &str) {
        #[cfg(debug_assertions)]
        eprintln!("[SWAP-ERROR] {}", message);

        // In production, this would use proper error reporting
        let _ = message; // Suppress unused warning in release builds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ApplicationId, BytecodeId, ChainId};
    use linera_views::memory::MemoryContext;

    // Helper to create a test runtime would go here
    // Note: Full integration tests require Linera test harness

    #[tokio::test]
    async fn test_state_initialization() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        assert_eq!(*state.total_pools.get(), 0);
        assert_eq!(*state.total_tvl.get(), U256::zero());
    }

    #[tokio::test]
    async fn test_graduation_pool_creation() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "test-token-graduation".to_string();
        let total_supply = U256::from(1_000_000_000u64);
        let total_raised = U256::from(69_000);

        // Simulate graduation
        let pool = state
            .create_pool(token_id.clone(), total_supply, total_raised, created_at)
            .await
            .unwrap();

        assert_eq!(pool.token_id, token_id);
        assert_eq!(pool.token_liquidity, total_supply);
        assert_eq!(pool.base_liquidity, total_raised);
        assert!(pool.is_locked);
        assert_eq!(pool.lock_expires_at, None);

        // Verify pool is retrievable
        let retrieved = state.get_pool_by_token(&token_id).await.unwrap().unwrap();
        assert_eq!(retrieved.pool_id, pool.pool_id);

        // Verify totals updated
        assert_eq!(*state.total_pools.get(), 1);
        assert_eq!(*state.total_tvl.get(), pool.tvl);
    }

    #[tokio::test]
    async fn test_idempotent_graduation() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "test-token-idempotent".to_string();
        let total_supply = U256::from(1_000_000);
        let total_raised = U256::from(10_000);

        // First graduation
        let pool1 = state
            .create_pool(token_id.clone(), total_supply, total_raised, created_at)
            .await
            .unwrap();

        // Second graduation should fail
        let result = state
            .create_pool(token_id.clone(), total_supply, total_raised, created_at)
            .await;

        assert!(result.is_err());

        // Verify only one pool exists
        assert_eq!(*state.total_pools.get(), 1);

        // Verify pool data unchanged
        let pool2 = state.get_pool_by_token(&token_id).await.unwrap().unwrap();
        assert_eq!(pool1.pool_id, pool2.pool_id);
        assert_eq!(pool1.token_liquidity, pool2.token_liquidity);
    }

    #[tokio::test]
    async fn test_multiple_graduations() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        // Graduate 3 tokens
        for i in 0..3 {
            let token_id = format!("token-{}", i);
            let total_supply = U256::from(1_000_000 * (i + 1));
            let total_raised = U256::from(10_000 * (i + 1));

            state
                .create_pool(token_id.clone(), total_supply, total_raised, created_at)
                .await
                .unwrap();
        }

        assert_eq!(*state.total_pools.get(), 3);

        // Verify all pools are retrievable
        for i in 0..3 {
            let token_id = format!("token-{}", i);
            let pool = state.get_pool_by_token(&token_id).await.unwrap().unwrap();
            assert_eq!(pool.token_id, token_id);
            assert!(pool.is_locked);
        }
    }

    #[tokio::test]
    async fn test_invalid_graduation_zero_supply() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "invalid-token".to_string();
        let result = state
            .create_pool(token_id, U256::zero(), U256::from(1000), created_at)
            .await;

        assert!(result.is_err());
        assert_eq!(*state.total_pools.get(), 0);
    }

    #[tokio::test]
    async fn test_invalid_graduation_zero_raised() {
        let context = MemoryContext::default();
        let mut state = SwapState::load(context).await.unwrap();

        let created_at = linera_sdk::linera_base_types::Timestamp::from(1234567890);
        state.initialize(created_at).await.unwrap();

        let token_id = "invalid-token".to_string();
        let result = state
            .create_pool(token_id, U256::from(1000), U256::zero(), created_at)
            .await;

        assert!(result.is_err());
        assert_eq!(*state.total_pools.get(), 0);
    }
}
