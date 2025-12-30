# Quick Start Guide - Fair Launch Swap Contract

Get the swap contract running in 5 minutes.

## 1. Build the Contract

```bash
cd ./contracts/swap

# Run tests
cargo test

# Build for production
cargo build --release --target wasm32-unknown-unknown
```

## 2. Deploy to Linera

```bash
# Publish bytecode
linera publish-bytecode \
  target/wasm32-unknown-unknown/release/fair_launch_swap.wasm \
  target/wasm32-unknown-unknown/release/fair_launch_swap.wasm

# Create application
linera create-application <BYTECODE_ID>
```

## 3. Query the GraphQL API

```bash
# Get stats
curl -X POST http://localhost:8080/chains/<CHAIN_ID>/applications/<APP_ID> \
  -H "Content-Type: application/json" \
  -d '{"query": "{ stats { total_pools total_tvl } }"}'

# List pools
curl -X POST http://localhost:8080/chains/<CHAIN_ID>/applications/<APP_ID> \
  -H "Content-Type: application/json" \
  -d '{"query": "{ pools(limit: 10) { pool_id token_id tvl } }"}'
```

## 4. Integrate with Token Contract

In your token contract:

```rust
use fair_launch_abi::Message;

// On graduation
runtime
    .prepare_message(Message::GraduateToken {
        token_id: "token-123".to_string(),
        total_supply: U256::from(1_000_000_000),
        total_raised: U256::from(69_000),
    })
    .with_tracking()
    .send_to(swap_chain_id);
```

## 5. Frontend Integration

```typescript
// Install dependencies
npm install @apollo/client graphql

// Create client
import { ApolloClient, InMemoryCache, gql } from '@apollo/client';

const client = new ApolloClient({
  uri: 'http://localhost:8080/chains/<CHAIN_ID>/applications/<APP_ID>',
  cache: new InMemoryCache(),
});

// Query pool
const { data } = await client.query({
  query: gql`
    query {
      poolByToken(token_id: "token-123") {
        pool {
          pool_id
          tvl
          is_locked
        }
      }
    }
  `
});

console.log(data.poolByToken.pool);
```

## Common Queries

### Check if Token Graduated
```graphql
query {
  hasGraduated(token_id: "token-123")
}
```

### Get Pool Details
```graphql
query {
  poolByToken(token_id: "token-123") {
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
}
```

### Get Top Pools
```graphql
query {
  topPoolsByTvl(limit: 5) {
    pool_id
    token_id
    tvl
  }
}
```

### Get Platform Stats
```graphql
query {
  stats {
    total_pools
    total_tvl
    average_pool_tvl
  }
}
```

## Testing

Run all tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test test_pool_creation
```

Run with output:
```bash
cargo test -- --nocapture
```

## Troubleshooting

### Contract won't compile
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

### Tests failing
```bash
# Check Rust version (need 1.75+)
rustc --version

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

### GraphQL queries timeout
```typescript
// Use smaller page sizes
pools(limit: 10)  // Not 100

// Add timeout
const timeout = 5000; // 5 seconds
```

## Next Steps

- Read [README.md](./README.md) for architecture details
- Check [EXAMPLES.md](./EXAMPLES.md) for integration patterns
- Review [INTEGRATION.md](./INTEGRATION.md) for deployment guide
- See [SUMMARY.md](./SUMMARY.md) for implementation overview

## Support

- GitHub Issues: https://github.com/your-org/fair-launch-linera
- Linera Docs: https://linera.dev
- Discord: https://discord.gg/linera
