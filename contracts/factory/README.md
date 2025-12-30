# Fair Launch Factory Contract

Production-ready factory contract for the Fair Launch platform on Linera blockchain. This contract serves as a registry and launcher for new token microchains.

## Architecture

### Overview
The Factory contract implements the Factory Pattern for spawning new token microchains:
- Each token launch gets its own dedicated microchain for independent scaling
- Factory maintains a comprehensive registry of all created tokens
- Cross-chain messages coordinate token initialization with guaranteed delivery (`.with_tracking()`)
- GraphQL service provides rich querying capabilities

### Key Components

#### 1. FactoryState (`state.rs`)
Manages persistent storage using Linera's view system:
- `tokens: MapView<String, TokenLaunch>` - Main token registry indexed by ChainId
- `token_count: RegisterView<u64>` - Total number of created tokens
- `creator_registry: MapView<AccountOwner, String>` - Creator-to-tokens mapping
- `token_index: MapView<u64, String>` - Sequential index for pagination

#### 2. FactoryContract (`contract.rs`)
Handles operations and messages:
- **Operations**: `CreateToken` - Spawns new token microchain
- **Messages**: Handles graduation notifications, trade updates, pool creation
- **Chain Creation**: Uses `runtime.open_chain()` to spawn token microchains
- **Validation**: Strict metadata and bonding curve validation

#### 3. FactoryService (`service.rs`)
GraphQL service for querying:
- Token lookup by ID
- Pagination support
- Creator filtering
- Recent launches
- Graduated tokens
- Search by name/symbol
- Factory statistics

## Security Features

### Input Validation
All metadata fields are strictly validated:
- Name: Required, 1-100 characters, non-empty after trim
- Symbol: Required, 1-20 characters, non-empty after trim
- Description: Max 1000 characters
- URLs: Must use http://, https://, or ipfs:// schemes
- Bonding curve parameters: All must be non-zero, supply > scale

### Authentication
- All token creation operations require authenticated caller
- Caller becomes the creator and owner of the new token chain
- Application accounts cannot own chains (security constraint)

### Error Handling
Comprehensive error types:
- `FactoryError` - State-level errors (storage, validation)
- `ContractError` - Contract-level errors (auth, chain creation)
- All errors use `thiserror` for proper error chaining
- Logging at appropriate levels (info, warn, error)

### Message Tracking
All cross-chain messages use `.with_tracking()`:
- Guaranteed delivery to token chains
- Automatic retry on failure
- Message ordering preserved

## Usage

### Creating a Token

```rust
// Operation structure
FactoryOperation::CreateToken {
    metadata: TokenMetadata {
        name: "My Token".to_string(),
        symbol: "MTK".to_string(),
        description: "A fair launch token".to_string(),
        image_url: Some("https://example.com/logo.png".to_string()),
        twitter: Some("@mytoken".to_string()),
        telegram: None,
        website: Some("https://mytoken.com".to_string()),
    },
    curve_config: Some(BondingCurveConfig {
        k: U256::from(1000),
        scale: U256::from(1_000_000),
        target_raise: U256::from(69_000),
        max_supply: U256::from(1_000_000_000u64),
    }),
}
```

### GraphQL Queries

#### Get All Tokens (Paginated)
```graphql
query {
  tokens(offset: 0, limit: 20) {
    tokenId
    creator
    metadata {
      name
      symbol
      description
      imageUrl
      twitter
      telegram
      website
    }
    curveConfig {
      k
      scale
      targetRaise
      maxSupply
    }
    currentSupply
    totalRaised
    isGraduated
    createdAt
    dexPoolId
  }
}
```

#### Get Token by ID
```graphql
query {
  token(tokenId: "chain-id-here") {
    tokenId
    metadata {
      name
      symbol
    }
    currentSupply
    totalRaised
    isGraduated
  }
}
```

#### Get Tokens by Creator
```graphql
query {
  tokensByCreator(creator: "User(0x...)") {
    tokenId
    metadata {
      name
      symbol
    }
    createdAt
  }
}
```

#### Get Recent Launches
```graphql
query {
  recentTokens(limit: 10) {
    tokenId
    metadata {
      name
      symbol
    }
    creator
    createdAt
  }
}
```

#### Get Graduated Tokens
```graphql
query {
  graduatedTokens(offset: 0, limit: 20) {
    tokenId
    metadata {
      name
    }
    dexPoolId
    totalRaised
  }
}
```

#### Search Tokens
```graphql
query {
  searchTokens(query: "pepe") {
    tokenId
    metadata {
      name
      symbol
    }
  }
}
```

#### Get Factory Statistics
```graphql
query {
  stats {
    totalTokens
    graduatedCount
    activeCount
    totalValueLocked
  }
}
```

## Cross-Chain Message Flow

