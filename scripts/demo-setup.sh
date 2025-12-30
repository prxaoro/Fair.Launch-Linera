#!/bin/bash
# =============================================================================
# FAIR LAUNCH DEMO SETUP - For Judges Video
# =============================================================================
# This script sets up a complete local demo environment showing:
# ✓ Local Linera network running
# ✓ Contracts deployed and initialized
# ✓ GraphQL service with real-time sync
# ✓ Frontend connected to local network
# ✓ End-to-end token creation, trading, portfolio tracking
# =============================================================================

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}    FAIR LAUNCH - Local Demo Setup for Judges Video${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Navigate to project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

echo -e "${YELLOW}📂 Project Directory: ${PROJECT_ROOT}${NC}"
echo ""

# =============================================================================
# STEP 1: Clean Previous Setup
# =============================================================================
echo -e "${GREEN}🧹 Step 1: Cleaning previous setup...${NC}"
pkill -f "linera" || true
rm -rf /tmp/linera_* || true
sleep 2
echo -e "   ✓ Clean complete"
echo ""

# =============================================================================
# STEP 2: Start Local Linera Network
# =============================================================================
echo -e "${GREEN}🚀 Step 2: Starting local Linera network (2 validators, 10 chains)...${NC}"

# Initialize network in temp directory
LINERA_TEMP=$(mktemp -d)
echo -e "   Network directory: ${LINERA_TEMP}"

# Start network
linera net up --extra-wallets 9 --testing-prng-seed 37 2>&1 | tee "${LINERA_TEMP}/network.log" &
NETWORK_PID=$!

# Wait for network to be ready
echo -e "   Waiting for network to initialize..."
sleep 10

# Check if network is running
if ! pgrep -f "linera-proxy" > /dev/null; then
    echo -e "${RED}❌ ERROR: Linera network failed to start${NC}"
    echo -e "Check logs at: ${LINERA_TEMP}/network.log"
    exit 1
fi

echo -e "   ✓ Network running (PID: ${NETWORK_PID})"
echo -e "   ✓ Network running on ports 9000-9001"
echo ""

# =============================================================================
# STEP 3: Build Contracts
# =============================================================================
echo -e "${GREEN}🔨 Step 3: Building all contracts...${NC}"
cd "${PROJECT_ROOT}/contracts"

# Build factory
echo -e "   Building factory contract..."
cd factory
cargo build --release --target wasm32-unknown-unknown
cd ..

# Build token
echo -e "   Building token contract..."
cd token
cargo build --release --target wasm32-unknown-unknown
cd ..

echo -e "   ✓ All contracts built successfully"
echo ""

# =============================================================================
# STEP 4: Deploy Contracts
# =============================================================================
echo -e "${GREEN}📦 Step 4: Deploying contracts to local network...${NC}"

# Deploy factory (this will output the application ID)
echo -e "   Deploying factory contract..."
FACTORY_BYTECODE="${PROJECT_ROOT}/contracts/target/wasm32-unknown-unknown/release/factory.wasm"
TOKEN_BYTECODE="${PROJECT_ROOT}/contracts/target/wasm32-unknown-unknown/release/token.wasm"

# Deploy factory application
FACTORY_APP_ID=$(linera publish-and-create \
    "${FACTORY_BYTECODE}" \
    --required-application-ids [] \
    | grep "Application ID" | awk '{print $3}')

echo -e "   ✓ Factory deployed: ${FACTORY_APP_ID}"

# Store deployment info
cat > "${PROJECT_ROOT}/.deployment.json" <<EOF
{
  "network": "local",
  "factory_application_id": "${FACTORY_APP_ID}",
  "graphql_endpoint": "http://localhost:8080",
  "deployed_at": "$(date -Iseconds)"
}
EOF

echo -e "   ✓ Deployment info saved to .deployment.json"
echo ""

# =============================================================================
# STEP 5: Start GraphQL Service
# =============================================================================
echo -e "${GREEN}🌐 Step 5: Starting GraphQL service on port 8080...${NC}"

# Start service in background
linera service --port 8080 > "${LINERA_TEMP}/graphql.log" 2>&1 &
SERVICE_PID=$!

# Wait for service to be ready
echo -e "   Waiting for GraphQL service..."
sleep 5

# Test GraphQL endpoint
if ! curl -s http://localhost:8080 > /dev/null; then
    echo -e "${RED}❌ ERROR: GraphQL service failed to start${NC}"
    echo -e "Check logs at: ${LINERA_TEMP}/graphql.log"
    exit 1
fi

echo -e "   ✓ GraphQL service running (PID: ${SERVICE_PID})"
echo -e "   ✓ GraphQL endpoint: http://localhost:8080/graphql"
echo ""

# =============================================================================
# STEP 6: Update Frontend Configuration
# =============================================================================
echo -e "${GREEN}⚙️  Step 6: Configuring frontend...${NC}"

cd "${PROJECT_ROOT}/frontend"

# Update .env with local endpoint
cat > .env.local <<EOF
VITE_GRAPHQL_ENDPOINT=http://localhost:8080
VITE_FACTORY_APPLICATION_ID=${FACTORY_APP_ID}
EOF

echo -e "   ✓ Frontend configured for local network"
echo ""

# =============================================================================
# STEP 7: Start Frontend Dev Server
# =============================================================================
echo -e "${GREEN}🎨 Step 7: Starting frontend dev server...${NC}"

# Frontend should already be running, just verify
if ! pgrep -f "vite" > /dev/null; then
    echo -e "   Starting fresh frontend server..."
    npm run dev > "${LINERA_TEMP}/frontend.log" 2>&1 &
    FRONTEND_PID=$!
    sleep 5
else
    FRONTEND_PID=$(pgrep -f "vite")
    echo -e "   Frontend already running (PID: ${FRONTEND_PID})"
fi

echo -e "   ✓ Frontend running at: http://localhost:3000"
echo ""

# =============================================================================
# DEMO READY
# =============================================================================
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ DEMO ENVIRONMENT READY FOR JUDGES VIDEO!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}📍 Services Running:${NC}"
echo -e "   • Linera Network: ports 9000-9001"
echo -e "   • GraphQL Service: http://localhost:8080"
echo -e "   • Frontend: http://localhost:3000"
echo ""
echo -e "${YELLOW}🎬 Demo Script for Judges:${NC}"
echo -e "   1. Open browser to http://localhost:3000"
echo -e "   2. Show 'Launch in seconds. Trade in real-time.' hero"
echo -e "   3. Click 'Start a new coin' → Create token with:"
echo -e "      - Name: 'Demo Coin'"
echo -e "      - Ticker: 'DEMO'"
echo -e "      - Description: 'Fair launch demo for judges'"
echo -e "   4. Show token appears in grid with bonding curve"
echo -e "   5. Click token → Show detail page with:"
echo -e "      - Live bonding curve chart"
echo -e "      - Trading panel (Buy/Sell)"
echo -e "      - Recent trades feed"
echo -e "   6. Execute buy: 100 TLIN"
echo -e "   7. Show updated bonding curve progress"
echo -e "   8. Execute sell: 50 tokens"
echo -e "   9. Click Portfolio → Show holdings and P&L"
echo -e "   10. Emphasize:"
echo -e "       ✓ Real-time sync with Linera network"
echo -e "       ✓ Instant finality (50ms)"
echo -e "       ✓ No gas wars"
echo -e "       ✓ Fair bonding curve pricing"
echo ""
echo -e "${YELLOW}📊 Deployment Info:${NC}"
echo -e "   Factory App ID: ${FACTORY_APP_ID}"
echo -e "   Network: Local (10 chains, 2 validators)"
echo -e "   GraphQL: http://localhost:8080/graphql"
echo ""
echo -e "${YELLOW}🔍 Logs:${NC}"
echo -e "   Network: ${LINERA_TEMP}/network.log"
echo -e "   GraphQL: ${LINERA_TEMP}/graphql.log"
echo -e "   Frontend: ${LINERA_TEMP}/frontend.log"
echo ""
echo -e "${RED}⚠️  To stop all services:${NC}"
echo -e "   pkill -f linera && pkill -f vite"
echo ""
echo -e "${GREEN}Ready to record! 🎥${NC}"
