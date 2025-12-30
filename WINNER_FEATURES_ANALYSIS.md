# Winner Features Analysis - What We're Missing

## Source: linera-meme Winner Project
**Location:** Reference implementation analyzed for architectural patterns

---

## 1. CRITICAL: Actual Payment System (Currently COMPLETELY MISSING)

### What We Have Now:
- ❌ Only **calculations** of costs (bonding curve math)
- ❌ NO actual fund transfers
- ❌ Users can "buy" tokens without paying anything
- ❌ Users get refunds when selling but money never left their account

### What Winners Do:
```rust
// From linera-meme/meme/src/contract.rs:309-341
fn fund_account(&mut self, to: Account, amount: Amount) {
    assert!(amount > Amount::ZERO, "Invalid fund amount");

    let signer = self.runtime.authenticated_signer().unwrap();
    let ownership = self.runtime.chain_ownership();
    let can_from_chain = ownership.all_owners().any(|&owner| owner == signer);

    let owner_balance = self.runtime.owner_balance(signer);
    let chain_balance = self.runtime.chain_balance();

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
        amount.try_sub(owner_balance).expect("Invalid amount")
    };

    assert!(from_owner_balance <= owner_balance, "Insufficient balance");
    assert!(from_chain_balance <= chain_balance, "Insufficient balance");

    // ACTUAL TRANSFERS using runtime.transfer()
    if from_owner_balance > Amount::ZERO {
        self.runtime.transfer(signer, to, from_owner_balance);
    }
    if from_chain_balance > Amount::ZERO {
        self.runtime.transfer(AccountOwner::CHAIN, to, from_chain_balance);
    }
}
```

### What We Must Implement:
1. **In token/src/contract.rs `execute_buy()`:**
   - After calculating `cost`, ACTUALLY transfer native tokens from buyer to application
   - Use `runtime.transfer(buyer_owner, application_account, cost)`
   - Verify balance before transfer

2. **In token/src/contract.rs `execute_sell()`:**
   - After calculating `return_amount`, ACTUALLY transfer native tokens from application to seller
   - Use `runtime.transfer(application_owner, seller_account, return_amount)`
   - Verify application has sufficient native token balance

3. **Need to add:**
   ```rust
   fn fund_account(&mut self, to: Account, amount: Amount) {
       // Exact implementation from winner
   }
   ```

---

## 2. CRITICAL: Account Structure (We're Using Wrong Type)

### What We Have Now:
```rust
// We use ChainId everywhere
let caller = self.runtime.chain_id();  // ❌ WRONG
self.state.get_balance(&caller).await  // ❌ Type is ChainId
```

### What Winners Do:
```rust
// Winners use Account structure everywhere
pub struct Account {
    pub chain_id: ChainId,
    pub owner: AccountOwner,  // The actual wallet/user
}

// Different account types:
fn owner_account(&mut self) -> Account {
    Account {
        chain_id: self.runtime.chain_id(),
        owner: match self.runtime.authenticated_signer() {
            Some(owner) => owner,
            _ => AccountOwner::CHAIN,
        },
    }
}

fn application_account(&mut self) -> Account {
    Account {
        chain_id: self.runtime.chain_id(),
        owner: AccountOwner::from(self.runtime.application_id().forget_abi()),
    }
}

fn message_caller_account(&mut self) -> Account {
    Account {
        chain_id: self.runtime.message_origin_chain_id().unwrap(),
        owner: AccountOwner::from(self.runtime.authenticated_caller_id().unwrap()),
    }
}
```

### What We Must Change:
1. **Update abi/src/lib.rs:**
   - Change all ChainId fields to Account
   - `pub trader: Account` instead of `pub trader: ChainId`

2. **Update token/src/state.rs:**
   - `balances: MapView<Account, U256>` instead of `MapView<ChainId, U256>`
   - `user_positions: MapView<Account, UserPosition>` instead of `MapView<ChainId, ...>`

3. **Update all contract operations:**
   - Use `self.owner_account()` instead of `self.runtime.chain_id()`
   - Use `self.application_account()` for application-owned funds

---

## 3. Message-Based Transfer Architecture

### What We Have Now:
```rust
// Direct state updates
self.state.set_balance(caller, current_balance + amount).await  // ❌
```

