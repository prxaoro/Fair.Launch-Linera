#!/bin/bash
# Deploy Fair Launch to Conway Testnet
# Requires: linera CLI installed and configured

set -e  # Exit on error

echo "🚀 Fair Launch - Conway Testnet Deployment"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/..)" && pwd)"

echo -e "${YELLOW}📁 Project root: $PROJECT_ROOT${NC}"

# Check if linera is installed
if ! command -v linera &> /dev/null; then
    echo -e "${RED}ERROR: linera CLI not found. Please install it first.${NC}"
    echo "Installation: cargo install linera-sdk --version 0.15.8"
    exit 1
fi

# Step 1: Build all contracts
echo -e "${GREEN}🔨 Building contracts...${NC}"
cd "$PROJECT_ROOT/contracts"

echo "  → Building ABI..."
cd abi && cargo build --release --target wasm32-unknown-unknown
cd ..

echo "  → Building Token contract..."
cd token && cargo build --release --target wasm32-unknown-unknown
cd ..

echo "  → Building Factory contract..."
cd factory && cargo build --release --target wasm32-unknown-unknown
cd ..

echo "  → Building Swap contract..."
cd swap && cargo build --release --target wasm32-unknown-unknown
cd ..

echo -e "${GREEN}✅ All contracts built successfully${NC}"

# Step 2: Configure wallet for Conway testnet
echo -e "${GREEN}🔧 Configuring wallet for Conway testnet...${NC}"

# Check if already configured
if linera wallet show 2>&1 | grep -q "faucet"; then
    echo -e "${YELLOW}Wallet already configured${NC}"
else
    echo "Initializing wallet for Conway testnet..."
    # Get faucet tokens
    linera faucet --help > /dev/null 2>&1 || {
        echo -e "${RED}ERROR: Conway testnet not configured. Please configure manually.${NC}"
        exit 1
    }
fi

# Get default chain
DEFAULT_CHAIN=$(linera wallet show | grep "default" | head -1 | awk '{print $2}')
echo -e "${YELLOW}Default chain: $DEFAULT_CHAIN${NC}"

# Step 3: Publish bytecode to testnet
echo -e "${GREEN}📦 Publishing bytecode to Conway testnet...${NC}"

# Publish Factory
echo "  → Publishing Factory bytecode..."
FACTORY_BYTECODE=$(linera publish-bytecode \
    "$PROJECT_ROOT/contracts/factory/target/wasm32-unknown-unknown/release/factory_contract.wasm" \
    "$PROJECT_ROOT/contracts/factory/target/wasm32-unknown-unknown/release/factory_service.wasm" \
    2>&1 | grep "bytecode ID" | awk '{print $NF}')

if [ -z "$FACTORY_BYTECODE" ]; then
    echo -e "${RED}ERROR: Failed to publish Factory bytecode${NC}"
    exit 1
fi
echo -e "${YELLOW}Factory bytecode: $FACTORY_BYTECODE${NC}"

# Publish Token (for factory to instantiate)
echo "  → Publishing Token bytecode..."
TOKEN_BYTECODE=$(linera publish-bytecode \
    "$PROJECT_ROOT/contracts/token/target/wasm32-unknown-unknown/release/token_contract.wasm" \
    "$PROJECT_ROOT/contracts/token/target/wasm32-unknown-unknown/release/token_service.wasm" \
    2>&1 | grep "bytecode ID" | awk '{print $NF}')

if [ -z "$TOKEN_BYTECODE" ]; then
    echo -e "${RED}ERROR: Failed to publish Token bytecode${NC}"
    exit 1
fi
echo -e "${YELLOW}Token bytecode: $TOKEN_BYTECODE${NC}"

# Publish Swap
echo "  → Publishing Swap bytecode..."
SWAP_BYTECODE=$(linera publish-bytecode \
    "$PROJECT_ROOT/contracts/swap/target/wasm32-unknown-unknown/release/swap_contract.wasm" \
    "$PROJECT_ROOT/contracts/swap/target/wasm32-unknown-unknown/release/swap_service.wasm" \
    2>&1 | grep "bytecode ID" | awk '{print $NF}')