### Token Creation
1. User calls `CreateToken` operation on Factory chain
2. Factory validates metadata and bonding curve config
3. Factory calls `runtime.open_chain()` to create new microchain
4. Factory sends `TokenCreated` message to new chain with `.with_tracking()`
5. Token chain receives message and initializes state
6. Factory records token in registry
7. Token broadcasts `NewLaunch` message

### Trade Notification
1. User trades on Token chain
2. Token updates its state
3. Token sends `TradeExecuted` message to Factory with `.with_tracking()`
4. Factory receives notification (optional analytics tracking)

### Graduation
1. Token bonding curve completes
2. Token sends `GraduateToken` message to Swap chain
3. Token sends notification to Factory
4. Factory updates token status to graduated
5. Swap creates pool and sends `PoolCreated` message
6. Factory updates token with pool ID

## Data Models

### TokenLaunch
```rust
pub struct TokenLaunch {
    pub token_id: String,              // ChainId as string
    pub creator: AccountOwner,         // Token creator
    pub metadata: TokenMetadata,       // Name, symbol, description, URLs
    pub curve_config: BondingCurveConfig,
    pub current_supply: U256,          // Circulating supply
    pub total_raised: U256,            // Total currency raised
    pub is_graduated: bool,            // DEX graduation status
    pub created_at: Timestamp,         // Creation timestamp
    pub dex_pool_id: Option<String>,   // DEX pool if graduated
}
```

### BondingCurveConfig
```rust
pub struct BondingCurveConfig {
    pub k: U256,              // Price constant
    pub scale: U256,          // Supply scale factor
    pub target_raise: U256,   // Target fundraise amount
    pub max_supply: U256,     // Max supply before graduation
}
```

## Error Handling

### State Errors
- `TokenAlreadyExists` - Duplicate token ID
- `TokenNotFound` - Token doesn't exist
- `InvalidMetadata` - Validation failed
- `StorageError` - Underlying storage failure

### Contract Errors
- `StateError` - Wrapped state error
- `ChainCreationFailed` - Failed to spawn microchain
- `Unauthorized` - Caller not authenticated
- `InvalidCurveConfig` - Bonding curve validation failed
- `ViewError` - Storage view error

## Testing

### Unit Tests
Run state and contract unit tests:
```bash
cargo test --package fair-launch-factory
```

### Test Coverage
- State initialization
- Token registration
- Duplicate prevention
- Metadata validation (all fields)
- URL format validation
- Creator registry
- Pagination (including edge cases)
- Token metrics updates
- Graduation status updates
- Complete token lifecycle

### Integration Testing
The `test_token_lifecycle` test simulates a complete token journey:
1. Token creation
2. Initial trading (supply/raised updates)
3. More trading (accumulation phase)
4. Graduation to DEX
5. Creator registry verification

## Performance Considerations

### Indexing
- Token index (`token_index`) enables efficient pagination
- Creator registry uses comma-separated string (trade-off for simplicity)
- For production at scale, consider separate index microchains

### Pagination
- Default limit: 20 tokens per query
- Maximum limit: 100 tokens per query
- Search queries limited to first 1000 tokens (should be indexed)

### Storage Optimization
- Uses `MapView` for efficient key-value lookups
- `RegisterView` for simple values (minimal overhead)
- Token count cached in register (no iteration needed)

## Deployment

### Prerequisites
1. Linera SDK 0.15.8 or later
2. Token contract bytecode deployed
3. Fair Launch ABI package

### Environment Variables
None required - factory uses on-chain configuration only.

### Deployment Steps
```bash
# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Deploy to Linera
linera publish-bytecode \
  target/wasm32-unknown-unknown/release/fair_launch_factory.wasm

# Create factory application
linera create-application <bytecode-id>
```

### Upgrading
Factory contract state is append-only and backwards compatible:
- New tokens added without affecting existing ones
- State migrations not required for schema-compatible updates
- Message handlers support forward compatibility

## Monitoring

### Logging
All critical operations are logged:
- `info`: Successful token creation, graduation
- `warn`: Query failures, invalid input
- `error`: State errors, chain creation failures

### Metrics to Track
- Total tokens created (`token_count`)
- Graduation rate (`graduated_count / total_tokens`)
- Total value locked (sum of `total_raised`)
- Creator activity (tokens per creator)
- Average time to graduation

## Future Enhancements

1. **Advanced Indexing**
   - Separate search microchain for full-text search
   - Category/tag indexing
   - Trending algorithm

2. **Rate Limiting**
   - Per-creator token creation limits
   - Anti-spam measures
   - Creation fees

3. **Analytics**
   - Real-time statistics microchain
   - Historical data tracking
   - Creator leaderboards

4. **Governance**
   - Token curation/flagging
   - Community moderation
   - Featured tokens

## License
SPDX-License-Identifier: MIT

## Contributing
See main repository CONTRIBUTING.md

## Support
- Documentation: https://docs.linera.io
- Discord: https://discord.gg/linera
- GitHub Issues: https://github.com/your-repo/issues
