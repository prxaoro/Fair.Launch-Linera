# Swap Contract Integration Guide

Complete guide for integrating the Fair Launch swap contract into the platform.

## Overview

The swap contract is the final piece of the Fair Launch platform's token lifecycle:

```
User Creates Token (Factory)
    ↓
Token Launches (Bonding Curve)
    ↓
Users Trade (Buy/Sell on Curve)
    ↓
Curve Completes (Target Raise Met)
    ↓
Token Graduates (Message to Swap)
    ↓
Pool Created (Liquidity Locked Forever)
```

## Prerequisites

1. **Deployed Contracts**:
   - `fair-launch-abi` (shared types)
   - `fair-launch-token` (token with bonding curve)
   - `fair-launch-swap` (this contract)

2. **Configuration**:
   - Swap contract application ID
   - Token contracts must know swap chain ID

## Deployment Steps

### 1. Build the Contract

```bash
cd contracts/swap

# Build for production
cargo build --release --target wasm32-unknown-unknown

# Verify build
ls -lh target/wasm32-unknown-unknown/release/fair_launch_swap.wasm
```

### 2. Publish to Linera

```bash
# Publish the bytecode
linera publish-bytecode \
  target/wasm32-unknown-unknown/release/fair_launch_swap.wasm \
  target/wasm32-unknown-unknown/release/fair_launch_swap.wasm

# Note the bytecode ID returned
BYTECODE_ID="<bytecode-id-from-output>"
```

### 3. Create Application Instance

```bash
# Create swap application
linera create-application $BYTECODE_ID \
  --json-argument '{}' \
  --json-parameters '{}'

# Note the application ID
SWAP_APP_ID="<app-id-from-output>"
```

### 4. Configure Token Contracts

Update token contracts to send graduation messages to the swap contract:

```rust
// In token contract configuration
const SWAP_APPLICATION_ID: &str = env!("SWAP_APP_ID");

// In graduation logic
runtime
    .prepare_message(Message::GraduateToken {
        token_id,
        total_supply,
        total_raised,
    })
    .with_tracking()
    .send_to(
        ChainId::from_str(SWAP_APPLICATION_ID).unwrap()
    );
```

## Message Protocol

### Token → Swap: GraduateToken

**When**: Token bonding curve completes (reaches max supply or target raise)

**Message**:
```rust
Message::GraduateToken {
    token_id: String,        // Format: "token-abc-123"
    total_supply: U256,      // Total tokens in circulation
    total_raised: U256,      // Total base currency raised
}
```

**Validation**:
- `total_supply` must be > 0
- `total_raised` must be > 0
- `token_id` must be unique (no duplicate pools)

**Response**: Swap contract sends `PoolCreated` message back

### Swap → Token: PoolCreated

**When**: Pool successfully created

**Message**:
```rust
Message::PoolCreated {
    token_id: String,    // Original token ID
    pool_id: String,     // Format: "pool-{token_id}"
}
```

**Action**: Token contract updates `is_graduated` flag and stores `pool_id`

## Error Handling

### Contract-Level Errors

The swap contract logs but doesn't panic on these errors:

1. **Invalid Supply**: `total_supply == 0`
   - Logged and skipped
   - No pool created
   - No response message sent

2. **Invalid Raised Amount**: `total_raised == 0`
   - Logged and skipped
   - No pool created
   - No response message sent

3. **Duplicate Graduation**: Token already has a pool
   - Logged as event (not error)
   - Returns existing pool info
   - Sends `PoolCreated` message (idempotent)

### Client-Side Error Handling

```typescript
// Check graduation status before querying pool
const hasGraduated = await swapClient.hasGraduated(tokenId);
if (!hasGraduated) {
  console.log('Token has not graduated yet');
  return;
}

// Get pool with error handling
try {
  const pool = await swapClient.getPoolByToken(tokenId);
  if (!pool) {
    console.error('Pool not found despite graduation status');
    return;
  }

  console.log('Pool TVL:', pool.pool.tvl);
} catch (error) {
  console.error('Failed to fetch pool:', error);
}
```

## GraphQL Integration

### Setup GraphQL Client

```typescript
import { ApolloClient, InMemoryCache, gql } from '@apollo/client';

const swapClient = new ApolloClient({
  uri: `http://localhost:8080/chains/${SWAP_CHAIN_ID}/applications/${SWAP_APP_ID}`,
  cache: new InMemoryCache(),
});
```

### Query Examples

#### Get Pool for Graduated Token
```typescript
const GET_POOL = gql`
  query GetTokenPool($tokenId: String!) {
    poolByToken(token_id: $tokenId) {
      pool {
        pool_id
        token_liquidity
        base_liquidity
        tvl
        is_locked
        created_at
      }
      is_active
      age_seconds
    }
    hasGraduated(token_id: $tokenId)
  }
`;

