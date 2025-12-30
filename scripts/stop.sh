#!/bin/bash
# Stop Fair Launch local network

set -e

echo "🛑 Stopping Fair Launch Network..."

# Read deployment info
DEPLOY_INFO="../.deployment.json"

if [ -f "$DEPLOY_INFO" ]; then
    NETWORK_PID=$(jq -r '.network_pid' "$DEPLOY_INFO")
    SERVICE_PID=$(jq -r '.service_pid' "$DEPLOY_INFO")

    echo "Stopping network (PID: $NETWORK_PID)..."
    kill $NETWORK_PID 2>/dev/null || echo "Network already stopped"

    echo "Stopping service (PID: $SERVICE_PID)..."
    kill $SERVICE_PID 2>/dev/null || echo "Service already stopped"

    echo "✅ Network stopped"
else
    echo "⚠️  No deployment info found. Killing all linera processes..."
    pkill -f "linera" || echo "No linera processes found"
fi
