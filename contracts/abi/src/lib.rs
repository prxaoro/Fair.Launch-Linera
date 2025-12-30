#[cfg(feature = "service")]
use async_graphql::SimpleObject;
use linera_sdk::linera_base_types::{Account, Timestamp};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Shared types for Fair Launch platform

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_url: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurveConfig {
    /// Constant k in price formula: price = k * (supply / scale)^2
    pub k: U256,
    /// Scale factor (e.g., 1_000_000 for 1M tokens)
    pub scale: U256,
    /// Target raise in base currency (e.g., 69000 tokens)
    pub target_raise: U256,
    /// Max supply before curve completes
    pub max_supply: U256,
    /// Creator fee percentage (0-10000, where 300 = 3%)
    pub creator_fee_bps: u16,
}

/// GraphQL-friendly version of BondingCurveConfig
#[derive(Debug, Clone)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct BondingCurveConfigGQL {
    pub k: String,
    pub scale: String,
    pub target_raise: String,
    pub max_supply: String,
    pub creator_fee_bps: u16,
}

impl From<&BondingCurveConfig> for BondingCurveConfigGQL {
    fn from(config: &BondingCurveConfig) -> Self {
        Self {
            k: config.k.to_string(),
            scale: config.scale.to_string(),
            target_raise: config.target_raise.to_string(),
            max_supply: config.max_supply.to_string(),
            creator_fee_bps: config.creator_fee_bps,
        }
    }
}

