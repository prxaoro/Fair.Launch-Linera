use fair_launch_abi::{BondingCurveConfig, TokenLaunch, TokenMetadata};
use linera_sdk::{
    linera_base_types::{Account, ChainId, Timestamp},
    views::{MapView, RegisterView, RootView, ViewStorageContext},
};
use linera_views::ViewError;
use primitive_types::U256;
use thiserror::Error;

/// Factory state errors
#[derive(Debug, Error)]
pub enum FactoryError {
    #[error("Token already exists with ID: {0}")]
    TokenAlreadyExists(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] anyhow::Error),

    #[error("View error: {0}")]
    ViewError(#[from] ViewError),
}

/// Factory state - tracks all created tokens
#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct FactoryState {
    /// All created tokens: token_id (ChainId string) → TokenLaunch
    pub tokens: MapView<String, TokenLaunch>,

    /// Total number of tokens created
    pub token_count: RegisterView<u64>,

    /// Creator registry: Account → Vec<token_id>
    /// Stores comma-separated token IDs for each creator
    pub creator_registry: MapView<Account, String>,

    /// Index for fast lookup: index → token_id
    pub token_index: MapView<u64, String>,
}

impl FactoryState {
    /// Register a new token launch
    pub async fn register_token(
        &mut self,
        token_id: String,
        creator: Account,  // Changed from ChainId to Account
        metadata: TokenMetadata,
        curve_config: BondingCurveConfig,
        created_at: Timestamp,
    ) -> Result<(), FactoryError> {
        // Validate metadata
        Self::validate_metadata(&metadata)?;

        // Check for duplicates
        if self.tokens.get(&token_id).await?.is_some() {
            return Err(FactoryError::TokenAlreadyExists(token_id));
        }

        // Create token launch record
        let token_launch = TokenLaunch {
            token_id: token_id.clone(),
            creator,
            metadata,
            curve_config,
            current_supply: U256::zero(),
            total_raised: U256::zero(),
            is_graduated: false,
            created_at,
            dex_pool_id: None,
        };

        // Store token
        self.tokens.insert(&token_id, token_launch)?;

        // Update token count and index
        let count = *self.token_count.get();
        self.token_index.insert(&count, token_id.clone())?;
        self.token_count.set(count + 1);

        // Update creator registry
        let mut creator_tokens = self
            .creator_registry
            .get(&creator)
            .await?
            .unwrap_or_default();

        if !creator_tokens.is_empty() {
            creator_tokens.push(',');
        }
        creator_tokens.push_str(&token_id);
        self.creator_registry.insert(&creator, creator_tokens)?;

        Ok(())
    }

    /// Get a token by ID
    pub async fn get_token(&self, token_id: &str) -> Result<TokenLaunch, FactoryError> {
        self.tokens
            .get(token_id)
            .await?
            .ok_or_else(|| FactoryError::TokenNotFound(token_id.to_string()))
    }

