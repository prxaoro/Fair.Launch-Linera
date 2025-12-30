# Swap Contract Examples

Practical examples for integrating with the Fair Launch swap contract.

## Table of Contents
1. [Basic Pool Queries](#basic-pool-queries)
2. [Advanced Queries](#advanced-queries)
3. [Integration Patterns](#integration-patterns)
4. [TypeScript SDK](#typescript-sdk)
5. [Error Handling](#error-handling)

## Basic Pool Queries

### Get All Pools
```graphql
query GetAllPools {
  pools(offset: 0, limit: 20) {
    pool_id
    token_id
    token_liquidity
    base_liquidity
    initial_ratio
    tvl
    is_locked
    lock_expires_at
    created_at
    trade_count
  }
}
```

### Get Specific Pool
```graphql
query GetPool($poolId: String!) {
  pool(pool_id: $poolId) {
    pool {
      pool_id
      token_id
      token_liquidity
      base_liquidity
      initial_ratio
      tvl
      is_locked
      created_at
    }
    is_active
    age_seconds
  }
}
```

Variables:
```json
{
  "poolId": "pool-token-abc-123"
}
```

### Check Token Graduation Status
```graphql
query CheckGraduation($tokenId: String!) {
  hasGraduated(token_id: $tokenId)
  poolByToken(token_id: $tokenId) {
    pool {
      pool_id
      tvl
      created_at
    }
  }
}
```

Variables:
```json
{
  "tokenId": "token-xyz-456"
}
```

## Advanced Queries

### Get Top Pools with Stats
```graphql
query TopPoolsWithStats {
  stats {
    total_pools
    total_tvl
    average_pool_tvl
  }
  topPoolsByTvl(limit: 5) {
    pool_id
    token_id
    tvl
    token_liquidity
    base_liquidity
    initial_ratio
    created_at
  }
}
```

### Get Recent Activity
```graphql
query RecentActivity {
  recentPools(limit: 10) {
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
```

### Get Locked Liquidity Overview
```graphql
query LockedLiquidityOverview {
  lockedLiquiditySummary {
    total_locked_pools
    total_locked_tvl
    permanently_locked_pools
    temporarily_locked_pools
  }
  topPoolsByTvl(limit: 3) {
    pool_id
    token_id
    tvl
  }
}
```

### Paginated Pool List
```graphql
query PaginatedPools($offset: Int!, $limit: Int!) {
  pools(offset: $offset, limit: $limit) {
    pool_id
    token_id
    tvl
    created_at
  }
  stats {
    total_pools
  }
}
```

Variables:
```json
{
  "offset": 0,
  "limit": 10
}
```

## Integration Patterns

### React Hook for Pool Data
```typescript
import { useState, useEffect } from 'react';

interface Pool {
  pool_id: string;
  token_id: string;
  token_liquidity: string;
  base_liquidity: string;
  tvl: string;
  is_locked: boolean;
  created_at: number;
}

function usePool(poolId: string) {
  const [pool, setPool] = useState<Pool | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchPool = async () => {
      try {
        setLoading(true);
        const response = await fetch('/graphql', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            query: `
              query GetPool($poolId: String!) {
                pool(pool_id: $poolId) {
                  pool {
                    pool_id
                    token_id
                    token_liquidity
                    base_liquidity
                    tvl
                    is_locked
                    created_at
                  }
                }
              }
            `,
            variables: { poolId }
          })
        });

        const data = await response.json();
        if (data.errors) {
          throw new Error(data.errors[0].message);
        }

        setPool(data.data.pool?.pool || null);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    fetchPool();
  }, [poolId]);

  return { pool, loading, error };
}

// Usage
function PoolDetails({ poolId }: { poolId: string }) {
  const { pool, loading, error } = usePool(poolId);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!pool) return <div>Pool not found</div>;

  return (
    <div>
      <h2>Pool: {pool.pool_id}</h2>
      <p>Token: {pool.token_id}</p>
      <p>TVL: {pool.tvl}</p>
      <p>Locked: {pool.is_locked ? 'Yes' : 'No'}</p>
    </div>
  );
}
```

### React Hook for Pool List
```typescript
function usePools(offset = 0, limit = 20) {
  const [pools, setPools] = useState<Pool[]>([]);
  const [totalPools, setTotalPools] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchPools = async () => {
      try {
        setLoading(true);
        const response = await fetch('/graphql', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            query: `
              query GetPools($offset: Int!, $limit: Int!) {
                pools(offset: $offset, limit: $limit) {
                  pool_id
                  token_id
                  tvl
                  is_locked
                  created_at
                }
                stats {
                  total_pools
                }
              }
            `,
            variables: { offset, limit }
          })
        });

        const data = await response.json();
        if (data.errors) {
          throw new Error(data.errors[0].message);
        }

        setPools(data.data.pools || []);
        setTotalPools(data.data.stats.total_pools);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    fetchPools();
  }, [offset, limit]);

  return { pools, totalPools, loading, error };
}

// Usage with pagination
function PoolList() {
  const [page, setPage] = useState(0);
  const limit = 10;
  const { pools, totalPools, loading, error } = usePools(page * limit, limit);

  const totalPages = Math.ceil(totalPools / limit);

  if (loading) return <div>Loading pools...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      <h2>Pools ({totalPools} total)</h2>
      {pools.map(pool => (
        <div key={pool.pool_id}>
          <h3>{pool.token_id}</h3>
          <p>TVL: {pool.tvl}</p>
          <p>Status: {pool.is_locked ? 'Locked' : 'Active'}</p>
        </div>
      ))}

      <div>
        <button
          onClick={() => setPage(p => Math.max(0, p - 1))}
          disabled={page === 0}
        >
          Previous
        </button>
        <span>Page {page + 1} of {totalPages}</span>
        <button
          onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
          disabled={page >= totalPages - 1}
        >
          Next
        </button>
      </div>
    </div>
  );
}
```

## TypeScript SDK

### Client Class
```typescript
import { U256 } from 'primitive-types';

interface SwapStats {
  total_pools: number;
  total_tvl: string;
  average_pool_tvl: string;
}

interface PoolInfo {
  pool_id: string;
  token_id: string;
  token_liquidity: string;
  base_liquidity: string;
  initial_ratio: string;
  tvl: string;
  is_locked: boolean;
  lock_expires_at: number | null;
  created_at: number;
  trade_count: number;
}

interface PoolDetails {
  pool: PoolInfo;
  is_active: boolean;
  age_seconds: number;
}

class SwapClient {
  constructor(private endpoint: string) {}

  private async query<T>(query: string, variables?: any): Promise<T> {
    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, variables })
    });

    const data = await response.json();
    if (data.errors) {
      throw new Error(data.errors[0].message);
    }

    return data.data;
  }

  async getStats(): Promise<SwapStats> {
    const data = await this.query<{ stats: SwapStats }>(`
      query {
        stats {
          total_pools
          total_tvl
          average_pool_tvl
        }
      }
    `);
    return data.stats;
  }

  async getPools(offset = 0, limit = 20): Promise<PoolInfo[]> {
    const data = await this.query<{ pools: PoolInfo[] }>(`
      query GetPools($offset: Int!, $limit: Int!) {
        pools(offset: $offset, limit: $limit) {
          pool_id
          token_id
          token_liquidity
          base_liquidity
          initial_ratio
          tvl
          is_locked
          lock_expires_at
          created_at
          trade_count
        }
      }
    `, { offset, limit });
    return data.pools;
  }

  async getPool(poolId: string): Promise<PoolDetails | null> {
    const data = await this.query<{ pool: PoolDetails | null }>(`
      query GetPool($poolId: String!) {
        pool(pool_id: $poolId) {
          pool {
            pool_id
            token_id
            token_liquidity
            base_liquidity
            initial_ratio
            tvl
            is_locked
            created_at
          }
          is_active
          age_seconds
        }
      }
    `, { poolId });
    return data.pool;
  }

  async getPoolByToken(tokenId: string): Promise<PoolDetails | null> {
    const data = await this.query<{ poolByToken: PoolDetails | null }>(`
      query GetPoolByToken($tokenId: String!) {
        poolByToken(token_id: $tokenId) {
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
    `, { tokenId });
    return data.poolByToken;
  }

  async hasGraduated(tokenId: string): Promise<boolean> {
    const data = await this.query<{ hasGraduated: boolean }>(`
      query CheckGraduation($tokenId: String!) {
        hasGraduated(token_id: $tokenId)
      }
    `, { tokenId });
    return data.hasGraduated;
  }

  async getTopPoolsByTvl(limit = 10): Promise<PoolInfo[]> {
    const data = await this.query<{ topPoolsByTvl: PoolInfo[] }>(`
      query GetTopPools($limit: Int!) {
        topPoolsByTvl(limit: $limit) {
          pool_id
          token_id
          tvl
          token_liquidity
          base_liquidity
          created_at
        }
      }
    `, { limit });
    return data.topPoolsByTvl;
  }

  async getRecentPools(limit = 10): Promise<PoolInfo[]> {
    const data = await this.query<{ recentPools: PoolInfo[] }>(`
      query GetRecentPools($limit: Int!) {
        recentPools(limit: $limit) {
          pool_id
          token_id
          tvl
          created_at
        }
      }
    `, { limit });
    return data.recentPools;
  }
}

// Usage
const swapClient = new SwapClient('http://localhost:8080/graphql');

// Get stats
const stats = await swapClient.getStats();
console.log(`Total Pools: ${stats.total_pools}`);
console.log(`Total TVL: ${stats.total_tvl}`);

// Get top pools
const topPools = await swapClient.getTopPoolsByTvl(5);
topPools.forEach(pool => {
  console.log(`${pool.token_id}: ${pool.tvl} TVL`);
});

// Check if token graduated
const hasGraduated = await swapClient.hasGraduated('token-abc-123');
if (hasGraduated) {
  const poolDetails = await swapClient.getPoolByToken('token-abc-123');
  console.log(`Pool ID: ${poolDetails.pool.pool_id}`);
  console.log(`TVL: ${poolDetails.pool.tvl}`);
  console.log(`Age: ${poolDetails.age_seconds} seconds`);
}
```

## Error Handling

### Handling GraphQL Errors
```typescript
async function safeQuery<T>(
  query: string,
  variables?: any
): Promise<{ data?: T; error?: string }> {
  try {
    const response = await fetch('/graphql', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, variables })
    });

    if (!response.ok) {
      return {
        error: `HTTP ${response.status}: ${response.statusText}`
      };
    }

    const result = await response.json();

    if (result.errors && result.errors.length > 0) {
      return {
        error: result.errors.map(e => e.message).join(', ')
      };
    }

    return { data: result.data };
  } catch (err) {
    return {
      error: err instanceof Error ? err.message : 'Unknown error'
    };
  }
}

// Usage
const { data, error } = await safeQuery<{ pool: PoolDetails }>(
  `query GetPool($poolId: String!) {
    pool(pool_id: $poolId) {
      pool { pool_id tvl }
    }
  }`,
  { poolId: 'pool-xyz' }
);

if (error) {
  console.error('Query failed:', error);
} else {
  console.log('Pool data:', data?.pool);
}
```

### Retry Logic
```typescript
async function queryWithRetry<T>(
  queryFn: () => Promise<T>,
  maxRetries = 3,
  delayMs = 1000
): Promise<T> {
  let lastError: Error;

  for (let i = 0; i < maxRetries; i++) {
    try {
      return await queryFn();
    } catch (err) {
      lastError = err as Error;
      console.warn(`Attempt ${i + 1} failed:`, err);

      if (i < maxRetries - 1) {
        await new Promise(resolve =>
          setTimeout(resolve, delayMs * (i + 1))
        );
      }
    }
  }

  throw lastError!;
}

// Usage
const pool = await queryWithRetry(
  () => swapClient.getPool('pool-abc-123'),
  3,
  1000
);
```

## Best Practices

1. **Use Pagination**: Always paginate large result sets
   ```typescript
   const pools = await swapClient.getPools(0, 20); // First 20
   ```

2. **Cache Results**: Cache pool data to reduce queries
   ```typescript
   const cache = new Map<string, PoolInfo>();

   async function getCachedPool(poolId: string): Promise<PoolInfo> {
     if (cache.has(poolId)) {
       return cache.get(poolId)!;
     }

     const details = await swapClient.getPool(poolId);
     if (details) {
       cache.set(poolId, details.pool);
       return details.pool;
     }

     throw new Error('Pool not found');
   }
   ```

3. **Handle Errors Gracefully**: Always handle query failures
   ```typescript
   try {
     const pool = await swapClient.getPool(poolId);
     if (!pool) {
       // Handle not found
     }
   } catch (err) {
     // Handle error
   }
   ```

4. **Use TypeScript**: Type-safe queries prevent runtime errors
   ```typescript
   interface PoolResponse {
     pool: PoolDetails | null;
   }

   const data = await query<PoolResponse>(/* ... */);
   ```

5. **Monitor Rate Limits**: Implement request throttling
   ```typescript
   class RateLimitedClient {
     private lastRequest = 0;
     private minInterval = 100; // ms

     async query<T>(fn: () => Promise<T>): Promise<T> {
       const now = Date.now();
       const wait = Math.max(0, this.minInterval - (now - this.lastRequest));

       if (wait > 0) {
         await new Promise(resolve => setTimeout(resolve, wait));
       }

       this.lastRequest = Date.now();
       return fn();
     }
   }
   ```
