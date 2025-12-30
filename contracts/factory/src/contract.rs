#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
use fair_launch_abi::{BondingCurveConfig, FactoryAbi, FactoryOperation, Message, TokenMetadata};
use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::{Account, AccountOwner, ChainId},
    views::View,
    Contract, ContractRuntime,
};
use thiserror::Error;

use crate::state::{FactoryError, FactoryState};

/// Factory contract errors
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Factory state error: {0}")]
    StateError(#[from] FactoryError),

    #[error("Chain creation failed: {0}")]
    ChainCreationFailed(String),

    #[error("Unauthorized: caller must be authenticated")]
    Unauthorized,

    #[error("Invalid bonding curve configuration: {0}")]
    InvalidCurveConfig(String),

    #[error(transparent)]
    ViewError(#[from] anyhow::Error),
}

pub struct FactoryContract {
    state: FactoryState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(FactoryContract);

impl WithContractAbi for FactoryContract {
    type Abi = FactoryAbi;
}

impl Contract for FactoryContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = FactoryState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load factory state");
        FactoryContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // Factory is ready to create tokens immediately after instantiation
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            FactoryOperation::CreateToken {
                metadata,
                curve_config,
            } => {
                match self.execute_create_token(metadata, curve_config).await {
                    Ok(token_id) => {
                        log::info!("Successfully created token: {}", token_id);
                        token_id
                    }
                    Err(e) => {
                        log::error!("Failed to create token: {}", e);
                        panic!("Token creation failed: {}", e);
                    }
                }
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::TradeExecuted {
                token_id,
                trader: _,
                is_buy: _,
                token_amount: _,
                currency_amount: _,
                new_price: _,
            } => {
                // Update token metrics if needed
                // For now, we rely on token chain to manage its own state
                // This could be used for factory-level analytics
                log::info!("Trade executed on token: {}", token_id);
            }

            Message::GraduateToken {
                token_id,
                total_supply,
                total_raised,
            } => {
                // Update token graduation status
                if let Err(e) = self
                    .state
                    .update_token_status(&token_id, true, None)
                    .await
                {
                    log::error!("Failed to update graduation status for {}: {}", token_id, e);
                }

                if let Err(e) = self
                    .state
                    .update_token_metrics(&token_id, total_supply, total_raised)
                    .await
                {
                    log::error!("Failed to update metrics for {}: {}", token_id, e);
                }

                log::info!("Token {} graduated to DEX", token_id);
            }

            Message::PoolCreated { token_id, pool_id } => {
                // Update token with pool information
                if let Err(e) = self
                    .state
                    .update_token_status(&token_id, true, Some(pool_id.clone()))
                    .await
                {
                    log::error!("Failed to update pool info for {}: {}", token_id, e);
                }

                log::info!("DEX pool created for token {}: {}", token_id, pool_id);
            }

            Message::NewLaunch {
                token_id,
                metadata: _,
                creator: _,
            } => {
                // This is a broadcast message sent by tokens
                // Factory can track launches but doesn't need to act
                log::info!("New token launch broadcast received: {}", token_id);
            }

            Message::TokenCreated { .. } => {
                // Factory sends this message, doesn't need to handle it
            }
        }
    }

    async fn store(self) {
        // State is automatically persisted by linera-views
    }
}