const { data } = await swapClient.query({
  query: GET_POOL,
  variables: { tokenId: 'token-abc-123' }
});

if (data.hasGraduated && data.poolByToken) {
  const { pool } = data.poolByToken;
  console.log(`Pool ${pool.pool_id} has ${pool.tvl} TVL (locked)`);
}
```

#### List All Pools
```typescript
const LIST_POOLS = gql`
  query ListPools($offset: Int!, $limit: Int!) {
    pools(offset: $offset, limit: $limit) {
      pool_id
      token_id
      tvl
      is_locked
      created_at
    }
    stats {
      total_pools
      total_tvl
    }
  }
`;

const { data } = await swapClient.query({
  query: LIST_POOLS,
  variables: { offset: 0, limit: 20 }
});

console.log(`Total pools: ${data.stats.total_pools}`);
console.log(`Total TVL: ${data.stats.total_tvl}`);
```

## Frontend Integration

### React Component for Pool Display

```typescript
import React from 'react';
import { useQuery, gql } from '@apollo/client';

const POOL_QUERY = gql`
  query GetPool($tokenId: String!) {
    poolByToken(token_id: $tokenId) {
      pool {
        pool_id
        token_liquidity
        base_liquidity
        tvl
        initial_ratio
        created_at
      }
      age_seconds
    }
  }
`;

export function PoolInfo({ tokenId }: { tokenId: string }) {
  const { data, loading, error } = useQuery(POOL_QUERY, {
    variables: { tokenId },
    pollInterval: 10000, // Refresh every 10s
  });

  if (loading) return <div>Loading pool info...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!data?.poolByToken) return <div>No pool found</div>;

  const { pool, age_seconds } = data.poolByToken;

  return (
    <div className="pool-info">
      <h3>Liquidity Pool (Locked)</h3>
      <div className="pool-stats">
        <div>
          <label>Total Value Locked</label>
          <span>{formatTVL(pool.tvl)}</span>
        </div>
        <div>
          <label>Token Liquidity</label>
          <span>{formatTokens(pool.token_liquidity)}</span>
        </div>
        <div>
          <label>Base Liquidity</label>
          <span>{formatBase(pool.base_liquidity)}</span>
        </div>
        <div>
          <label>Initial Price</label>
          <span>{formatPrice(pool.initial_ratio)}</span>
        </div>
        <div>
          <label>Pool Age</label>
          <span>{formatAge(age_seconds)}</span>
        </div>
      </div>
      <div className="lock-status">
        🔒 Liquidity permanently locked (anti-rug)
      </div>
    </div>
  );
}
```

### Pool List Component

```typescript
const POOLS_QUERY = gql`
  query ListPools($offset: Int!, $limit: Int!) {
    pools(offset: $offset, limit: $limit) {
      pool_id
      token_id
      tvl
      created_at
    }
    stats {
      total_pools
      total_tvl
    }
  }
`;

export function PoolsList() {
  const [page, setPage] = React.useState(0);
  const limit = 10;

  const { data, loading } = useQuery(POOLS_QUERY, {
    variables: { offset: page * limit, limit },
  });

  if (loading) return <div>Loading...</div>;

  const totalPages = Math.ceil((data?.stats.total_pools || 0) / limit);

  return (
    <div>
      <h2>Graduated Tokens ({data.stats.total_pools})</h2>
      <p>Total Liquidity Locked: {formatTVL(data.stats.total_tvl)}</p>

      <div className="pools-grid">
        {data.pools.map(pool => (
          <PoolCard key={pool.pool_id} pool={pool} />
        ))}
      </div>

      <Pagination
        page={page}
        totalPages={totalPages}
        onPageChange={setPage}
      />
    </div>
  );
}
```

## Monitoring and Analytics

### Key Metrics to Track

1. **Pool Creation Rate**
   ```graphql
   query PoolMetrics {
     stats {
       total_pools
       total_tvl
     }
     recentPools(limit: 10) {
       pool_id
       created_at
     }
   }
   ```

2. **TVL Growth**
   ```graphql
   query TVLMetrics {
     stats {
       total_tvl
       average_pool_tvl
     }
     topPoolsByTvl(limit: 5) {
       token_id
       tvl
     }
   }
   ```

3. **Graduation Success Rate**
   - Track tokens created vs tokens graduated
   - Monitor time-to-graduation
   - Analyze graduation completion rate

### Logging

The contract logs these events:

```rust
// Graduation received
"Graduation request received for token {token_id} from chain {chain_id}"