    /// Get all tokens created by a specific creator
    pub async fn get_tokens_by_creator(
        &self,
        creator: &Account,
    ) -> Result<Vec<TokenLaunch>, FactoryError> {
        let token_ids_str = self.creator_registry.get(creator).await?.unwrap_or_default();

        if token_ids_str.is_empty() {
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        for token_id in token_ids_str.split(',') {
            if let Ok(Some(token)) = self.tokens.get(token_id).await {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }

    /// Get all tokens (paginated)
    pub async fn get_all_tokens(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<TokenLaunch>, FactoryError> {
        let total_count = *self.token_count.get();
        let end = (offset + limit).min(total_count);

        let mut tokens = Vec::new();

        for i in offset..end {
            if let Ok(Some(token_id)) = self.token_index.get(&i).await {
                if let Ok(Some(token)) = self.tokens.get(&token_id).await {
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    /// Get total token count
    pub fn get_token_count(&self) -> u64 {
        *self.token_count.get()
    }

    /// Update token status (for graduation notifications)
    pub async fn update_token_status(
        &mut self,
        token_id: &str,
        is_graduated: bool,
        dex_pool_id: Option<String>,
    ) -> Result<(), FactoryError> {
        let mut token = self.get_token(token_id).await?;

        token.is_graduated = is_graduated;
        token.dex_pool_id = dex_pool_id;

        self.tokens.insert(token_id, token)?;

        Ok(())
    }

    /// Update token supply and raised amount (for trade notifications)
    pub async fn update_token_metrics(
        &mut self,
        token_id: &str,
        current_supply: U256,
        total_raised: U256,
    ) -> Result<(), FactoryError> {
        let mut token = self.get_token(token_id).await?;

        token.current_supply = current_supply;
        token.total_raised = total_raised;

        self.tokens.insert(token_id, token)?;

        Ok(())
    }

    /// Validate token metadata
    fn validate_metadata(metadata: &TokenMetadata) -> Result<(), FactoryError> {
        if metadata.name.trim().is_empty() {
            return Err(FactoryError::InvalidMetadata(
                "Token name cannot be empty".to_string(),
            ));
        }

        if metadata.symbol.trim().is_empty() {
            return Err(FactoryError::InvalidMetadata(
                "Token symbol cannot be empty".to_string(),
            ));
        }

        if metadata.name.len() > 100 {
            return Err(FactoryError::InvalidMetadata(
                "Token name too long (max 100 characters)".to_string(),
            ));
        }

        if metadata.symbol.len() > 20 {
            return Err(FactoryError::InvalidMetadata(
                "Token symbol too long (max 20 characters)".to_string(),
            ));
        }

        if metadata.description.len() > 1000 {
            return Err(FactoryError::InvalidMetadata(
                "Token description too long (max 1000 characters)".to_string(),
            ));
        }

        // Validate URL formats if provided
        if let Some(ref url) = metadata.image_url {
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("ipfs://") {
                return Err(FactoryError::InvalidMetadata(
                    "Invalid image URL format".to_string(),
                ));
            }
        }

        if let Some(ref url) = metadata.website {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(FactoryError::InvalidMetadata(
                    "Invalid website URL format".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{Account, AccountOwner, ChainId, Timestamp};
    use linera_views::memory::MemoryContext;

    fn create_test_metadata() -> TokenMetadata {
        TokenMetadata {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "A test token for unit tests".to_string(),
            image_url: Some("https://example.com/image.png".to_string()),
            twitter: Some("@testtoken".to_string()),
            telegram: Some("@testtoken".to_string()),
            website: Some("https://testtoken.com".to_string()),
        }
    }

    #[tokio::test]
    async fn test_register_token() {
        let context = MemoryContext::default();
        let mut state = FactoryState::load(context).await.unwrap();

        let token_id = "test-token-123".to_string();
        let creator = Account {
            chain_id: ChainId::root(0),
            owner: AccountOwner::CHAIN,
        };
        let metadata = create_test_metadata();
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        let result = state
            .register_token(
                token_id.clone(),
                creator,
                metadata.clone(),
                curve_config,
                created_at,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(state.get_token_count(), 1);

        let token = state.get_token(&token_id).await.unwrap();
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.metadata.name, "Test Token");
    }

    #[tokio::test]
    async fn test_duplicate_token_prevention() {
        let context = MemoryContext::default();
        let mut state = FactoryState::load(context).await.unwrap();

        let token_id = "test-token-123".to_string();
        let creator = Account {
            chain_id: ChainId::root(0),
            owner: AccountOwner::CHAIN,
        };
        let metadata = create_test_metadata();
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        // First registration should succeed
        state
            .register_token(
                token_id.clone(),
                creator,
                metadata.clone(),
                curve_config.clone(),
                created_at,
            )
            .await
            .unwrap();

        // Second registration should fail
        let result = state
            .register_token(token_id.clone(), creator, metadata, curve_config, created_at)
            .await;

        assert!(matches!(result, Err(FactoryError::TokenAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_metadata_validation() {
        let context = MemoryContext::default();
        let mut state = FactoryState::load(context).await.unwrap();

        let creator = Account {
            chain_id: ChainId::root(0),
            owner: AccountOwner::CHAIN,
        };
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        // Test empty name
        let mut metadata = create_test_metadata();
        metadata.name = "".to_string();

        let result = state
            .register_token(
                "test-1".to_string(),
                creator,
                metadata.clone(),
                curve_config.clone(),
                created_at,
            )
            .await;

        assert!(matches!(result, Err(FactoryError::InvalidMetadata(_))));

        // Test empty symbol
        metadata = create_test_metadata();
        metadata.symbol = "".to_string();

        let result = state
            .register_token("test-2".to_string(), creator, metadata, curve_config, created_at)
            .await;

        assert!(matches!(result, Err(FactoryError::InvalidMetadata(_))));
    }

    #[tokio::test]
    async fn test_get_tokens_by_creator() {
        let context = MemoryContext::default();
        let mut state = FactoryState::load(context).await.unwrap();

        let creator = Account {
            chain_id: ChainId::root(0),
            owner: AccountOwner::CHAIN,
        };
        let metadata = create_test_metadata();
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        // Create multiple tokens
        for i in 0..3 {
            state
                .register_token(
                    format!("token-{}", i),
                    creator,
                    metadata.clone(),
                    curve_config.clone(),
                    created_at,
                )
                .await
                .unwrap();
        }

        let tokens = state.get_tokens_by_creator(&creator).await.unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[tokio::test]
    async fn test_pagination() {
        let context = MemoryContext::default();
        let mut state = FactoryState::load(context).await.unwrap();

        let creator = Account {
            chain_id: ChainId::root(0),
            owner: AccountOwner::CHAIN,
        };
        let metadata = create_test_metadata();
        let curve_config = BondingCurveConfig::default();
        let created_at = Timestamp::from(0);

        // Create 10 tokens
        for i in 0..10 {
            state
                .register_token(
                    format!("token-{}", i),
                    creator,
                    metadata.clone(),
                    curve_config.clone(),
                    created_at,
                )
                .await
                .unwrap();
        }

        // Get first page
        let page1 = state.get_all_tokens(0, 5).await.unwrap();
        assert_eq!(page1.len(), 5);

        // Get second page
        let page2 = state.get_all_tokens(5, 5).await.unwrap();
        assert_eq!(page2.len(), 5);
    }
}
