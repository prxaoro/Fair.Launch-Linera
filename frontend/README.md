# Fair Launch Platform Frontend

A professional, production-ready React + TypeScript frontend for the Fair Launch platform on Linera blockchain.

## Features

- **Modern Tech Stack**: React 18, TypeScript, Vite, Tailwind CSS
- **Real-time Updates**: Automatic polling every 2 seconds via React Query
- **Beautiful UI**: Vibrant, colorful design inspired by modern DeFi platforms
- **Interactive Charts**: Recharts for bonding curve visualization
- **Type-safe GraphQL**: Full TypeScript integration with GraphQL queries
- **State Management**: Zustand for global state, React Query for server state
- **Form Validation**: Zod schemas with comprehensive error handling
- **Responsive Design**: Mobile-first approach with dedicated mobile navigation
- **Production-ready**: Error boundaries, loading states, retry logic

## Pages

1. **Home** (`/`) - Browse all token launches in a grid
2. **Token Detail** (`/token/:id`) - View bonding curve, trade, and see live feed
3. **Create Token** (`/create`) - Launch a new token with validation
4. **Portfolio** (`/portfolio`) - View holdings and P&L tracking

## Tech Stack

### Core
- **React 18.2** - UI library
- **TypeScript 5.3** - Type safety
- **Vite 5.0** - Build tool and dev server

### Styling
- **Tailwind CSS 3.4** - Utility-first CSS
- **HeadlessUI** - Accessible components
- **Heroicons** - Icon library

### Data Fetching
- **TanStack Query (React Query) 5.17** - Server state management
- **Custom GraphQL Client** - Type-safe queries with error handling

### State Management
- **Zustand 4.5** - Global state (wallet, theme)
- **React Query** - Server state with caching and polling

### Charts
- **Recharts 2.12** - React-friendly charting library

### Forms & Validation
- **Zod 3.22** - Runtime type validation
- **React Hot Toast** - Toast notifications

## Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Environment Variables

Create a `.env` file in the root directory:

```env
VITE_GRAPHQL_ENDPOINT=http://localhost:8080
VITE_POLL_INTERVAL=2000
```

## Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── Button.tsx
│   ├── Card.tsx
│   ├── Input.tsx
│   ├── Layout.tsx
│   ├── TokenCard.tsx
│   ├── BondingCurveChart.tsx
│   ├── TradeFeed.tsx
│   ├── TradeForm.tsx
│   ├── WalletButton.tsx
│   ├── LoadingSpinner.tsx
│   └── ErrorBoundary.tsx
├── pages/               # Page components
│   ├── HomePage.tsx
│   ├── TokenDetailPage.tsx
│   ├── CreateTokenPage.tsx
│   └── PortfolioPage.tsx
├── hooks/               # Custom React hooks
│   ├── useTokens.ts
│   ├── useTrades.ts
│   └── usePortfolio.ts
├── lib/                 # Core utilities
│   ├── config.ts        # App configuration
│   ├── graphql-client.ts # GraphQL client
│   ├── queries.ts       # GraphQL queries
│   ├── store.ts         # Zustand store
│   └── utils.ts         # Utility functions
├── types/               # TypeScript types
│   └── index.ts
├── App.tsx              # Root component
├── main.tsx             # Entry point
└── index.css            # Global styles
```

## Key Features

### Real-time Data

All data automatically refreshes every 2 seconds using React Query's polling:

```typescript
const { data: tokens } = useTokens(); // Auto-polls
```

### Type Safety

Every component, hook, and function is fully typed:

```typescript
interface Token {
  id: string;
  name: string;
  symbol: string;
  // ... all fields typed
}
```

### Error Handling

Comprehensive error handling at every level:

- Network errors with retry logic
- Form validation with Zod
- Error boundaries for React errors
- Toast notifications for user feedback

### Performance Optimizations

- Code splitting by route
- React Query caching
- Virtualized lists for large datasets
- Optimistic updates for mutations

## GraphQL Queries

The app expects the following GraphQL schema structure:

```graphql
type Token {
  id: String!
  name: String!
  symbol: String!
  description: String!
  currentPrice: String!
  marketCap: String!
  # ... other fields
}

type Query {
  tokens(limit: Int, offset: Int): [Token!]!
  token(id: String!): Token
  bondingCurve(tokenId: String!): BondingCurveData
  recentTrades(tokenId: String!, limit: Int): [Trade!]!
  # ... other queries
}
```

## Customization

### Colors

Edit `tailwind.config.js` to customize the color scheme:

```js
colors: {
  primary: { /* purple shades */ },
  secondary: { /* teal shades */ },
  accent: { /* orange shades */ },
}
```

### Polling Interval

Adjust in `.env`:

```env
VITE_POLL_INTERVAL=2000  # 2 seconds
```

### Trading Configuration

Edit `src/lib/config.ts`:

```typescript
trading: {
  defaultSlippage: 0.5,  // 0.5%
  maxSlippage: 5.0,      // 5%
  minTradeAmount: '0.000001',
}
```

## Production Deployment

1. Build the app:
   ```bash
   npm run build
   ```

2. The `dist/` folder contains optimized static files

3. Deploy to any static hosting (Vercel, Netlify, Cloudflare Pages, etc.)

4. Set environment variables in your hosting platform

## Browser Support

- Chrome/Edge (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)
- Mobile browsers (iOS Safari, Chrome Android)

## Contributing

This is a production-ready template. Feel free to:

- Add new pages
- Customize components
- Extend the GraphQL queries
- Add new features

## License

MIT