if [ -z "$SWAP_BYTECODE" ]; then
    echo -e "${RED}ERROR: Failed to publish Swap bytecode${NC}"
    exit 1
fi
echo -e "${YELLOW}Swap bytecode: $SWAP_BYTECODE${NC}"

# Step 4: Create applications
echo -e "${GREEN}🚀 Creating applications on Conway testnet...${NC}"

# Create Factory application
echo "  → Creating Factory application..."
FACTORY_APP=$(linera create-application "$FACTORY_BYTECODE" \
    --json-argument '{"token_bytecode_id":"'"$TOKEN_BYTECODE"'"}' \
    2>&1 | grep "application ID" | awk '{print $NF}')

if [ -z "$FACTORY_APP" ]; then
    echo -e "${RED}ERROR: Failed to create Factory application${NC}"
    exit 1
fi
echo -e "${YELLOW}Factory app: $FACTORY_APP${NC}"

# Create Swap application
echo "  → Creating Swap application..."
SWAP_APP=$(linera create-application "$SWAP_BYTECODE" \
    --json-argument '{}' \
    2>&1 | grep "application ID" | awk '{print $NF}')

if [ -z "$SWAP_APP" ]; then
    echo -e "${RED}ERROR: Failed to create Swap application${NC}"
    exit 1
fi
echo -e "${YELLOW}Swap app: $SWAP_APP${NC}"

# Step 5: Request GraphQL endpoint
echo -e "${GREEN}🌐 Getting GraphQL endpoint...${NC}"
GRAPHQL_ENDPOINT=$(linera service --help 2>&1 | grep -o "http://[^[:space:]]*" | head -1)

if [ -z "$GRAPHQL_ENDPOINT" ]; then
    GRAPHQL_ENDPOINT="https://conway.graphql.linera.net"
fi

echo -e "${YELLOW}GraphQL endpoint: $GRAPHQL_ENDPOINT${NC}"

# Step 6: Save deployment info
DEPLOY_INFO="$PROJECT_ROOT/.deployment-testnet.json"
cat > "$DEPLOY_INFO" << EOF
{
  "network": "conway-testnet",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "default_chain": "$DEFAULT_CHAIN",
  "bytecode": {
    "factory": "$FACTORY_BYTECODE",
    "token": "$TOKEN_BYTECODE",
    "swap": "$SWAP_BYTECODE"
  },
  "applications": {
    "factory": "$FACTORY_APP",
    "swap": "$SWAP_APP"
  },
  "graphql_endpoint": "$GRAPHQL_ENDPOINT"
}
EOF

echo -e "${GREEN}✅ Deployment to Conway testnet complete!${NC}"
echo ""
echo -e "${YELLOW}📋 Deployment Summary:${NC}"
echo -e "  Network: ${GREEN}Conway Testnet${NC}"
echo -e "  Factory App: ${GREEN}$FACTORY_APP${NC}"
echo -e "  Swap App: ${GREEN}$SWAP_APP${NC}"
echo -e "  GraphQL Endpoint: ${GREEN}$GRAPHQL_ENDPOINT${NC}"
echo -e "  Default Chain: ${GREEN}$DEFAULT_CHAIN${NC}"
echo ""
echo -e "${YELLOW}🎮 Next Steps:${NC}"
echo "  1. Update frontend/.env with:"
echo "     VITE_FACTORY_APP_ID=$FACTORY_APP"
echo "     VITE_SWAP_APP_ID=$SWAP_APP"
echo "     VITE_GRAPHQL_ENDPOINT=$GRAPHQL_ENDPOINT"
echo ""
echo "  2. cd frontend && npm install && npm run dev"
echo "  3. Open http://localhost:5173 in your browser"
echo "  4. Create your first token launch on Conway testnet!"
echo ""
echo -e "${YELLOW}📄 Deployment info saved to: $DEPLOY_INFO${NC}"
echo ""
echo -e "${GREEN}🎉 Ready to launch tokens on Conway testnet!${NC}"
