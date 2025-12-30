#!/bin/bash

# Fair Launch - Quick Start Script
# One-command deployment for local testing

set -e

echo "================================================"
echo "🚀 Fair Launch - Quick Start"
echo "================================================"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first:"
    echo "   https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null &&! docker compose version &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose:"
    echo "   https://docs.docker.com/compose/install/"
    exit 1
fi

echo "✅ Docker detected"
echo ""

# Stop any running containers
echo "🛑 Stopping any running Fair Launch containers..."
docker compose down -v 2>/dev/null || docker-compose down -v 2>/dev/null || true

# Build and start
echo ""
echo "🏗️  Building and starting services..."
echo "   This may take 5-10 minutes on first run..."
echo ""

# Try docker compose first (newer), fall back to docker-compose
if docker compose version &> /dev/null; then
    docker compose up --build -d
else
    docker-compose up --build -d
fi

echo ""
echo "⏳ Waiting for services to be ready..."
sleep 10

# Wait for frontend
echo "   Waiting for frontend..."
for i in {1..30}; do
    if curl -s http://localhost:5173 > /dev/null; then
        break
    fi
    sleep 2
    echo -n "."
done

echo ""
echo ""
echo "================================================"
echo "✅ Fair Launch is running!"
echo "================================================"
echo ""
echo "📱 Frontend:      http://localhost:5173"
echo "🔗 GraphQL:       http://localhost:8080"
echo "📊 Network:       Local Linera testnet"
echo ""
echo "================================================"
echo "🎮 Quick Test:"
echo "================================================"
echo "1. Open http://localhost:5173 in your browser"
echo "2. Click 'Connect Wallet'"
echo "3. Navigate to 'Create Token'"
echo "4. Fill in the form and create your first token!"
echo ""
echo "================================================"
echo "📝 Logs:"
echo "================================================"
echo "View logs: docker compose logs -f"
echo "Stop:      docker compose down"
echo "Restart:   docker compose restart"
echo ""
echo "🚀 Happy building!"
echo ""
