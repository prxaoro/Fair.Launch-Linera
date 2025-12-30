#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use fair_launch_abi::{
    bonding_curve, Message, TokenAbi, TokenOperation, Trade,
};
use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::{Account, AccountOwner, Amount},
    views::View,
    Contract, ContractRuntime,
};
use primitive_types::U256;
use thiserror::Error;

use crate::state::TokenState;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: U256, need: U256 },

    #[error("Would exceed max supply: current {current}, adding {adding}, max {max}")]
    ExceedsMaxSupply { current: U256, adding: U256, max: U256 },

    #[error("Slippage exceeded: cost {cost}, max allowed {max_cost}")]
    SlippageExceeded { cost: U256, max_cost: U256 },

    #[error("Slippage exceeded: return {return_amount}, min required {min_return}")]
    SlippageExceededSell { return_amount: U256, min_return: U256 },

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Insufficient native token balance: have {have}, need {need}")]
    InsufficientNativeBalance { have: Amount, need: Amount },

    #[error("Amount conversion error")]
    AmountConversionError,

    #[error("State error: {0}")]
    StateError(String),
}

pub struct TokenContract {
    state: TokenState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(TokenContract);

impl WithContractAbi for TokenContract {
    type Abi = TokenAbi;
}

impl Contract for TokenContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = TokenState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        TokenContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // Token is initialized via Initialize operation from factory
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            TokenOperation::Initialize {
                creator,
                metadata,
                curve_config,
            } => {
                let token_id = format!("{}", self.runtime.application_id().forget_abi());
                let created_at = self.runtime.system_time();

                self.state
                    .initialize(token_id.clone(), creator, metadata.clone(), curve_config, created_at)
                    .await
                    .expect("Failed to initialize token");
            }

            TokenOperation::Buy { amount, max_cost } => {
                self.execute_buy(amount, max_cost).await
                    .expect("Buy operation failed");
            }

            TokenOperation::Sell { amount, min_return } => {
                self.execute_sell(amount, min_return).await
                    .expect("Sell operation failed");
            }

            TokenOperation::Graduate => {
                self.execute_graduation().await;
            }

            TokenOperation::Approve { spender, amount } => {
                self.execute_approve(spender, amount).await
                    .expect("Approve operation failed");
            }

            TokenOperation::TransferFrom { from, to, amount } => {
                self.execute_transfer_from(from, to, amount).await
                    .expect("TransferFrom operation failed");
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::TokenCreated {
                token_id,
                creator,
                metadata,
                curve_config,
            } => {
                // Initialize token when created by factory
                let created_at = self.runtime.system_time();
                self.state
                    .initialize(token_id, creator, metadata, curve_config, created_at)
                    .await
                    .expect("Failed to initialize token from message");
            }

            Message::TradeExecuted { .. } => {
                // Trade notifications - balance already updated in execute_operation
                // This message is just for event tracking/notifications
            }

            Message::PoolCreated { token_id: _, pool_id } => {
                self.state.dex_pool_id.set(Some(pool_id));
                self.state.is_graduated.set(true);
            }

            _ => {
                // Ignore other messages
            }
        }
    }

    async fn store(self) {
        // State is automatically persisted by linera-views
    }
}

impl TokenContract {
    /// Execute a buy operation
    async fn execute_buy(&mut self, amount: U256, max_cost: U256) -> Result<(), TokenError> {
        // Validate input
        if amount == U256::zero() {
            return Err(TokenError::InvalidAmount);
        }

        // Get caller's account (includes chain_id and owner)
        let caller = self.owner_account();

        let current_supply = *self.state.current_supply.get();
        let curve_config = self.state.curve_config.get().clone();

        // Calculate cost using bonding curve
        let cost = bonding_curve::calculate_buy_cost(
            current_supply,
            amount,
            curve_config.k,
            curve_config.scale,
        );

        // Check slippage protection
        if cost > max_cost {
            return Err(TokenError::SlippageExceeded { cost, max_cost });
        }

        // Check if curve would be completed
        let new_supply = current_supply + amount;
        if new_supply > curve_config.max_supply {
            return Err(TokenError::ExceedsMaxSupply {
                current: current_supply,
                adding: amount,
                max: curve_config.max_supply,
            });
        }

        // Calculate creator fee (e.g., 3% = 300 basis points)
        let fee_amount = (cost * U256::from(curve_config.creator_fee_bps)) / U256::from(10000);
        let creator = self.state.creator.get().clone().expect("Creator not set");

        // CRITICAL: Transfer cost from buyer
        // 1. Transfer fee to creator
        // 2. Transfer remaining to application
        let native_cost = Self::u256_to_amount(cost)?;
        let native_fee = Self::u256_to_amount(fee_amount)?;
        let native_to_app = native_cost.saturating_sub(native_fee);

        // Transfer fee to creator
        if native_fee > Amount::ZERO {
            self.fund_account(creator, native_fee)?;
        }

        // Transfer remaining to application
        let application = self.application_account();
        if native_to_app > Amount::ZERO {
            self.fund_account(application, native_to_app)?;
        }

        // Update state
        self.state.current_supply.set(new_supply);
        let total_raised = *self.state.total_raised.get();
        self.state.total_raised.set(total_raised + cost);

        // Update user balance
        let current_balance = self.state.get_balance(&caller).await;
        self.state
            .set_balance(caller, current_balance + amount)
            .await
            .expect("Failed to update balance");

        // Record trade
        let trade_id = format!("{}-{}", self.runtime.system_time().micros(), self.state.trade_count.get());
        let new_price = bonding_curve::calculate_current_price(new_supply, curve_config.k, curve_config.scale);

        let trade = Trade {
            token_id: self.state.token_id.get().clone(),
            trader: caller,
            is_buy: true,
            token_amount: amount,
            currency_amount: cost,
            price: new_price,
            timestamp: self.runtime.system_time(),
        };

        self.state
            .record_trade(trade_id, trade.clone())
            .await
            .expect("Failed to record trade");

        // Check if curve is complete
        if self.state.is_curve_complete() {
            self.execute_graduation().await;
        }

        Ok(())
    }

