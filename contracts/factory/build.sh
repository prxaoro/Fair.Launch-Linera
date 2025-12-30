#!/bin/bash
# Build script for Fair Launch Factory contract

set -e  # Exit on error

echo "Building Fair Launch Factory Contract..."
echo "========================================"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check for required tools
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command -v wasm-opt &> /dev/null; then
    echo -e "${YELLOW}Warning: wasm-opt not found. Install binaryen for smaller WASM files.${NC}"
fi

# Build for WASM target
echo -e "${YELLOW}Building for wasm32-unknown-unknown target...${NC}"
cargo build --release --target wasm32-unknown-unknown

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Build successful!${NC}"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

# Optimize WASM if wasm-opt is available
WASM_PATH="target/wasm32-unknown-unknown/release/fair_launch_factory.wasm"
if command -v wasm-opt &> /dev/null && [ -f "$WASM_PATH" ]; then
    echo -e "${YELLOW}Optimizing WASM binary...${NC}"
    wasm-opt -Oz "$WASM_PATH" -o "${WASM_PATH}.opt"
    mv "${WASM_PATH}.opt" "$WASM_PATH"
    echo -e "${GREEN}Optimization complete!${NC}"
fi

# Display file size
if [ -f "$WASM_PATH" ]; then
    SIZE=$(du -h "$WASM_PATH" | cut -f1)
    echo -e "${GREEN}WASM binary size: ${SIZE}${NC}"
    echo -e "${GREEN}Location: ${WASM_PATH}${NC}"
fi

echo ""
echo -e "${GREEN}Build complete!${NC}"
echo "To deploy: linera publish-bytecode $WASM_PATH"
