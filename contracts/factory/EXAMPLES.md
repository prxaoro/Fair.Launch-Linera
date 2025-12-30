# Factory Contract Examples

Complete examples for interacting with the Fair Launch Factory contract.

## Table of Contents
- [Creating Tokens](#creating-tokens)
- [GraphQL Queries](#graphql-queries)
- [Integration Examples](#integration-examples)
- [Error Handling](#error-handling)

## Creating Tokens

### Basic Token Creation

```rust
use fair_launch_abi::{FactoryOperation, TokenMetadata, BondingCurveConfig};
use primitive_types::U256;

// Create a simple token with default bonding curve
let operation = FactoryOperation::CreateToken {
    metadata: TokenMetadata {
        name: "Pepe Coin".to_string(),
        symbol: "PEPE".to_string(),
        description: "The dankest meme coin on Linera".to_string(),
        image_url: Some("https://example.com/pepe.png".to_string()),
        twitter: Some("@pepecoin".to_string()),
        telegram: Some("@pepecoin".to_string()),
        website: Some("https://pepecoin.com".to_string()),
    },
    curve_config: None,  // Uses default configuration
};
```

### Advanced Token with Custom Bonding Curve

```rust
// Create token with custom bonding curve parameters
let operation = FactoryOperation::CreateToken {
    metadata: TokenMetadata {
        name: "Wojak Token".to_string(),
        symbol: "WOJAK".to_string(),
        description: "For all the feels".to_string(),
        image_url: Some("ipfs://QmWojakHash".to_string()),
        twitter: Some("@wojaktoken".to_string()),
        telegram: None,
        website: None,
    },
    curve_config: Some(BondingCurveConfig {
        k: U256::from(2000),              // Higher price multiplier
        scale: U256::from(1_000_000),      // 1M tokens scale
        target_raise: U256::from(100_000), // Target 100k raise
        max_supply: U256::from(500_000_000u64), // 500M max supply
    }),
};
```

### Minimal Token (Testing)

```rust
// Minimal viable token for testing
let operation = FactoryOperation::CreateToken {
    metadata: TokenMetadata {
        name: "Test".to_string(),
        symbol: "TST".to_string(),
        description: "Test token".to_string(),
        image_url: None,
        twitter: None,
        telegram: None,
        website: None,
    },
    curve_config: None,
};
```

## GraphQL Queries

### Query: List All Tokens

```graphql
query ListTokens($offset: Int, $limit: Int) {
  tokens(offset: $offset, limit: $limit) {
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

Variables:
```json
{
  "offset": 0,
  "limit": 20
}
```

### Query: Get Single Token

```graphql
query GetToken($tokenId: String!) {
  token(tokenId: $tokenId) {
    tokenId
    creator
    metadata {
      name
      symbol
      description
      imageUrl
    }
    currentSupply
    totalRaised
    isGraduated
    createdAt
    dexPoolId
  }
}
```

Variables:
```json
{
  "tokenId": "e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65"
}
```

### Query: Tokens by Creator

```graphql
query TokensByCreator($creator: String!) {
  tokensByCreator(creator: $creator) {
    tokenId
    metadata {
      name
      symbol
    }
    currentSupply
    totalRaised
    createdAt
  }
}
```

Variables:
```json
{
  "creator": "User(0x123...)"
}
```

### Query: Recent Launches

```graphql
query RecentLaunches($limit: Int) {
  recentTokens(limit: $limit) {
    tokenId
    metadata {
      name
      symbol
      imageUrl
    }
    creator
    createdAt
    currentSupply
    totalRaised
  }
}
```

Variables:
```json
{
  "limit": 10
}
```

### Query: Graduated Tokens

```graphql
query GraduatedTokens($offset: Int, $limit: Int) {
  graduatedTokens(offset: $offset, limit: $limit) {
    tokenId
    metadata {
      name
      symbol
    }
    totalRaised
    dexPoolId
    currentSupply
  }
}
```

Variables:
```json
{
  "offset": 0,
  "limit": 20
}
```

### Query: Search Tokens

```graphql
query SearchTokens($query: String!) {
  searchTokens(query: $query) {
    tokenId
    metadata {
      name
      symbol
      description
      imageUrl
    }
    creator
    currentSupply
  }
}
```

Variables:
```json
{
  "query": "pepe"
}
```

### Query: Factory Statistics

```graphql
query FactoryStats {
  stats {
    totalTokens
    graduatedCount
    activeCount
    totalValueLocked
  }
}
```

### Query: Complete Dashboard Data

```graphql
query Dashboard {
  stats {
    totalTokens
    graduatedCount
    activeCount
    totalValueLocked
  }

  recentTokens(limit: 5) {
    tokenId
    metadata {
      name
      symbol
      imageUrl
    }
    creator
    currentSupply
    totalRaised
    createdAt
  }

  graduatedTokens(limit: 5) {
    tokenId
    metadata {
      name
      symbol
    }
    dexPoolId
    totalRaised
  }
}
```

## Integration Examples

### Frontend Integration (TypeScript)

```typescript
import { GraphQLClient } from 'graphql-request';

// Initialize client
const client = new GraphQLClient('https://your-factory-chain.linera.io/graphql');

// Fetch recent tokens
async function fetchRecentTokens(limit: number = 10) {
  const query = `
    query RecentTokens($limit: Int) {
      recentTokens(limit: $limit) {
        tokenId
        metadata {
          name
          symbol
          imageUrl
        }
        currentSupply
        totalRaised
        createdAt
      }
    }
  `;

  const data = await client.request(query, { limit });
  return data.recentTokens;
}

// Search tokens
async function searchTokens(searchQuery: string) {
  const query = `
    query SearchTokens($query: String!) {
      searchTokens(query: $query) {
        tokenId
        metadata {
          name
          symbol
          description
          imageUrl
        }
        currentSupply
      }
    }
  `;

  const data = await client.request(query, { query: searchQuery });
  return data.searchTokens;
}

// Get factory stats
async function getFactoryStats() {
  const query = `
    query {
      stats {
        totalTokens
        graduatedCount
        activeCount
        totalValueLocked
      }
    }
  `;

  const data = await client.request(query);
  return data.stats;
}

// Usage
const tokens = await fetchRecentTokens(10);
console.log('Recent tokens:', tokens);

const searchResults = await searchTokens('pepe');
console.log('Search results:', searchResults);

const stats = await getFactoryStats();
console.log('Factory stats:', stats);
```

### React Hook Example

```typescript
import { useQuery } from '@tanstack/react-query';
import { GraphQLClient } from 'graphql-request';

const client = new GraphQLClient('https://your-factory-chain.linera.io/graphql');

export function useRecentTokens(limit: number = 10) {
  return useQuery({
    queryKey: ['recentTokens', limit],
    queryFn: async () => {
      const query = `
        query RecentTokens($limit: Int) {
          recentTokens(limit: $limit) {
            tokenId
            metadata {
              name
              symbol
              imageUrl
            }
            currentSupply
            totalRaised
          }
        }
      `;
      const data = await client.request(query, { limit });
      return data.recentTokens;
    },
  });
}

export function useToken(tokenId: string) {
  return useQuery({
    queryKey: ['token', tokenId],
    queryFn: async () => {
      const query = `
        query GetToken($tokenId: String!) {
          token(tokenId: $tokenId) {
            tokenId
            metadata {
              name
              symbol
              description
              imageUrl
            }
            currentSupply
            totalRaised
            isGraduated
            dexPoolId
          }
        }
      `;
      const data = await client.request(query, { tokenId });
      return data.token;
    },
    enabled: !!tokenId,
  });
}

// Component usage
function TokenList() {
  const { data: tokens, isLoading, error } = useRecentTokens(10);

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading tokens</div>;

  return (
    <div>
      {tokens.map((token) => (
        <div key={token.tokenId}>
          <h3>{token.metadata.name} ({token.metadata.symbol})</h3>
          <p>Supply: {token.currentSupply}</p>
          <p>Raised: {token.totalRaised}</p>
        </div>
      ))}
    </div>
  );
}
```

### Python Integration

```python
import requests
import json

class FactoryClient:
    def __init__(self, endpoint: str):
        self.endpoint = endpoint

    def query(self, query: str, variables: dict = None):
        """Execute GraphQL query"""
        payload = {
            'query': query,
            'variables': variables or {}
        }

        response = requests.post(
            self.endpoint,
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
        response.raise_for_status()

        data = response.json()
        if 'errors' in data:
            raise Exception(f"GraphQL errors: {data['errors']}")

        return data['data']

    def get_recent_tokens(self, limit: int = 10):
        """Get recent token launches"""
        query = """
        query RecentTokens($limit: Int) {
          recentTokens(limit: $limit) {
            tokenId
            metadata {
              name
              symbol
            }
            currentSupply
            totalRaised
          }
        }
        """
        return self.query(query, {'limit': limit})

    def search_tokens(self, search_query: str):
        """Search tokens by name or symbol"""
        query = """
        query SearchTokens($query: String!) {
          searchTokens(query: $query) {
            tokenId
            metadata {
              name
              symbol
              description
            }
          }
        }
        """
        return self.query(query, {'query': search_query})

    def get_stats(self):
        """Get factory statistics"""
        query = """
        query {
          stats {
            totalTokens
            graduatedCount
            activeCount
            totalValueLocked
          }
        }
        """
        return self.query(query)

# Usage
client = FactoryClient('https://your-factory-chain.linera.io/graphql')

# Get recent tokens
recent = client.get_recent_tokens(10)
print(f"Recent tokens: {recent}")

# Search
results = client.search_tokens('pepe')
print(f"Search results: {results}")

# Get stats
stats = client.get_stats()
print(f"Factory stats: {stats}")
```

## Error Handling

### Common Errors and Solutions

#### 1. Invalid Metadata
```rust
// Error: Token name cannot be empty
FactoryOperation::CreateToken {
    metadata: TokenMetadata {
        name: "".to_string(),  // ❌ Empty name
        symbol: "TEST".to_string(),
        // ...
    },
    curve_config: None,
}

// Solution: Provide valid name
metadata.name = "My Token".to_string();  // ✅
```

#### 2. URL Validation Errors
```rust
// Error: Invalid image URL format
metadata.image_url = Some("not-a-url".to_string());  // ❌

// Solution: Use proper URL schemes
metadata.image_url = Some("https://example.com/image.png".to_string());  // ✅
metadata.image_url = Some("ipfs://QmHash...".to_string());  // ✅
```

#### 3. Bonding Curve Validation
```rust
// Error: k parameter must be greater than zero
curve_config.k = U256::zero();  // ❌

// Solution: Use positive values
curve_config.k = U256::from(1000);  // ✅
```

#### 4. GraphQL Query Errors
```typescript
// Error: Token not found
const token = await client.request(`
  query {
    token(tokenId: "invalid-id") {
      tokenId
    }
  }
`);
// Returns: null

// Solution: Check if token exists
if (token === null) {
  console.error('Token not found');
} else {
  console.log('Token:', token);
}
```

### Error Handling Best Practices

```typescript
// Comprehensive error handling
async function createToken(metadata: TokenMetadata) {
  try {
    // Validate metadata client-side first
    if (!metadata.name || metadata.name.trim().length === 0) {
      throw new Error('Token name is required');
    }

    if (!metadata.symbol || metadata.symbol.trim().length === 0) {
      throw new Error('Token symbol is required');
    }

    if (metadata.name.length > 100) {
      throw new Error('Token name too long (max 100 characters)');
    }

    // Execute operation
    const result = await executeOperation(operation);
    return result;

  } catch (error) {
    if (error.message.includes('Unauthorized')) {
      console.error('User not authenticated');
      // Redirect to login
    } else if (error.message.includes('InvalidMetadata')) {
      console.error('Invalid token metadata:', error.message);
      // Show validation errors to user
    } else if (error.message.includes('ChainCreationFailed')) {
      console.error('Failed to create token chain:', error.message);
      // Retry or show error
    } else {
      console.error('Unexpected error:', error);
      // Generic error handling
    }

    throw error;
  }
}
```

## Advanced Usage

### Batch Query Multiple Tokens

```graphql
query BatchTokenData {
  token1: token(tokenId: "chain-id-1") {
    tokenId
    metadata { name symbol }
    currentSupply
  }

  token2: token(tokenId: "chain-id-2") {
    tokenId
    metadata { name symbol }
    currentSupply
  }

  token3: token(tokenId: "chain-id-3") {
    tokenId
    metadata { name symbol }
    currentSupply
  }
}
```

### Pagination Helper (TypeScript)

```typescript
async function getAllTokens(client: GraphQLClient) {
  const allTokens = [];
  let offset = 0;
  const limit = 100;

  while (true) {
    const query = `
      query Tokens($offset: Int, $limit: Int) {
        tokens(offset: $offset, limit: $limit) {
          tokenId
          metadata { name symbol }
        }
      }
    `;

    const data = await client.request(query, { offset, limit });

    if (data.tokens.length === 0) {
      break;
    }

    allTokens.push(...data.tokens);
    offset += limit;
  }

  return allTokens;
}
```

## Testing

### Unit Test Example

```rust
#[tokio::test]
async fn test_create_and_query_token() {
    let state = create_test_state().await;

    let creator = AccountOwner::from(ChainId::root(0));
    let metadata = TokenMetadata {
        name: "Test Token".to_string(),
        symbol: "TEST".to_string(),
        description: "Test".to_string(),
        image_url: None,
        twitter: None,
        telegram: None,
        website: None,
    };

    let curve_config = BondingCurveConfig::default();
    let token_id = "test-token-123".to_string();

    // Create token
    state.register_token(
        token_id.clone(),
        creator,
        metadata.clone(),
        curve_config,
        Timestamp::from(0),
    ).await.unwrap();

    // Query token
    let token = state.get_token(&token_id).await.unwrap();

    assert_eq!(token.metadata.name, "Test Token");
    assert_eq!(token.metadata.symbol, "TEST");
}
```
