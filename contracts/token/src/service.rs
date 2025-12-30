#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use fair_launch_abi::{bonding_curve, TokenAbi};
use linera_sdk::{
    abi::WithServiceAbi,
    linera_base_types::Account,
    views::View,
    Service, ServiceRuntime,
};
use primitive_types::U256;
use std::sync::Arc;

use crate::state::TokenState;

pub struct TokenService {
    state: Arc<TokenState>,
}

linera_sdk::service!(TokenService);

impl WithServiceAbi for TokenService {
    type Abi = TokenAbi;
}

impl Service for TokenService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = TokenState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        TokenService {
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
    state: Arc<TokenState>,
}

#[derive(SimpleObject)]
pub struct TokenInfo {
    pub token_id: String,
    pub creator: String, // ChainId serialized as String for GraphQL
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub current_supply: String,
    pub total_raised: String,
    pub current_price: String,
    pub holder_count: u64,
    pub trade_count: u64,
    pub is_graduated: bool,
    pub progress_percentage: f64,
}

#[derive(SimpleObject)]
pub struct BuySellQuote {
    pub token_amount: String,
    pub currency_amount: String,
    pub price_impact: f64,
    pub new_price: String,
}

#[Object]
impl QueryRoot {
    /// Get token information
    async fn token_info(&self) -> TokenInfo {
        let token_id = self.state.token_id.get().clone();
        let creator = self.state.creator.get().clone().expect("Token creator not initialized");
        let metadata = self.state.metadata.get().clone();
        let current_supply = *self.state.current_supply.get();
        let total_raised = *self.state.total_raised.get();
        let curve_config = self.state.curve_config.get().clone();

        let current_price = bonding_curve::calculate_current_price(
            current_supply,
            curve_config.k,
            curve_config.scale,
        );

        let progress_percentage = if curve_config.max_supply > U256::zero() {
            let progress = (current_supply * U256::from(10000)) / curve_config.max_supply;
            progress.as_u64() as f64 / 100.0
        } else {
            0.0
        };

        TokenInfo {
            token_id,
            creator: creator.to_string(),
            name: metadata.name,
            symbol: metadata.symbol,
            description: metadata.description,
            current_supply: current_supply.to_string(),
            total_raised: total_raised.to_string(),
            current_price: current_price.to_string(),
            holder_count: *self.state.holder_count.get(),
            trade_count: *self.state.trade_count.get(),
            is_graduated: *self.state.is_graduated.get(),
            progress_percentage,
        }
    }

    /// Get buy quote
    async fn buy_quote(&self, amount: String) -> Option<BuySellQuote> {
        let amount_u256 = U256::from_dec_str(&amount).ok()?;
        let current_supply = *self.state.current_supply.get();
        let curve_config = self.state.curve_config.get().clone();

        let cost = bonding_curve::calculate_buy_cost(
            current_supply,
            amount_u256,
            curve_config.k,
            curve_config.scale,
        );

        let current_price = bonding_curve::calculate_current_price(
            current_supply,
            curve_config.k,
            curve_config.scale,
        );

        let new_supply = current_supply + amount_u256;
        let new_price = bonding_curve::calculate_current_price(
            new_supply,
            curve_config.k,
            curve_config.scale,
        );

        let price_impact = if current_price > U256::zero() {
            let impact = ((new_price.saturating_sub(current_price)) * U256::from(10000)) / current_price;
            impact.as_u64() as f64 / 100.0
        } else {
            0.0
        };

        Some(BuySellQuote {
            token_amount: amount,
            currency_amount: cost.to_string(),
            price_impact,
            new_price: new_price.to_string(),
        })
    }

    /// Get sell quote
    async fn sell_quote(&self, amount: String) -> Option<BuySellQuote> {
        let amount_u256 = U256::from_dec_str(&amount).ok()?;
        let current_supply = *self.state.current_supply.get();
        let curve_config = self.state.curve_config.get().clone();

        if amount_u256 > current_supply {
            return None;
        }

        let return_amount = bonding_curve::calculate_sell_return(
            current_supply,
            amount_u256,
            curve_config.k,
            curve_config.scale,
        );

        let current_price = bonding_curve::calculate_current_price(
            current_supply,
            curve_config.k,
            curve_config.scale,
        );

        let new_supply = current_supply - amount_u256;
        let new_price = bonding_curve::calculate_current_price(
            new_supply,
            curve_config.k,
            curve_config.scale,
        );

        let price_impact = if current_price > U256::zero() {
            let impact = ((current_price.saturating_sub(new_price)) * U256::from(10000)) / current_price;
            impact.as_u64() as f64 / 100.0
        } else {
            0.0
        };

        Some(BuySellQuote {
            token_amount: amount,
            currency_amount: return_amount.to_string(),
            price_impact,
            new_price: new_price.to_string(),
        })
    }

    /// Get user balance
    async fn balance(&self, account_json: String) -> Option<String> {
        let account: Account = serde_json::from_str(&account_json).ok()?;
        let balance = self.state.get_balance(&account).await;
        Some(balance.to_string())
    }

    /// Get user position
    async fn user_position(&self, account_json: String) -> Option<fair_launch_abi::UserPositionGQL> {
        let account: Account = serde_json::from_str(&account_json).ok()?;
        self.state
            .user_positions
            .get(&account)
            .await
            .ok()
            .flatten()
            .as_ref()
            .map(|p| p.into())
    }

    /// Get recent trades
    async fn recent_trades(&self, limit: Option<i32>) -> Vec<fair_launch_abi::TradeGQL> {
        let limit = limit.unwrap_or(20).max(1).min(100) as usize;
        self.state.get_trades(0, limit).await.iter().map(|t| t.into()).collect()
    }

    /// Get trades for specific user
    async fn user_trades(&self, account_json: String, limit: Option<i32>) -> Vec<fair_launch_abi::TradeGQL> {
        let account: Account = match serde_json::from_str(&account_json) {
            Ok(acc) => acc,
            Err(_) => return Vec::new(),
        };
        let limit = limit.unwrap_or(20).max(1).min(100) as usize;
        let all_trades = self.state.get_trades(0, 1000).await;

        all_trades
            .into_iter()
            .filter(|t| t.trader == account)
            .take(limit)
            .map(|t| (&t).into())
            .collect()
    }

    /// Get allowance amount that spender can spend on behalf of owner
    async fn allowance(&self, owner_json: String, spender_json: String) -> Option<String> {
        let owner: Account = serde_json::from_str(&owner_json).ok()?;
        let spender: Account = serde_json::from_str(&spender_json).ok()?;
        let allowance = self.state.get_allowance(&owner, &spender).await;
        Some(allowance.to_string())
    }
}

pub struct EmptyMutation;

#[Object]
impl EmptyMutation {
    /// Placeholder mutation (operations are handled via execute_operation)
    async fn _placeholder(&self) -> bool {
        true
    }
}