### What Winners Do:
```rust
// Operations send messages to creator chain
fn on_op_transfer(&mut self, to: Account, amount: Amount) -> Result<MemeResponse, MemeError> {
    let from = self.owner_account();
    self.runtime
        .prepare_message(MemeMessage::Transfer { from, to, amount })
        .with_authentication()
        .send_to(self.runtime.application_creator_chain_id());  // Send to creator chain
    Ok(MemeResponse::Ok)
}

// Messages execute on creator chain only
async fn execute_message(&mut self, message: Self::Message) {
    // Ensure messages only run on creator chain
    if self.runtime.chain_id() != self.runtime.application_creator_chain_id() {
        panic!("Messages must only be run on creation chain");
    }

    match message {
        MemeMessage::Transfer { from, to, amount } => {
            self.state.transfer(from, to, amount).await
        }
    }
}
```

### Why This Matters:
- **Single source of truth** - All state updates happen on creator chain only
- **Prevents desynchronization** - User chains can't have different balances
- **Cross-chain safety** - Operations from any chain update same state

### What We Must Implement:
1. **Add to abi/src/lib.rs:**
   ```rust
   pub enum Message {
       TradeExecuted { trader: Account, is_buy: bool, token_amount: U256, currency_amount: U256 },
       Transfer { from: Account, to: Account, amount: U256 },
   }
   ```

2. **In token/src/contract.rs:**
   - `execute_operation()` sends messages, doesn't modify state
   - `execute_message()` modifies state on creator chain only
   - Add creator chain check at start of `execute_message()`

---

## 4. Proper Token Instantiation

### What We Have Now:
```rust
async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
    // Token is initialized via Initialize operation from factory
    // ❌ Does NOTHING during instantiation
}
```

### What Winners Do:
```rust
async fn instantiate(&mut self, mut instantiation_argument: InstantiationArgument) {
    // 1. Validate parameters
    self.runtime.application_parameters();
    let signer = self.runtime.authenticated_signer().unwrap();
    assert!(self.creator_signer() == signer, "Invalid owner");

    // 2. Initialize state with creator, application, metadata
    self.state
        .instantiate(creator, application, instantiation_argument)
        .await
        .expect("Failed instantiate");

    // 3. Mint initial balance to creator (100 tokens for testing)
    self.state
        .mint(creator, self.state.initial_owner_balance().await)
        .await
        .expect("Failed initialize balance");

    // 4. Initialize liquidity if configured
    if let Some(liquidity) = self.initial_liquidity() {
        let swap_creator_chain = self.swap_creator_chain_id();
        self.state
            .initialize_liquidity(liquidity, swap_creator_chain)
            .await
            .expect("Failed initialize liquidity");
    }

    // 5. Register with AMS (Application Management System)
    self.register_application().await;

    // 6. Register logo with blob gateway
    self.register_logo().await;

    // 7. Create liquidity pool immediately
    self.create_liquidity_pool()
        .await
        .expect("Failed create liquidity pool");
}
```

### What We Must Implement:
1. **Actual instantiation logic** in `token/src/contract.rs`
2. **Mint initial creator balance** (100 tokens for testing)
3. **Initialize liquidity** with initial_liquidity parameter
4. **Create pool during instantiation** (not waiting for graduation)

---

## 5. Application Parameters Pattern

### What We Have Now:
```rust
type Parameters = ();  // ❌ No parameters
```

### What Winners Do:
```rust
pub struct MemeParameters {
    pub creator: Account,
    pub initial_liquidity: Option<Liquidity>,
    pub virtual_initial_liquidity: bool,
    pub swap_creator_chain_id: ChainId,
}

// Access parameters anywhere in contract:
fn creator(&mut self) -> Account {
    self.runtime.application_parameters().creator
}

fn initial_liquidity(&mut self) -> Option<Liquidity> {
    self.runtime.application_parameters().initial_liquidity
}
```

### What We Must Add:
1. **In abi/src/lib.rs:**
   ```rust
   pub struct TokenParameters {
       pub creator: Account,
       pub metadata: TokenMetadata,
       pub curve_config: BondingCurveConfig,
       pub swap_application_id: ApplicationId,
   }
   ```

2. **In token/src/contract.rs:**
   ```rust
   type Parameters = TokenParameters;

   fn creator(&mut self) -> Account {
       self.runtime.application_parameters().creator
   }
   ```

---

## 6. Approve/Allowance System for DEX

### What We Have Now:
```rust
// ❌ No approve mechanism
// ❌ DEX can't spend user tokens
```

