#!/bin/bash
# Test script for Fair Launch Factory contract

set -e

echo "Running Fair Launch Factory Tests..."
echo "====================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Run unit tests
echo -e "${YELLOW}Running unit tests...${NC}"
cargo test --package fair-launch-factory -- --nocapture

if [ $? -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
else
    echo -e "${RED}Tests failed!${NC}"
    exit 1
fi

# Run clippy for linting
echo ""
echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy --package fair-launch-factory -- -D warnings

if [ $? -eq 0 ]; then
    echo -e "${GREEN}No clippy warnings!${NC}"
else
    echo -e "${RED}Clippy found issues!${NC}"
    exit 1
fi

# Check formatting
echo ""
echo -e "${YELLOW}Checking code formatting...${NC}"
cargo fmt --package fair-launch-factory -- --check

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Code is properly formatted!${NC}"
else
    echo -e "${RED}Code needs formatting! Run: cargo fmt${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}All checks passed!${NC}"
