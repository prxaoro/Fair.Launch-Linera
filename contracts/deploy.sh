#!/bin/bash
set -e

echo "=================================="
echo "Fair Launch - Local Deployment"
echo "=================================="

# Step 1: Set up Linera network environment
echo ""
echo "[1/3] Setting up Linera network..."
eval "$(linera net up --other-initial-chains 9 --testing-prng-seed 37 --validators 2)"

# Verify wallet
echo ""
echo "Wallet information:"
linera wallet show

# Step 2: Deploy contracts
echo ""
echo "[2/3] Deploying Fair Launch contracts..."
linera project publish-and-create

# Step 3: Get application IDs
echo ""
echo "[3/3] Extracting application IDs..."
export FACTORY_APP_ID=$(linera project show | grep "factory" | awk '{print $NF}')
echo "Factory Application ID: $FACTORY_APP_ID"

echo ""
echo "=================================="
echo "Deployment Complete!"
echo "=================================="
echo ""
echo "Next steps:"
echo "1. Start GraphQL service: linera service --port 8080"
echo "2. Update frontend with application IDs"
echo "3. Test token creation and trading"
echo ""
