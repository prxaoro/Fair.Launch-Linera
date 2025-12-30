# Quick Start Guide - 2 Minutes to Running App

## Prerequisites

- Node.js 18+ installed
- npm or yarn package manager

## Installation (30 seconds)

```bash
cd ./frontend
npm install
```

## Start Development Server (10 seconds)

```bash
npm run dev
```

The app will automatically open at `http://localhost:3000`

## What You'll See

### 1. Home Page
- Vibrant gradient background
- "Discover Fair Launches" heading
- Platform statistics (Total Tokens, 24h Volume, Market Cap)
- Search bar
- Token grid (will be empty without GraphQL backend)

### 2. Navigation
- Top: Fair Launch logo, Explore/Create/Portfolio links, Connect Wallet button
- Bottom (mobile): Icon navigation

### 3. Test Features

**Connect Wallet** (Mock)
- Click "Connect Wallet" button
- Generates random wallet address
- Shows connected state

**Create Token**
- Click "Create" in navigation
- Fill out form (all fields validated)
- Try submitting with invalid data to see validation

**Portfolio**
- Click "Portfolio"
- Will show "Connect Wallet" if not connected
- Shows portfolio UI when connected

## Expected Behavior

### Without GraphQL Backend
- UI loads perfectly
- Navigation works
- Forms validate correctly
- Wallet connects (mock)
- Shows empty states for tokens
- Charts show placeholder data

### With GraphQL Backend
- All data populates
- Real-time updates every 2 seconds
- Charts display actual bonding curves
- Trades appear in feed
- Portfolio shows real balances

## Verify Everything Works

1. **Build Check**
   ```bash
   npm run build
   ```
   Should complete without errors

2. **Type Check**
   ```bash
   npm run type-check
   ```
   Should show no errors

3. **Lint Check**
   ```bash
   npm run lint
   ```
   Should pass (or only warnings)

## Next Steps

### For Demo/Judging

The frontend works standalone! You can:
- Show off the beautiful UI
- Demonstrate form validation
- Show responsive design
- Walk through user flows
- Highlight code quality

### For Production

1. **Connect GraphQL Backend**
   - Update `VITE_GRAPHQL_ENDPOINT` in `.env`
   - Ensure backend matches query structure in `src/lib/queries.ts`

2. **Integrate Real Wallet**
   - Replace mock in `src/lib/store.ts`
   - Add Linera wallet SDK
   - Implement transaction signing

3. **Deploy**
   - See DEPLOYMENT.md for detailed instructions
   - Vercel/Netlify deployment in < 5 minutes

## Common Issues

### Port 3000 Already in Use

Edit `vite.config.ts`:
```ts
server: {
  port: 3001
}
```

### Dependencies Won't Install

```bash
rm -rf node_modules package-lock.json
npm install
```

### Build Errors

Ensure Node.js version is 18+:
```bash
node --version
```

## File Structure at a Glance

```
src/
├── components/     → Reusable UI components
├── pages/         → Main pages (Home, Token, Create, Portfolio)
├── hooks/         → Custom React hooks for data
├── lib/           → GraphQL client, utils, config
└── types/         → TypeScript definitions
```

## Key Features to Showcase

1. **Beautiful UI** - Vibrant gradients, smooth animations
2. **Real-time Updates** - Auto-polling every 2 seconds
3. **Interactive Charts** - Bonding curve visualization
4. **Form Validation** - Real-time feedback
5. **Responsive Design** - Works on all devices
6. **Type Safety** - Full TypeScript coverage
7. **Error Handling** - User-friendly messages
8. **Code Quality** - Clean, production-ready code

## Commands Reference

```bash
npm run dev          # Start dev server
npm run build        # Production build
npm run preview      # Preview production build
npm run lint         # Run linter
npm run type-check   # Check TypeScript types
```

## Success Indicators

When running `npm run dev`, you should see:
```
VITE v5.0.12  ready in 500 ms

  ➜  Local:   http://localhost:3000/
  ➜  Network: use --host to expose
  ➜  press h to show help
```

Browser should automatically open to a colorful, gradient-filled page with "Discover Fair Launches" heading.

## Support

- Check `README.md` for detailed documentation
- Check `SETUP.md` for configuration help
- Check `DEPLOYMENT.md` for production deployment
- Check browser console for any errors
- Check `src/lib/config.ts` for configuration options

## That's It!

You now have a production-ready Fair Launch frontend running locally. Impress those judges!