impl Default for BondingCurveConfig {
    fn default() -> Self {
        Self {
            k: U256::from(1000),
            scale: U256::from(1_000_000),
            target_raise: U256::from(69_000),
            max_supply: U256::from(1_000_000_000u64),
            creator_fee_bps: 300, // 3% default fee
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLaunch {
    pub token_id: String,
    pub creator: Account,  // Changed from ChainId to Account
    pub metadata: TokenMetadata,
    pub curve_config: BondingCurveConfig,
    pub current_supply: U256,
    pub total_raised: U256,
    pub is_graduated: bool,
    pub created_at: Timestamp,
    pub dex_pool_id: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct TokenLaunchGQL {
    pub token_id: String,
    pub metadata: TokenMetadata,
    pub curve_config: BondingCurveConfigGQL,
    pub current_supply: String,
    pub total_raised: String,
    pub is_graduated: bool,
    pub dex_pool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub token_id: String,
    pub trader: Account,  // Changed from ChainId to Account
    pub is_buy: bool,
    pub token_amount: U256,
    pub currency_amount: U256,
    pub price: U256,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct TradeGQL {
    pub token_id: String,
    pub is_buy: bool,
    pub token_amount: String,
    pub currency_amount: String,
    pub price: String,
}

impl From<&Trade> for TradeGQL {
    fn from(trade: &Trade) -> Self {
        Self {
            token_id: trade.token_id.clone(),
            is_buy: trade.is_buy,
            token_amount: trade.token_amount.to_string(),
            currency_amount: trade.currency_amount.to_string(),
            price: trade.price.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub token_id: String,
    pub balance: U256,
    pub total_invested: U256,
    pub trades_count: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct UserPositionGQL {
    pub token_id: String,
    pub balance: String,
    pub total_invested: String,
    pub trades_count: u64,
}

impl From<&UserPosition> for UserPositionGQL {
    fn from(pos: &UserPosition) -> Self {
        Self {
            token_id: pos.token_id.clone(),
            balance: pos.balance.to_string(),
            total_invested: pos.total_invested.to_string(),
            trades_count: pos.trades_count,
        }
    }
}

/// Operations for Factory contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactoryOperation {
    CreateToken {
        metadata: TokenMetadata,
        curve_config: Option<BondingCurveConfig>,
    },
}

/// Operations for Token contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenOperation {
    Buy {
        amount: U256,
        max_cost: U256, // Slippage protection
    },
    Sell {
        amount: U256,
        min_return: U256, // Slippage protection
    },
    /// Called by factory when token is created
    Initialize {
        creator: Account,  // Changed from ChainId to Account
        metadata: TokenMetadata,
        curve_config: BondingCurveConfig,
    },
    /// Graduate to DEX when curve completes
    Graduate,
    /// Approve spender to transfer tokens on behalf of owner
    Approve {
        spender: Account,
        amount: U256,
    },
    /// Transfer tokens from owner to recipient (requires allowance)
    TransferFrom {
        from: Account,
        to: Account,
        amount: U256,
    },
}

/// Cross-chain messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Factory → Token: Token created
    TokenCreated {
        token_id: String,
        creator: Account,  // Changed from ChainId to Account
        metadata: TokenMetadata,
        curve_config: BondingCurveConfig,
    },

    /// Token → User: Trade executed
    TradeExecuted {
        token_id: String,
        trader: Account,  // Changed from ChainId to Account
        is_buy: bool,
        token_amount: U256,
        currency_amount: U256,
        new_price: U256,
    },

    /// Token → Swap: Graduate to DEX
    GraduateToken {
        token_id: String,
        total_supply: U256,
        total_raised: U256,
    },

    /// Swap → Token: Pool created
    PoolCreated {
        token_id: String,
        pool_id: String,
    },

    /// Factory → All: New token launched (broadcast)
    NewLaunch {
        token_id: String,
        metadata: TokenMetadata,
        creator: Account,  // Changed from ChainId to Account
    },
}

/// GraphQL-friendly version of PoolInfo from swap contract
#[derive(Debug, Clone)]
#[cfg_attr(feature = "service", derive(SimpleObject))]
pub struct PoolInfoGQL {
    pub pool_id: String,
    pub token_id: String,
    pub token_liquidity: String,
    pub base_liquidity: String,
    pub initial_ratio: String,
    pub created_at: String,
    pub is_locked: bool,
    pub lock_expires_at: Option<String>,
    pub trade_count: u64,
    pub tvl: String,
}

/// Bonding curve calculations
pub mod bonding_curve {
    use super::*;

    /// Calculate cost to buy `amount` tokens at current supply
    /// Formula: Integral of k * (supply / scale)^2 from current_supply to new_supply
    pub fn calculate_buy_cost(
        current_supply: U256,
        amount: U256,
        k: U256,
        scale: U256,
    ) -> U256 {
        let new_supply = current_supply + amount;

        // Integral: k * (x^3 / (3 * scale^2))
        let scale_squared = scale * scale;
        let integral_new = (k * new_supply * new_supply * new_supply) / (U256::from(3) * scale_squared);
        let integral_old = (k * current_supply * current_supply * current_supply) / (U256::from(3) * scale_squared);

        integral_new - integral_old
    }

    /// Calculate return for selling `amount` tokens at current supply
    pub fn calculate_sell_return(
        current_supply: U256,
        amount: U256,
        k: U256,
        scale: U256,
    ) -> U256 {
        if amount > current_supply {
            return U256::zero();
        }

        let new_supply = current_supply - amount;

        let scale_squared = scale * scale;
        let integral_old = (k * current_supply * current_supply * current_supply) / (U256::from(3) * scale_squared);
        let integral_new = (k * new_supply * new_supply * new_supply) / (U256::from(3) * scale_squared);

        integral_old - integral_new
    }

    /// Calculate current price at given supply
    /// Formula: k * (supply / scale)^2
    /// Optimized to minimize precision loss: (k * supply / scale) * supply / scale
    pub fn calculate_current_price(supply: U256, k: U256, scale: U256) -> U256 {
        if supply == U256::zero() || scale == U256::zero() {
            return U256::zero();
        }
        // Avoid overflow and precision loss by dividing incrementally
        (k * supply / scale) * supply / scale
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_buy_cost_calculation() {
            let k = U256::from(1000);
            let scale = U256::from(1_000_000);
            let current_supply = U256::from(0);
            let amount = U256::from(100_000);

            let cost = calculate_buy_cost(current_supply, amount, k, scale);
            assert!(cost > U256::zero());
        }

        #[test]
        fn test_sell_return_calculation() {
            let k = U256::from(1000);
            let scale = U256::from(1_000_000);
            let current_supply = U256::from(100_000);
            let amount = U256::from(50_000);

            let return_amount = calculate_sell_return(current_supply, amount, k, scale);
            assert!(return_amount > U256::zero());
        }

        #[test]
        fn test_price_calculation() {
            let k = U256::from(1000);
            let scale = U256::from(1_000_000);
            let supply = U256::from(500_000);

            let price = calculate_current_price(supply, k, scale);
            assert!(price > U256::zero());
        }
    }
}

/// Operations for Swap contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapOperation {
    /// Add liquidity to a pool
    AddLiquidity {
        pool_id: String,
        token_amount: U256,
        base_amount: U256,
    },
    /// Swap tokens
    Swap {
        pool_id: String,
        token_in: String,
        amount_in: U256,
        min_amount_out: U256,
    },
}

/// ABI definitions for the three contracts

use linera_sdk::abi::{ContractAbi, ServiceAbi};

// Token Contract ABI
pub struct TokenAbi;

impl ContractAbi for TokenAbi {
    type Operation = TokenOperation;
    type Response = ();
}

#[cfg(feature = "service")]
impl ServiceAbi for TokenAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

// Factory Contract ABI
pub struct FactoryAbi;

impl ContractAbi for FactoryAbi {
    type Operation = FactoryOperation;
    type Response = String; // Returns token_id
}

#[cfg(feature = "service")]
impl ServiceAbi for FactoryAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

// Swap Contract ABI
pub struct SwapAbi;

impl ContractAbi for SwapAbi {
    type Operation = SwapOperation;
    type Response = ();
}

#[cfg(feature = "service")]
impl ServiceAbi for SwapAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

#[cfg(test)]
mod bonding_curve_tests;
