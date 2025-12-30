#!/bin/bash

# Fair Launch - Create First Token via CLI
# This will create a token that will appear in your frontend!

# Set environment variables
export LINERA_WALLET=/tmp/.tmpKRl5jk/wallet_0.json
export LINERA_KEYSTORE=/tmp/.tmpKRl5jk/keystore_0.json
export LINERA_STORAGE="rocksdb:/tmp/.tmpKRl5jk/client_0.db"

# Application and chain IDs
FACTORY_APP_ID="ba329760710cc839fc3f99f0853c21861b11449b2f723e7c397fcb6ef24222d5"
DEFAULT_CHAIN="dfada58d53643b15bf79b9ceaeb0e57a00b801ad8d6ba657e2c1e8f3b1e38ac9"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Creating Your First Token! 🚀"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Factory App: $FACTORY_APP_ID"
echo "Chain: $DEFAULT_CHAIN"
echo ""

# Create the operation JSON
cat > /tmp/create-token-op.json << 'EOF'
{
  "CreateToken": {
    "metadata": {
      "name": "Fair Launch Test Token",
      "symbol": "FLTT",
      "description": "The very first token on Fair Launch! This is a test token to demonstrate the platform.",
      "image_url": "https://ui-avatars.com/api/?name=FLTT&background=9333ea&color=fff&size=400",
      "twitter": "https://twitter.com/linera_io",
      "telegram": null,
      "website": "https://linera.io"
    },
    "curve_config": {
      "k": "1000000000000000000",
      "scale": "1000000000000000000000000",
      "target_raise": "10000000000000000000000",
      "max_supply": "1000000000000000000000000",
      "creator_fee_bps": 300
    }
  }
}
EOF

echo "📝 Token Details:"
echo "  Name: Fair Launch Test Token"
echo "  Symbol: FLTT"
echo "  Target Raise: 10,000 tokens"
echo "  Max Supply: 1,000,000 tokens"
echo "  Creator Fee: 3%"
echo ""
echo "Executing CreateToken operation..."
echo ""

# Execute the operation
linera execute \
  --chain "$DEFAULT_CHAIN" \
  --operation-json-path /tmp/create-token-op.json \
  --application-id "$FACTORY_APP_ID"

if [ $? -eq 0 ]; then
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  ✅ TOKEN CREATED SUCCESSFULLY!"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "🎉 Your first token is now on the blockchain!"
  echo ""
  echo "Next steps:"
  echo "1. Refresh your frontend at http://localhost:3000"
  echo "2. You should see 'Fair Launch Test Token (FLTT)' in the grid!"
  echo "3. Click on it to see the bonding curve and trading interface"
  echo ""
  echo "The GraphQL query will automatically fetch it:"
  echo "  { tokens { tokenId metadata { name symbol } } }"
  echo ""
else
  echo ""
  echo "❌ Failed to create token"
  echo "Check the error message above for details"
fi