    /// Execute a sell operation
    async fn execute_sell(&mut self, amount: U256, min_return: U256) -> Result<(), TokenError> {
        // Validate input
        if amount == U256::zero() {
            return Err(TokenError::InvalidAmount);
        }

        // Get caller's account (includes chain_id and owner)
        let caller = self.owner_account();

        // Check user has enough balance
        let current_balance = self.state.get_balance(&caller).await;
        if current_balance < amount {
            return Err(TokenError::InsufficientBalance {
                have: current_balance,
                need: amount,
            });
        }

        let current_supply = *self.state.current_supply.get();
        let curve_config = self.state.curve_config.get().clone();

        // Calculate return using bonding curve
        let return_amount = bonding_curve::calculate_sell_return(
            current_supply,
            amount,
            curve_config.k,
            curve_config.scale,
        );

        // Check slippage protection
        if return_amount < min_return {
            return Err(TokenError::SlippageExceededSell {
                return_amount,
                min_return,
            });
        }

        // Calculate creator fee on sell
        let fee_amount = (return_amount * U256::from(curve_config.creator_fee_bps)) / U256::from(10000);
        let net_return = return_amount.saturating_sub(fee_amount);
        let creator = self.state.creator.get().clone().expect("Creator not set");

        // CRITICAL: Transfer from application
        // 1. Transfer fee to creator
        // 2. Transfer net return to seller
        let native_fee = Self::u256_to_amount(fee_amount)?;
        let native_net_return = Self::u256_to_amount(net_return)?;
        let seller_account = self.owner_account();

        // Transfer fee to creator
        if native_fee > Amount::ZERO {
            self.transfer_from_application(creator, native_fee)?;
        }

        // Transfer net return to seller
        if native_net_return > Amount::ZERO {
            self.transfer_from_application(seller_account, native_net_return)?;
        }

        // Update state
        let new_supply = current_supply - amount;
        self.state.current_supply.set(new_supply);
        let total_raised = *self.state.total_raised.get();
        self.state.total_raised.set(total_raised.saturating_sub(return_amount));

        // Update user balance
        self.state
            .set_balance(caller, current_balance - amount)
            .await
            .expect("Failed to update balance");

        // Record trade
        let trade_id = format!("{}-{}", self.runtime.system_time().micros(), self.state.trade_count.get());
        let new_price = bonding_curve::calculate_current_price(new_supply, curve_config.k, curve_config.scale);

        let trade = Trade {
            token_id: self.state.token_id.get().clone(),
            trader: caller,
            is_buy: false,
            token_amount: amount,
            currency_amount: return_amount,
            price: new_price,
            timestamp: self.runtime.system_time(),
        };

        self.state
            .record_trade(trade_id, trade.clone())
            .await
            .expect("Failed to record trade");

        Ok(())
    }

    /// Execute graduation to DEX
    async fn execute_graduation(&mut self) {
        if *self.state.is_graduated.get() {
            return; // Already graduated
        }

        let token_id = self.state.token_id.get().clone();
        let total_supply = *self.state.current_supply.get();
        let total_raised = *self.state.total_raised.get();

        // Send graduation message to swap chain
        // In a real implementation, this would be the actual swap application ID
        let swap_chain = self.runtime.chain_id();

        self.runtime
            .prepare_message(Message::GraduateToken {
                token_id,
                total_supply,
                total_raised,
            })
            .with_tracking()
            .send_to(swap_chain);
    }

    /// Execute approve operation - allows spender to transfer tokens on behalf of owner
    async fn execute_approve(&mut self, spender: Account, amount: U256) -> Result<(), TokenError> {
        let owner = self.owner_account();

        // Set allowance
        self.state
            .set_allowance(owner, spender, amount)
            .await
            .map_err(|e| TokenError::StateError(e.to_string()))?;

        Ok(())
    }