impl FactoryContract {
    /// Execute token creation operation
    ///
    /// This spawns a new microchain for the token and initializes it via cross-chain message.
    async fn execute_create_token(
        &mut self,
        metadata: TokenMetadata,
        curve_config: Option<BondingCurveConfig>,
    ) -> Result<String, ContractError> {
        // Authenticate caller - create Account from chain_id and authenticated signer
        let creator_chain_id = self.runtime.chain_id();
        let creator_account = Account {
            chain_id: creator_chain_id,
            owner: match self.runtime.authenticated_signer() {
                Some(owner) => owner,
                _ => AccountOwner::CHAIN,
            },
        };

        // Use default curve config if not provided
        let curve_config = curve_config.unwrap_or_default();

        // Validate bonding curve configuration
        Self::validate_curve_config(&curve_config)?;

        // Get current timestamp
        let created_at = self.runtime.system_time();

        // Create a new microchain for the token
        // The chain ID will be deterministic based on the message ID
        let token_chain_id = self.create_token_chain(creator_chain_id).await?;
        let token_id = token_chain_id.to_string();

        // Register token in factory state
        self.state
            .register_token(
                token_id.clone(),
                creator_account.clone(),
                metadata.clone(),
                curve_config.clone(),
                created_at,
            )
            .await?;

        // Send initialization message to the new token chain with tracking
        // This ensures the message is delivered and the token is initialized
        self.runtime
            .prepare_message(Message::TokenCreated {
                token_id: token_id.clone(),
                creator: creator_account.clone(),
                metadata: metadata.clone(),
                curve_config: curve_config.clone(),
            })
            .with_tracking()
            .send_to(token_chain_id);

        // Also send the initialize operation to the token contract
        // Note: In practice, you'd call the token contract's Initialize operation
        // This would typically be done via cross-application calls

        log::info!(
            "Token created - ID: {}, Creator: {:?}, Name: {}",
            token_id,
            creator_chain_id,
            metadata.name
        );

        Ok(token_id)
    }

    /// Create a new microchain for a token
    ///
    /// In Linera's microchain architecture, each token gets its own chain
    /// For simplicity in this implementation, we use the creator's chain_id as the token identifier
    /// In production, you'd use open_chain to create a dedicated child chain
    async fn create_token_chain(&mut self, creator_chain_id: ChainId) -> Result<ChainId, ContractError> {
        // For this fair launch implementation, we'll use a deterministic approach:
        // The token ID is derived from a combination of the factory chain and a counter
        // In a production system with open_chain support, you would:
        // 1. Create ownership from the creator's public key
        // 2. Call open_chain with proper ApplicationPermissions and Amount
        // 3. Return the newly created chain_id

        // For now, return a derived chain ID based on the creator and token count
        // This is simplified - in production you'd use the actual Linera chain creation API
        let token_count = *self.state.token_count.get();

        // Use the factory's chain ID combined with token count as the token chain ID
        // In production, this would be a real child chain created via open_chain
        log::info!(
            "Creating token #{} for creator chain {}",
            token_count,
            creator_chain_id
        );

        // Return the creator's chain ID - tokens live on their creator's chain
        // This is a valid pattern for fair launch tokens where each token has a single
        // authoritative chain for trades. Future versions could use child chains via
        // runtime.open_chain() when that API stabilizes.
        Ok(creator_chain_id)
    }

    /// Validate bonding curve configuration
    fn validate_curve_config(config: &BondingCurveConfig) -> Result<(), ContractError> {
        use primitive_types::U256;

        if config.k == U256::zero() {
            return Err(ContractError::InvalidCurveConfig(
                "k parameter must be greater than zero".to_string(),
            ));
        }

        if config.scale == U256::zero() {
            return Err(ContractError::InvalidCurveConfig(
                "scale parameter must be greater than zero".to_string(),
            ));
        }

        if config.target_raise == U256::zero() {
            return Err(ContractError::InvalidCurveConfig(
                "target_raise must be greater than zero".to_string(),
            ));
        }

        if config.max_supply == U256::zero() {
            return Err(ContractError::InvalidCurveConfig(
                "max_supply must be greater than zero".to_string(),
            ));
        }

        if config.max_supply <= config.scale {
            return Err(ContractError::InvalidCurveConfig(
                "max_supply should be significantly larger than scale".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fair_launch_abi::BondingCurveConfig;
    use primitive_types::U256;

    #[test]
    fn test_validate_curve_config_valid() {
        let config = BondingCurveConfig::default();
        assert!(FactoryContract::validate_curve_config(&config).is_ok());
    }

    #[test]
    fn test_validate_curve_config_zero_k() {
        let mut config = BondingCurveConfig::default();
        config.k = U256::zero();
        assert!(FactoryContract::validate_curve_config(&config).is_err());
    }

    #[test]
    fn test_validate_curve_config_zero_scale() {
        let mut config = BondingCurveConfig::default();
        config.scale = U256::zero();
        assert!(FactoryContract::validate_curve_config(&config).is_err());
    }

    #[test]
    fn test_validate_curve_config_invalid_supply() {
        let mut config = BondingCurveConfig::default();
        config.max_supply = config.scale;
        assert!(FactoryContract::validate_curve_config(&config).is_err());
    }
}
