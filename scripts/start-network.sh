#!/bin/bash

# Start Linera local testnet
set -e

echo "Starting Linera local testnet..."

# Initialize and start local network
linera net up --extra-wallets 2 --testing-prng-seed 37

echo "Linera network started successfully!"
echo "Validator running on port 9000"

# Show wallet info
linera wallet show

exit 0