    /// Execute transferFrom operation - transfer tokens from owner to recipient using allowance
    async fn execute_transfer_from(&mut self, from: Account, to: Account, amount: U256) -> Result<(), TokenError> {
        // Validate input
        if amount == U256::zero() {
            return Err(TokenError::InvalidAmount);
        }

        // Get spender (caller)
        let spender = self.owner_account();

        // Check allowance
        let allowance = self.state.get_allowance(&from, &spender).await;
        if allowance < amount {
            return Err(TokenError::InsufficientBalance {
                have: allowance,
                need: amount,
            });
        }

        // Check from account has enough balance
        let from_balance = self.state.get_balance(&from).await;
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance {
                have: from_balance,
                need: amount,
            });
        }

        // Decrease allowance
        self.state
            .decrease_allowance(&from, &spender, amount)
            .await
            .map_err(|e| TokenError::StateError(e.to_string()))?;

        // Transfer tokens from -> to
        self.state
            .set_balance(from.clone(), from_balance - amount)
            .await
            .map_err(|e| TokenError::StateError(e.to_string()))?;

        let to_balance = self.state.get_balance(&to).await;
        self.state
            .set_balance(to, to_balance + amount)
            .await
            .map_err(|e| TokenError::StateError(e.to_string()))?;

        Ok(())
    }

    /// Convert U256 to Amount (native token amount)
    fn u256_to_amount(value: U256) -> Result<Amount, TokenError> {
        // Convert U256 to u128 for Amount
        if value > U256::from(u128::MAX) {
            return Err(TokenError::AmountConversionError);
        }
        Ok(Amount::from_tokens(value.as_u128()))
    }

    /// Convert Amount to U256
    fn amount_to_u256(amount: Amount) -> U256 {
        // Amount is internally u128 units (attos)
        U256::from(u128::from(amount))
    }

    /// Get the owner account (authenticated signer on current chain)
    fn owner_account(&mut self) -> Account {
        Account {
            chain_id: self.runtime.chain_id(),
            owner: match self.runtime.authenticated_signer() {
                Some(owner) => owner,
                _ => AccountOwner::CHAIN,
            },
        }
    }

    /// Get the application account (application-owned funds)
    fn application_account(&mut self) -> Account {
        Account {
            chain_id: self.runtime.chain_id(),
            owner: AccountOwner::from(self.runtime.application_id().forget_abi()),
        }
    }

    /// Transfer native tokens from buyer to application
    /// Copied from linera-meme winner pattern
    fn fund_account(&mut self, to: Account, amount: Amount) -> Result<(), TokenError> {
        if amount <= Amount::ZERO {
            return Err(TokenError::InvalidAmount);
        }

        let signer = self.runtime.authenticated_signer().unwrap();
        let ownership = self.runtime.chain_ownership();

        // Check if signer is chain owner (can transfer from chain balance)
        let can_from_chain = ownership.all_owners().any(|&owner| owner == signer);

        let owner_balance = self.runtime.owner_balance(signer);
        let _chain_balance = self.runtime.chain_balance();

        // Try to take from owner balance first
        let from_owner_balance = if amount <= owner_balance {
            amount
        } else {
            owner_balance
        };

        // If owner balance insufficient, take from chain balance (if authorized)
        let from_chain_balance = if amount <= owner_balance || !can_from_chain {
            Amount::ZERO
        } else {
            amount.try_sub(owner_balance).map_err(|_| {
                TokenError::InsufficientNativeBalance {
                    have: owner_balance,
                    need: amount,
                }
            })?
        };

        // Verify sufficient total balance
        if from_owner_balance.try_add(from_chain_balance).is_err()
            || from_owner_balance.try_add(from_chain_balance).unwrap() < amount {
            return Err(TokenError::InsufficientNativeBalance {
                have: from_owner_balance.try_add(from_chain_balance).unwrap_or(Amount::ZERO),
                need: amount,
            });
        }

        // ACTUAL TRANSFERS using runtime.transfer()
        if from_owner_balance > Amount::ZERO {
            self.runtime.transfer(signer, to, from_owner_balance);
        }
        if from_chain_balance > Amount::ZERO {
            self.runtime.transfer(AccountOwner::CHAIN, to, from_chain_balance);
        }

        Ok(())
    }

    /// Transfer native tokens from application to user (for sells/refunds)
    fn transfer_from_application(&mut self, to: Account, amount: Amount) -> Result<(), TokenError> {
        if amount <= Amount::ZERO {
            return Ok(());
        }

        let application_owner = AccountOwner::from(self.runtime.application_id().forget_abi());
        let application_balance = self.runtime.owner_balance(application_owner);

        if application_balance < amount {
            return Err(TokenError::InsufficientNativeBalance {
                have: application_balance,
                need: amount,
            });
        }

        // Transfer from application to user
        self.runtime.transfer(application_owner, to, amount);

        Ok(())
    }
}
