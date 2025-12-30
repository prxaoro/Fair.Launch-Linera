# Factory Contract Deployment Guide

Complete guide for deploying the Fair Launch Factory contract to Linera blockchain.

## Prerequisites

### System Requirements
- Rust 1.75+ with wasm32-unknown-unknown target
- Linera CLI 0.15.8 or later
- Node.js 18+ (for frontend integration)
- Git

### Install Rust and WASM Target
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install binaryen for WASM optimization (optional but recommended)
# macOS
brew install binaryen

# Ubuntu/Debian
sudo apt-get install binaryen

# Windows
# Download from https://github.com/WebAssembly/binaryen/releases
```

### Install Linera CLI
```bash
# Download and install Linera CLI
cargo install linera-client --version 0.15.8

# Verify installation
linera --version
# Should output: linera 0.15.8 or later
```

## Pre-Deployment Steps

### 1. Build the Contract

```bash
cd ./contracts/factory

# Run tests first
./test.sh

# Build for production
./build.sh
```

Expected output:
```
Building Fair Launch Factory Contract...
========================================
Checking prerequisites...
Building for wasm32-unknown-unknown target...
   Compiling fair-launch-factory v0.1.0
    Finished release [optimized] target(s) in 45.32s
Build successful!
Optimizing WASM binary...
Optimization complete!
WASM binary size: 234K
Location: target/wasm32-unknown-unknown/release/fair_launch_factory.wasm

Build complete!
To deploy: linera publish-bytecode target/wasm32-unknown-unknown/release/fair_launch_factory.wasm
```

### 2. Verify Build Artifacts

```bash
# Check WASM file exists
ls -lh target/wasm32-unknown-unknown/release/fair_launch_factory.wasm

# Verify WASM is valid
wasm-validate target/wasm32-unknown-unknown/release/fair_launch_factory.wasm
```

## Deployment to Linera

### 1. Set Up Linera Wallet

```bash
# Initialize Linera wallet (if not already done)
linera wallet init

# Check wallet status
linera wallet show

# Get your default chain
export CHAIN_ID=$(linera wallet show | grep "Default chain" | awk '{print $3}')
echo "Default chain: $CHAIN_ID"
```

### 2. Publish Contract Bytecode

```bash
# Publish the factory contract bytecode
linera publish-bytecode \
  target/wasm32-unknown-unknown/release/fair_launch_factory.wasm \
  --wait

# Save the bytecode ID (output will look like: Bytecode ID: e123456...)
export FACTORY_BYTECODE_ID="<bytecode-id-from-output>"
echo "Factory Bytecode ID: $FACTORY_BYTECODE_ID"
```

### 3. Create Factory Application

```bash
# Create the factory application on your chain
linera create-application \
  $FACTORY_BYTECODE_ID \
  --wait

# Save the application ID (output will look like: Application ID: e123456...)
export FACTORY_APP_ID="<application-id-from-output>"
echo "Factory Application ID: $FACTORY_APP_ID"
```

### 4. Verify Deployment

```bash
# Check application is running
linera query-applications

# Should show your factory application
```

### 5. Query Factory GraphQL Endpoint

```bash
# Get your node's GraphQL endpoint
export GRAPHQL_ENDPOINT="http://localhost:8080/chains/$CHAIN_ID/applications/$FACTORY_APP_ID"

# Test with a simple query
curl -X POST $GRAPHQL_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { stats { totalTokens graduatedCount activeCount } }"
  }'

# Expected response:
# {
#   "data": {
#     "stats": {
#       "totalTokens": 0,
#       "graduatedCount": 0,
#       "activeCount": 0
#     }
#   }
# }
```

## Multi-Chain Deployment

### 1. Deploy to Multiple Chains

```bash
# Create factory on additional chains
for CHAIN in chain1 chain2 chain3; do
  linera create-application \
    $FACTORY_BYTECODE_ID \
    --target-chain $CHAIN \
    --wait
done
```

### 2. Set Up Load Balancing

```nginx
# nginx.conf
upstream factory_backends {
    server chain1.linera.io:8080;
    server chain2.linera.io:8080;
    server chain3.linera.io:8080;
}

