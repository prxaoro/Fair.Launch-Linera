# Fair Launch Swap Contract

Production-ready swap contract for the Fair Launch platform. Creates and manages permanently locked liquidity pools for tokens that graduate from the bonding curve.

## Features

- **Automatic Pool Creation**: Receives `GraduateToken` messages from token contracts and creates locked pools
- **Permanent Liquidity Lock**: All pools are permanently locked (anti-rug feature)
- **Pool Registry**: Maintains searchable registry of all graduated tokens
- **GraphQL API**: Rich query interface for pool data and statistics
- **Production Security**: Input validation, idempotent operations, error handling

## Architecture

### State Management

- `pools`: MapView storing all pool information (pool_id → PoolInfo)
- `token_to_pool`: MapView for token → pool lookup (token_id → pool_id)
- `total_pools`: Counter of all created pools
- `total_tvl`: Aggregate total value locked across all pools

### Message Flow

```
Token Contract (Bonding Curve Complete)
    ↓
    Message::GraduateToken { token_id, total_supply, total_raised }
    ↓
Swap Contract
    → Validates inputs (non-zero supply/raised)
    → Checks for duplicate (idempotent)
    → Creates locked pool
    → Updates registry
    ↓
    Message::PoolCreated { token_id, pool_id }
    ↓
Token Contract (Updates graduation status)
```

## Pool Structure

Each pool contains:
- `pool_id`: Unique identifier (format: "pool-{token_id}")
- `token_id`: Associated token contract ID
- `token_liquidity`: Total token supply in pool
- `base_liquidity`: Total base currency in pool (from bonding curve)
- `initial_ratio`: Price ratio (base_per_token, scaled by 1M)
- `is_locked`: Always `true` (permanent lock)
- `lock_expires_at`: Always `None` (never expires)
- `tvl`: Total value locked (2 × base_liquidity)

## GraphQL API

### Queries

#### Get Swap Statistics
```graphql
query {
  stats {
    total_pools
    total_tvl
    average_pool_tvl
  }
}
```

#### List All Pools (Paginated)
```graphql
query {
  pools(offset: 0, limit: 20) {
    pool_id
    token_id
    token_liquidity
    base_liquidity
    initial_ratio
    tvl
    is_locked
    created_at
  }
}
```

#### Get Pool by ID
```graphql
query {
  pool(pool_id: "pool-token-abc-123") {
    pool {
      pool_id
      token_id
      token_liquidity
      base_liquidity
      tvl
      is_locked
      created_at
    }
    is_active
    age_seconds
  }
}
```

#### Get Pool by Token ID
```graphql
query {
  poolByToken(token_id: "token-abc-123") {
    pool {
      pool_id
      token_liquidity
      base_liquidity
      initial_ratio
      tvl
    }
    is_active
    age_seconds
  }
}
```

#### Check if Token Has Graduated
```graphql
query {
  hasGraduated(token_id: "token-xyz-456")
}
```

#### Get Top Pools by TVL
```graphql
query {
  topPoolsByTvl(limit: 10) {
    pool_id
    token_id
    tvl
    token_liquidity
    base_liquidity
  }
}
```

#### Get Recently Created Pools
```graphql
query {
  recentPools(limit: 10) {
    pool_id
    token_id
    created_at
    tvl
  }
}
```

#### Get Locked Liquidity Summary
```graphql
query {
  lockedLiquiditySummary {
    total_locked_pools
    total_locked_tvl
    permanently_locked_pools
    temporarily_locked_pools
  }
}
```

## Security Features

### Input Validation
- Rejects zero token supply
- Rejects zero base currency raised
- Validates token_id format and uniqueness

### Idempotency
- Duplicate graduation requests are handled safely
- No state corruption from message replays
- Returns existing pool if already graduated

### Anti-Rug Protection
- All liquidity is permanently locked
- No withdrawal mechanisms
- No admin privileges
- Immutable pool parameters

### Error Handling
- Comprehensive error logging
- Graceful failure modes
- No panic on invalid inputs (logged and skipped)

## Deployment

### Prerequisites
- Linera SDK 0.13+
- fair-launch-abi package
- Rust 1.75+

### Build
```bash
cd contracts/swap
cargo build --release --target wasm32-unknown-unknown
```

### Deploy
```bash
linera publish-and-create \
  --contract target/wasm32-unknown-unknown/release/fair_launch_swap.wasm \
  --service target/wasm32-unknown-unknown/release/fair_launch_swap.wasm
```

### Configuration

The swap contract requires no configuration. It automatically:
- Initializes on first instantiation
- Listens for `GraduateToken` messages
- Creates pools with permanent locks
- Responds with `PoolCreated` confirmations

## Testing

### Run Unit Tests
```bash
cargo test
```

### Run Integration Tests
```bash
cargo test --test integration_tests
```

### Test Coverage
- Pool creation validation
- Duplicate graduation handling
- TVL calculation accuracy
- Pagination correctness
- Edge cases (min/max liquidity)
- Concurrent query handling

## Example Usage

### From Token Contract (on graduation)
```rust
// Token contract sends graduation message
runtime
    .prepare_message(Message::GraduateToken {
        token_id: "token-abc-123".to_string(),
        total_supply: U256::from(1_000_000_000),
        total_raised: U256::from(69_000),
    })
    .with_tracking()
    .send_to(swap_chain_id);
```

### From Frontend (query pools)
```typescript
const response = await fetch('/graphql', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: `
      query {
        pools(limit: 10) {
          pool_id
          token_id
          tvl
          is_locked
        }
      }
    `
  })
});

const data = await response.json();
console.log(data.data.pools);
```

## Error Handling

The contract handles these error scenarios:
- **Invalid supply/raised**: Logs error, skips pool creation
- **Duplicate graduation**: Returns existing pool, logs event
- **Message authentication failure**: Logs warning (handled by Linera)
- **State corruption**: Panics with detailed error (should never happen)

## Performance Characteristics

- **Pool Creation**: O(log n) - MapView insertion
- **Pool Lookup by ID**: O(log n) - MapView get
- **Pool Lookup by Token**: O(log n) - Two MapView gets
- **Pool Listing**: O(n × limit) - Iterator with pagination
- **TVL Calculation**: O(1) - Cached in state

## Limitations

- No token swapping (pools are locked, view-only)
- No liquidity provision/withdrawal (permanent lock)
- No pool parameter updates (immutable)
- Pagination requires iteration (no indexed sorting)

## Future Enhancements

Potential improvements (not in current scope):
- Indexed views for efficient sorting by TVL/age
- Historical pool snapshots
- On-chain price oracles
- Event emission for pool creation
- Cross-chain pool registry aggregation

## License

MIT

## Support

For issues or questions:
- Open GitHub issue
- Contact Fair Launch team
- Review Linera documentation: https://linera.dev
