# Fair Launch Frontend Setup Guide

## Quick Start

### 1. Install Dependencies

```bash
cd ./frontend
npm install
```

### 2. Configure Environment

The `.env` file is already created with default values:

```env
VITE_GRAPHQL_ENDPOINT=http://localhost:8080
VITE_POLL_INTERVAL=2000
```

Update these values if your GraphQL endpoint is different.

### 3. Start Development Server

```bash
npm run dev
```

The app will open at `http://localhost:3000`

## Available Scripts

```bash
# Development
npm run dev              # Start dev server with hot reload

# Production
npm run build            # Build for production
npm run preview          # Preview production build locally

# Code Quality
npm run lint             # Run ESLint
npm run type-check       # Run TypeScript compiler check
```

## Verification Checklist

After running `npm run dev`, verify:

1. **Home Page** - Token grid loads
2. **Navigation** - All links work (Home, Create, Portfolio)
3. **Wallet Connection** - Connect wallet button works
4. **Create Token** - Form validation works
5. **Token Detail** - Charts render correctly
6. **Real-time Updates** - Data refreshes every 2 seconds

## GraphQL Integration

The frontend expects these GraphQL queries to work:

### Required Queries
- `tokens(limit, offset)` - List all tokens
- `token(id)` - Get token details
- `bondingCurve(tokenId)` - Get bonding curve data
- `recentTrades(tokenId, limit)` - Get trade history
- `balances(address)` - Get user portfolio (requires wallet)

### Required Mutations
- `createToken(input)` - Create new token
- `executeTrade(input)` - Execute buy/sell trade

## Customization

### Change Color Scheme

Edit `tailwind.config.js`:

```js
colors: {
  primary: { /* Your primary color palette */ },
  secondary: { /* Your secondary color palette */ },
  accent: { /* Your accent color palette */ },
}
```

### Change Polling Interval

Edit `.env`:

```env
VITE_POLL_INTERVAL=5000  # 5 seconds instead of 2
```

### Add New Features

1. Create new page in `src/pages/`
2. Add route in `src/App.tsx`
3. Create new GraphQL query in `src/lib/queries.ts`
4. Create custom hook in `src/hooks/`

## Troubleshooting

### Port Already in Use

Change port in `vite.config.ts`:

```ts
server: {
  port: 3001,  // Change to any available port
}
```

### GraphQL Connection Errors

1. Check GraphQL endpoint is running
2. Verify VITE_GRAPHQL_ENDPOINT in `.env`
3. Check browser console for CORS errors

### Build Errors

```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Type Errors

```bash
# Run type checker
npm run type-check
```

## Production Deployment

### Vercel

```bash
npm install -g vercel
vercel
```

### Netlify

```bash
npm run build
# Drag and drop 'dist' folder to Netlify
```

### Custom Server

```bash
npm run build
# Serve 'dist' folder with any static server
```

## Support

For issues or questions:

1. Check the main README.md
2. Review GraphQL queries in src/lib/queries.ts
3. Check browser console for errors
4. Verify GraphQL endpoint is accessible

## Next Steps

1. Connect to actual Linera blockchain
2. Integrate real wallet (MetaMask, WalletConnect, or Linera wallet)
3. Replace mock data with real GraphQL responses
4. Add analytics tracking
5. Set up monitoring and error reporting