server {
    listen 443 ssl;
    server_name factory.fairlaunch.io;

    location /graphql {
        proxy_pass http://factory_backends;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Post-Deployment Configuration

### 1. Configure Environment Variables

Create `.env` file for your application:
```env
# Factory Contract
FACTORY_APP_ID=e123456...
FACTORY_CHAIN_ID=e789012...
FACTORY_GRAPHQL_ENDPOINT=https://factory.fairlaunch.io/graphql

# Token Contract (deployed separately)
TOKEN_BYTECODE_ID=e345678...

# Swap Contract (deployed separately)
SWAP_APP_ID=e901234...

# Network
LINERA_RPC=https://rpc.linera.io
LINERA_CHAIN=mainnet
```

### 2. Initialize Monitoring

```bash
# Set up Prometheus metrics scraping
cat > prometheus.yml <<EOF
scrape_configs:
  - job_name: 'factory'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
EOF

# Start Prometheus
prometheus --config.file=prometheus.yml
```

### 3. Set Up Alerting

```yaml
# alertmanager.yml
route:
  group_by: ['alertname']
  receiver: 'team-notifications'

receivers:
  - name: 'team-notifications'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#fair-launch-alerts'

# Define alerts
groups:
  - name: factory_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(factory_errors_total[5m]) > 0.05
        annotations:
          summary: "High error rate in factory contract"

      - alert: ChainCreationFailure
        expr: factory_chain_creation_failures_total > 0
        annotations:
          summary: "Token chain creation failed"
```

## Testing Deployment

### 1. Create Test Token

```bash
# Create a test token via CLI
linera execute-operation \
  --application $FACTORY_APP_ID \
  --operation '{
    "CreateToken": {
      "metadata": {
        "name": "Test Token",
        "symbol": "TEST",
        "description": "Testing factory deployment",
        "image_url": "https://example.com/test.png",
        "twitter": null,
        "telegram": null,
        "website": null
      },
      "curve_config": null
    }
  }' \
  --wait
```

### 2. Verify Token Creation

```bash
# Query factory for new token
curl -X POST $GRAPHQL_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { recentTokens(limit: 1) { tokenId metadata { name symbol } } }"
  }'

# Expected response:
# {
#   "data": {
#     "recentTokens": [
#       {
#         "tokenId": "e567890...",
#         "metadata": {
#           "name": "Test Token",
#           "symbol": "TEST"
#         }
#       }
#     ]
#   }
# }
```

### 3. Integration Test Script

```bash
#!/bin/bash
# integration-test.sh

set -e

echo "Running integration tests..."

# Test 1: Create token
echo "Test 1: Creating token..."
RESULT=$(linera execute-operation \
  --application $FACTORY_APP_ID \
  --operation '{
    "CreateToken": {
      "metadata": {
        "name": "Integration Test",
        "symbol": "ITEST",
        "description": "Testing",
        "image_url": null,
        "twitter": null,
        "telegram": null,
        "website": null
      },
      "curve_config": null
    }
  }' \
  --wait)

echo "✓ Token created"

# Test 2: Query token
echo "Test 2: Querying tokens..."
QUERY_RESULT=$(curl -s -X POST $GRAPHQL_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{"query": "query { stats { totalTokens } }"}')

TOKEN_COUNT=$(echo $QUERY_RESULT | jq -r '.data.stats.totalTokens')

if [ "$TOKEN_COUNT" -gt "0" ]; then
  echo "✓ Query successful (Total tokens: $TOKEN_COUNT)"
else
  echo "✗ Query failed"
  exit 1
fi

# Test 3: Search tokens
echo "Test 3: Searching tokens..."
SEARCH_RESULT=$(curl -s -X POST $GRAPHQL_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{"query": "query { searchTokens(query: \"Integration\") { tokenId } }"}')

SEARCH_COUNT=$(echo $SEARCH_RESULT | jq -r '.data.searchTokens | length')

if [ "$SEARCH_COUNT" -gt "0" ]; then
  echo "✓ Search successful (Found: $SEARCH_COUNT)"
else
  echo "✗ Search failed"
  exit 1
fi

echo "All integration tests passed!"
```

## Production Deployment Checklist

- [ ] Run full test suite (`./test.sh`)
- [ ] Build optimized WASM (`./build.sh`)
- [ ] Verify WASM binary size (< 500KB recommended)
- [ ] Deploy to testnet first
- [ ] Run integration tests
- [ ] Set up monitoring and alerting
- [ ] Configure backup nodes
- [ ] Set up load balancing
- [ ] Document application IDs and endpoints
- [ ] Create deployment rollback plan
- [ ] Deploy to mainnet
- [ ] Verify mainnet deployment
- [ ] Update DNS records
- [ ] Notify users

## Upgrading

### Contract Upgrade Process

```bash
# 1. Build new version
./build.sh

# 2. Publish new bytecode
NEW_BYTECODE_ID=$(linera publish-bytecode \
  target/wasm32-unknown-unknown/release/fair_launch_factory.wasm \
  --wait | grep "Bytecode ID" | awk '{print $3}')

# 3. Test on testnet first
linera create-application $NEW_BYTECODE_ID --target-chain testnet --wait

# 4. If successful, upgrade mainnet application
linera upgrade-application \
  --application $FACTORY_APP_ID \
  --new-bytecode $NEW_BYTECODE_ID \
  --wait
```

### State Migration

If state schema changes:
```rust
// Add migration logic in contract.rs
impl Contract for FactoryContract {
    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let mut state = FactoryState::load(runtime.view_storage_context())
            .await
            .expect("Failed to load state");

        // Check version and migrate if needed
        if needs_migration(&state) {
            migrate_state(&mut state).await;
        }

        FactoryContract { state, runtime }
    }
}
```

## Troubleshooting

### Build Failures

**Problem**: `error: linking with 'rust-lld' failed`
```bash
# Solution: Clean and rebuild
cargo clean
./build.sh
```

**Problem**: WASM binary too large
```bash
# Solution: Ensure optimization is enabled
wasm-opt -Oz input.wasm -o output.wasm
```

### Deployment Failures

**Problem**: `Error: Insufficient balance`
```bash
# Solution: Add funds to your chain
linera transfer 1000000 --target-chain $CHAIN_ID
```

**Problem**: `Error: Bytecode already exists`
```bash
# Solution: This is fine, reuse the existing bytecode ID
```

### Runtime Errors

**Problem**: Chain creation fails
```bash
# Check logs
linera node logs --follow

# Verify chain ownership
linera wallet show
```

**Problem**: GraphQL queries timeout
```bash
# Increase timeout
curl --max-time 30 -X POST $GRAPHQL_ENDPOINT ...

# Check node health
linera node status
```

## Security Considerations

1. **Access Control**: Factory is permissionless - anyone can create tokens
2. **Rate Limiting**: Consider implementing rate limits in frontend
3. **Metadata Validation**: All validation happens on-chain
4. **Chain Ownership**: Token creators own their token chains
5. **Message Tracking**: All cross-chain messages use `.with_tracking()`

## Support

- Documentation: https://docs.linera.io
- Discord: https://discord.gg/linera
- GitHub: https://github.com/linera-io/linera-protocol

## License

MIT