// Pool created
"Pool created successfully: {pool_id} for token {token_id} with {supply} tokens and {raised} base currency (locked permanently)"

// Pool creation message sent
"Sent PoolCreated message for token {token_id} to chain {chain_id}"

// Errors
"Invalid graduation: token {token_id} has zero supply"
"Invalid graduation: token {token_id} has zero raised amount"
"Failed to create pool for token {token_id}: {error}"
```

## Testing Integration

### End-to-End Test Flow

```rust
#[tokio::test]
async fn test_token_graduation_integration() {
    // 1. Deploy swap contract
    let swap_app = deploy_swap_contract().await;

    // 2. Deploy token contract
    let token_app = deploy_token_contract(swap_app.id()).await;

    // 3. Initialize token
    token_app.initialize(
        creator,
        metadata,
        curve_config,
    ).await;

    // 4. Buy tokens until curve completes
    for _ in 0..100 {
        token_app.buy(U256::from(10_000_000)).await;
    }

    // 5. Trigger graduation
    token_app.graduate().await;

    // Wait for cross-chain message processing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 6. Verify pool created
    let pool = swap_app.get_pool_by_token(token_app.id()).await;
    assert!(pool.is_some());
    assert!(pool.unwrap().is_locked);

    // 7. Verify token updated
    let token_state = token_app.get_state().await;
    assert!(token_state.is_graduated);
    assert!(token_state.dex_pool_id.is_some());
}
```

## Security Considerations

### Message Authentication

The swap contract relies on Linera's message authentication:

```rust
// Message sender is authenticated by runtime
let sender_chain = self.runtime.message_id()
    .expect("Message must have an ID")
    .chain_id;
```

**Security Properties**:
- Messages are cryptographically signed
- Sender chain is verified by Linera protocol
- No additional authentication needed

### Reentrancy Protection

The contract is reentrancy-safe:

```rust
// State is saved atomically after all operations
async fn store(mut self) {
    self.state.save().await.expect("Failed to save");
}
```

**Protection Mechanisms**:
- No external calls during state mutations
- Atomic state updates
- No re-entrance possible during message handling

### Input Validation

All inputs are validated:

```rust
// Validate token supply
if total_supply == U256::zero() {
    log_error("Invalid supply");
    return;
}

// Validate raised amount
if total_raised == U256::zero() {
    log_error("Invalid raised amount");
    return;
}

// Check duplicate
if self.state.has_pool(&token_id).await? {
    log_event("Duplicate graduation");
    return;
}
```

## Troubleshooting

### Pool Not Created

**Symptoms**: Token graduated but no pool found

**Diagnosis**:
1. Check swap contract logs for errors
2. Verify graduation message was sent
3. Check message reached swap chain
4. Verify token_id format

**Solution**:
```bash
# Check swap contract logs
linera query-chain $SWAP_CHAIN_ID logs

# Verify message status
linera query-message $MESSAGE_ID

# Manually trigger graduation
linera execute-operation $TOKEN_APP_ID Graduate
```

### Duplicate Pool Error

**Symptoms**: Error when trying to graduate already-graduated token

**Diagnosis**: Token already has a pool

**Solution**: This is expected behavior. Check existing pool:
```graphql
query {
  poolByToken(token_id: "token-123") {
    pool { pool_id }
  }
}
```

### Query Timeout

**Symptoms**: GraphQL queries timeout or hang

**Diagnosis**:
1. Too many pools (pagination issue)
2. Network latency
3. Contract not responding

**Solution**:
```typescript
// Use smaller page sizes
const pools = await client.getPools(0, 10); // Not 100

// Add timeout
const promise = client.getPool(poolId);
const timeout = new Promise((_, reject) =>
  setTimeout(() => reject(new Error('Timeout')), 5000)
);
const result = await Promise.race([promise, timeout]);
```

## Production Checklist

Before deploying to production:

- [ ] All tests passing (`cargo test`)
- [ ] Contract builds successfully
- [ ] GraphQL queries tested
- [ ] Message flow verified end-to-end
- [ ] Error handling tested
- [ ] Frontend integration complete
- [ ] Monitoring setup
- [ ] Documentation updated
- [ ] Security audit completed
- [ ] Load testing performed

## Support

For integration support:
- Review examples in `EXAMPLES.md`
- Check Linera docs: https://linera.dev
- Open GitHub issue with logs and reproduction steps
