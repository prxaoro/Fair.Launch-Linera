#!/bin/bash

# Deploy Fair Launch contracts to local Linera network
set -e

echo "Deploying Fair Launch contracts..."

# Detect environment (Docker vs local)
if [ -d "/app/contracts" ]; then
    cd /app/contracts
    OUTPUT_DIR="/app"
else
    # Running locally - find project root
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR/../contracts"
    OUTPUT_DIR="$SCRIPT_DIR/.."
fi

# Verify WASM files exist
if [ ! -f "target/wasm32-unknown-unknown/release/fair_launch_factory_contract.wasm" ]; then
    echo "ERROR: Factory contract WASM not found. Build contracts first!"
    exit 1
fi

# Deploy Factory contract
echo "Deploying Factory contract..."
FACTORY_BYTECODE=$(linera publish-module \
    target/wasm32-unknown-unknown/release/fair_launch_factory_contract.wasm \
    target/wasm32-unknown-unknown/release/fair_launch_factory_service.wasm \
    2>/dev/null)

if [ -z "$FACTORY_BYTECODE" ]; then
    echo "ERROR: Failed to publish Factory bytecode"
    exit 1
fi

echo "Factory Bytecode ID: $FACTORY_BYTECODE"

# Create Factory application
FACTORY_APP=$(linera create-application "$FACTORY_BYTECODE" \
    --json-argument '{}' \
    2>/dev/null)

if [ -z "$FACTORY_APP" ]; then
    echo "ERROR: Failed to create Factory application"
    exit 1
fi

echo "Factory Application ID: $FACTORY_APP"

# Deploy Swap contract
echo "Deploying Swap contract..."
SWAP_BYTECODE=$(linera publish-module \
    target/wasm32-unknown-unknown/release/fair_launch_swap_contract.wasm \
    target/wasm32-unknown-unknown/release/fair_launch_swap_service.wasm \
    2>/dev/null)

if [ -z "$SWAP_BYTECODE" ]; then
    echo "ERROR: Failed to publish Swap bytecode"
    exit 1
fi

echo "Swap Bytecode ID: $SWAP_BYTECODE"

# Create Swap application
SWAP_APP=$(linera create-application "$SWAP_BYTECODE" \
    --json-argument '{}' \
    2>/dev/null)

if [ -z "$SWAP_APP" ]; then
    echo "ERROR: Failed to create Swap application"
    exit 1
fi

echo "Swap Application ID: $SWAP_APP"

# Save Application IDs for reference
echo "FACTORY_APP=$FACTORY_APP" > "$OUTPUT_DIR/app_ids.env"
echo "SWAP_APP=$SWAP_APP" >> "$OUTPUT_DIR/app_ids.env"

echo ""
echo "==================================================="
echo "✅ Deployment Complete!"
echo "==================================================="
echo "Factory Application: $FACTORY_APP"
echo "Swap Application:    $SWAP_APP"
echo "==================================================="
echo ""

# Note: Token contracts are created dynamically by Factory

exit 0