### What Winners Do:
```rust
// state.rs:186-223
pub async fn approve(
    &mut self,
    owner: Account,
    spender: Account,
    amount: Amount,
) -> Result<(), MemeError> {
    // Self approve not allowed
    if owner == spender {
        return Err(MemeError::InvalidOwner);
    }

    let owner_balance = self.balance_of(owner).await;
    if owner_balance < amount {
        return Err(MemeError::InsufficientFunds);
    }

    // Get or create allowances map for owner
    let mut allowances = if let Some(_allowances) = self.allowances.get(&owner).await? {
        _allowances
    } else {
        HashMap::new()
    };

    // Add to existing allowance (not replace)
    let spender_allowance = if let Some(allowance) = allowances.get(&spender) {
        allowance.try_add(amount)?
    } else {
        amount
    };

    // Deduct from owner balance (locked for allowance)
    self.balances.insert(&owner, owner_balance.try_sub(amount)?)?;

    // Store updated allowance
    allowances.insert(spender, spender_allowance);
    Ok(self.allowances.insert(&owner, allowances)?)
}

// state.rs:225-246
pub async fn transfer_from(
    &mut self,
    owner: Account,      // The account spending the allowance
    from: Account,       // Where tokens come from
    to: Account,         // Where tokens go to
    amount: Amount,
) -> Result<(), MemeError> {
    let Some(mut allowances) = self.allowances.get(&from).await? else {
        panic!("Invalid from");
    };
    let Some(&allowance) = allowances.get(&owner) else {
        panic!("Invalid owner");
    };
    assert!(allowance >= amount, "Insufficient allowance");

    // Update recipient balance
    let balance = match self.balances.get(&to).await? {
        Some(balance) => balance.try_add(amount)?,
        _ => amount,
    };
    self.balances.insert(&to, balance)?;

    // Reduce allowance
    allowances.insert(owner, allowance.try_sub(amount)?);
    Ok(self.allowances.insert(&from, allowances)?)
}
```

### What We Must Add:
1. **In token/src/state.rs:**
   ```rust
   pub allowances: MapView<Account, HashMap<Account, U256>>,
   ```

2. **Operations:**
   - `TokenOperation::Approve { spender: Account, amount: U256 }`
   - `TokenOperation::TransferFrom { from: Account, to: Account, amount: U256 }`

3. **Why:** DEX needs to spend user tokens when creating pools!

---

## 7. State Management Patterns from Winners

### What Winners Do Better:

1. **Result-based error handling everywhere:**
   ```rust
   pub async fn transfer(&mut self, from: Account, to: Account, amount: Amount)
       -> Result<(), MemeError>  // ❌ We use expect() and panic!
   ```

2. **Proper loading with namespaced keys:**
   ```rust
   // gmic-buildathon/src/state.rs:172-240
   pub async fn load(context: ViewStorageContext) -> Result<Self, ViewError> {
       let owner_context = context.clone_with_base_key(b"gm_owner".to_vec());
       let last_gm_context = context.clone_with_base_key(b"gm_last_gm".to_vec());
       let owner = RegisterView::load(owner_context).await?;
       let last_gm = MapView::load(last_gm_context).await?;
       // ... for each field
   }
   ```

3. **Explicit save() calls:**
   ```rust
   pub async fn store(self) {
       self.state.save().await.expect("Failed to save state");
   }
   ```

---

## SUMMARY: What We MUST Fix To Match Winners

### Priority 1 (BLOCKING - App doesn't work without these):
1. ✅ **Add runtime.transfer() payment system** - Users must actually pay to buy tokens
2. ✅ **Convert ChainId → Account everywhere** - Critical type mismatch
3. ✅ **Message-based transfer architecture** - Prevents state desync

### Priority 2 (IMPORTANT - App incomplete without these):
4. ✅ **Proper token instantiation** - Mint initial tokens, create pool
5. ✅ **Application parameters** - Creator, metadata, swap app ID
6. ✅ **Approve/allowance system** - DEX integration requires this

### Priority 3 (NICE TO HAVE):
7. ⚠️ Namespaced state keys for cleaner storage
8. ⚠️ Better error handling (Result instead of panic)
9. ⚠️ Explicit save() instead of auto-persist

---

## Next Steps:
1. Start with Priority 1 items (payment system, Account structure, messages)
2. Test each change thoroughly
3. Move to Priority 2 (instantiation, parameters, allowances)
4. Full integration testing against winner patterns
