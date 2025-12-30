use fair_launch_abi::{BondingCurveConfig, TokenMetadata, Trade, UserPosition};
use linera_sdk::{
    linera_base_types::{Account, Timestamp},
    views::{MapView, RegisterView, RootView, ViewStorageContext},
};
use primitive_types::U256;

/// Token state - stores all token data on its microchain
#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct TokenState {
    /// Token unique ID
    pub token_id: RegisterView<String>,

    /// Creator of the token (Account includes chain_id and owner)
    pub creator: RegisterView<Option<Account>>,

    /// Token metadata (name, symbol, etc.)
    pub metadata: RegisterView<TokenMetadata>,

    /// Bonding curve configuration
    pub curve_config: RegisterView<BondingCurveConfig>,

    /// Current circulating supply
    pub current_supply: RegisterView<U256>,

    /// Total currency raised from sales
    pub total_raised: RegisterView<U256>,

    /// Whether token has graduated to DEX
    pub is_graduated: RegisterView<bool>,

    /// Creation timestamp
    pub created_at: RegisterView<Timestamp>,

    /// DEX pool ID after graduation
    pub dex_pool_id: RegisterView<Option<String>>,

    /// User balances: Account → token balance
    pub balances: MapView<Account, U256>,

    /// Trade history: trade_id → Trade
    pub trades: MapView<String, Trade>,

    /// User positions: Account → UserPosition
    pub user_positions: MapView<Account, UserPosition>,

    /// Total number of holders
    pub holder_count: RegisterView<u64>,

    /// Total number of trades
    pub trade_count: RegisterView<u64>,

    /// Allowances: "{owner}:{spender}" → amount approved
    /// Allows spenders to transfer tokens on behalf of owners (for DEX integration)
    pub allowances: MapView<String, U256>,
}

impl TokenState {
    /// Initialize new token
    pub async fn initialize(
        &mut self,
        token_id: String,
        creator: Account,  // Changed from ChainId to Account
        metadata: TokenMetadata,
        curve_config: BondingCurveConfig,
        created_at: Timestamp,
    ) -> Result<(), anyhow::Error> {
        self.token_id.set(token_id);
        self.creator.set(Some(creator));
        self.metadata.set(metadata);
        self.curve_config.set(curve_config);
        self.current_supply.set(U256::zero());
        self.total_raised.set(U256::zero());
        self.is_graduated.set(false);
        self.created_at.set(created_at);
        self.dex_pool_id.set(None);
        self.holder_count.set(0);
        self.trade_count.set(0);
        Ok(())
    }

    /// Get user balance
    pub async fn get_balance(&self, account: &Account) -> U256 {  // Changed from ChainId to Account
        self.balances.get(account).await.unwrap_or_default().unwrap_or(U256::zero())
    }

    /// Set user balance
    pub async fn set_balance(&mut self, account: Account, balance: U256) -> Result<(), anyhow::Error> {  // Changed from ChainId to Account
        if balance == U256::zero() {
            self.balances.remove(&account)?;
            // Decrement holder count if balance goes to zero
            let current_count = self.holder_count.get();
            if *current_count > 0 {
                self.holder_count.set(*current_count - 1);
            }
        } else {
            // Check if this is a new holder
            let had_balance = self.balances.get(&account).await?.is_some();
            self.balances.insert(&account, balance)?;

            if !had_balance {
                // Increment holder count for new holder
                let current_count = self.holder_count.get();
                self.holder_count.set(*current_count + 1);
            }
        }
        Ok(())
    }

    /// Record a trade
    pub async fn record_trade(
        &mut self,
        trade_id: String,
        trade: Trade,
    ) -> Result<(), anyhow::Error> {
        self.trades.insert(&trade_id, trade.clone())?;

        // Update user position
        let mut position = self.user_positions
            .get(&trade.trader)
            .await?
            .unwrap_or(UserPosition {
                token_id: self.token_id.get().clone(),
                balance: U256::zero(),
                total_invested: U256::zero(),
                trades_count: 0,
            });

        if trade.is_buy {
            position.balance += trade.token_amount;
            position.total_invested += trade.currency_amount;
        } else {
            position.balance = position.balance.saturating_sub(trade.token_amount);
        }
        position.trades_count += 1;

        self.user_positions.insert(&trade.trader, position)?;

        // Increment trade count
        let count = self.trade_count.get();
        self.trade_count.set(*count + 1);

        Ok(())
    }

    /// Check if bonding curve has completed
    pub fn is_curve_complete(&self) -> bool {
        let current_supply = *self.current_supply.get();
        let max_supply = self.curve_config.get().max_supply;
        current_supply >= max_supply
    }

    /// Get all trades (paginated)
    pub async fn get_trades(&self, offset: usize, limit: usize) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut count = 0;
        let mut skipped = 0;

        for entry in self.trades.indices().await.unwrap() {
            if skipped < offset {
                skipped += 1;
                continue;
            }

            if count >= limit {
                break;
            }

            if let Ok(Some(trade)) = self.trades.get(&entry).await {
                trades.push(trade);
                count += 1;
            }
        }

        trades
    }

    /// Create allowance key from owner and spender accounts
    fn allowance_key(owner: &Account, spender: &Account) -> String {
        format!("{}:{}",
            serde_json::to_string(owner).unwrap_or_default(),
            serde_json::to_string(spender).unwrap_or_default()
        )
    }

    /// Get allowance amount that spender can spend on behalf of owner
    pub async fn get_allowance(&self, owner: &Account, spender: &Account) -> U256 {
        let key = Self::allowance_key(owner, spender);
        self.allowances.get(&key).await.unwrap_or(None).unwrap_or(U256::zero())
    }

    /// Set allowance amount (approve)
    pub async fn set_allowance(&mut self, owner: Account, spender: Account, amount: U256) -> Result<(), anyhow::Error> {
        let key = Self::allowance_key(&owner, &spender);
        self.allowances.insert(&key, amount)?;
        Ok(())
    }

    /// Decrease allowance amount (used in transferFrom)
    pub async fn decrease_allowance(&mut self, owner: &Account, spender: &Account, amount: U256) -> Result<(), anyhow::Error> {
        let current = self.get_allowance(owner, spender).await;
        let new_allowance = current.saturating_sub(amount);
        let key = Self::allowance_key(owner, spender);
        self.allowances.insert(&key, new_allowance)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::ChainId;
    use linera_views::memory::MemoryContext;

    #[tokio::test]
    async fn test_token_state_initialization() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        let token_id = "test-token".to_string();
        let creator = AccountOwner::from(ChainId::root(0));
        let metadata = TokenMetadata {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "A test token".to_string(),
            image_url: None,
            twitter: None,
            telegram: None,
            website: None,
        };
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        state.initialize(token_id.clone(), creator, metadata, curve_config, created_at)
            .await
            .unwrap();

        assert_eq!(state.token_id.get().as_str(), "test-token");
        assert_eq!(*state.current_supply.get(), U256::zero());
    }

    #[tokio::test]
    async fn test_balance_operations() {
        let context = MemoryContext::default();
        let mut state = TokenState::load(context).await.unwrap();

        let account = AccountOwner::from(ChainId::root(0));
        let balance = U256::from(1000);

        // Set balance
        state.set_balance(account, balance).await.unwrap();
        assert_eq!(state.get_balance(&account).await, balance);

        // Update balance
        let new_balance = U256::from(2000);
        state.set_balance(account, new_balance).await.unwrap();
        assert_eq!(state.get_balance(&account).await, new_balance);
    }
}
