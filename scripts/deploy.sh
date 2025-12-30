#!/bin/bash
# Fair Launch Deployment Script for Local Linera Network

set -e  # Exit on error

echo "🚀 Fair Launch - Deployment Script"
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
STORAGE_DIR="/tmp/fair-launch-storage"
WALLET_PATH="$STORAGE_DIR/wallet.db"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${YELLOW}📁 Project root: $PROJECT_ROOT${NC}"

# Clean previous deployment
if [ -d "$STORAGE_DIR" ]; then
    echo -e "${YELLOW}🧹 Cleaning previous deployment...${NC}"
    rm -rf "$STORAGE_DIR"
fi

mkdir -p "$STORAGE_DIR"

# Step 1: Build contracts
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

# Step 2: Start local Linera network
echo -e "${GREEN}📡 Starting local Linera network...${NC}"
linera net up --storage rocksdb:"$STORAGE_DIR" &
NETWORK_PID=$!

# Wait for network to be ready
sleep 5

# Step 3: Initialize wallet
echo -e "${GREEN}💼 Initializing wallet...${NC}"
export LINERA_WALLET="$WALLET_PATH"
export LINERA_STORAGE="rocksdb:$STORAGE_DIR"

# Get default chain
DEFAULT_CHAIN=$(linera wallet show | grep "Public Key" | head -1 | awk '{print $4}')
echo -e "${YELLOW}Default chain: $DEFAULT_CHAIN${NC}"

# Step 4: Publish bytecode
echo -e "${GREEN}📦 Publishing bytecode...${NC}"

# Publish Factory
echo "  → Publishing Factory..."
FACTORY_BYTECODE=$(linera project publish-and-create \
    --path "$PROJECT_ROOT/contracts/factory" 2>&1 | grep "bytecode ID" | awk '{print $NF}')
echo -e "${YELLOW}Factory bytecode: $FACTORY_BYTECODE${NC}"

# Publish Token
echo "  → Publishing Token..."
TOKEN_BYTECODE=$(linera project publish-and-create \
    --path "$PROJECT_ROOT/contracts/token" 2>&1 | grep "bytecode ID" | awk '{print $NF}')
echo -e "${YELLOW}Token bytecode: $TOKEN_BYTECODE${NC}"

# Publish Swap
echo "  → Publishing Swap..."
SWAP_BYTECODE=$(linera project publish-and-create \
    --path "$PROJECT_ROOT/contracts/swap" 2>&1 | grep "bytecode ID" | awk '{print $NF}')
echo -e "${YELLOW}Swap bytecode: $SWAP_BYTECODE${NC}"

# Step 5: Create applications
echo -e "${GREEN}🚀 Creating applications...${NC}"

# Create Factory application
echo "  → Creating Factory application..."
FACTORY_APP=$(linera create-application "$FACTORY_BYTECODE" 2>&1 | grep "application ID" | awk '{print $NF}')
echo -e "${YELLOW}Factory app: $FACTORY_APP${NC}"

# Create Swap application
echo "  → Creating Swap application..."
SWAP_APP=$(linera create-application "$SWAP_BYTECODE" 2>&1 | grep "application ID" | awk '{print $NF}')
echo -e "${YELLOW}Swap app: $SWAP_APP${NC}"

# Step 6: Start Linera service (GraphQL endpoint)
echo -e "${GREEN}🌐 Starting Linera service on port 8080...${NC}"
linera service --port 8080 &
SERVICE_PID=$!

sleep 3

# Step 7: Save deployment info
DEPLOY_INFO="$PROJECT_ROOT/.deployment.json"
cat > "$DEPLOY_INFO" << EOF
{
  "network": "local",
  "storage": "$STORAGE_DIR",
  "wallet": "$WALLET_PATH",
  "default_chain": "$DEFAULT_CHAIN",
  "factory_bytecode": "$FACTORY_BYTECODE",
  "token_bytecode": "$TOKEN_BYTECODE",
  "swap_bytecode": "$SWAP_BYTECODE",
  "factory_app": "$FACTORY_APP",
  "swap_app": "$SWAP_APP",
  "graphql_endpoint": "http://localhost:8080",
  "network_pid": $NETWORK_PID,
  "service_pid": $SERVICE_PID,
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}✅ Deployment complete!${NC}"
echo ""
echo -e "${YELLOW}📋 Deployment Summary:${NC}"
echo -e "  Factory App: ${GREEN}$FACTORY_APP${NC}"
echo -e "  Swap App: ${GREEN}$SWAP_APP${NC}"
echo -e "  GraphQL Endpoint: ${GREEN}http://localhost:8080${NC}"
echo -e "  Default Chain: ${GREEN}$DEFAULT_CHAIN${NC}"
echo ""
echo -e "${YELLOW}🎮 Next Steps:${NC}"
echo "  1. cd frontend && npm install && npm run dev"
echo "  2. Open http://localhost:5173 in your browser"
echo "  3. Create your first token launch!"
echo ""
echo -e "${YELLOW}📄 Deployment info saved to: $DEPLOY_INFO${NC}"
echo ""
echo -e "${RED}⚠️  To stop the network:${NC}"
echo "  kill $NETWORK_PID $SERVICE_PID"
