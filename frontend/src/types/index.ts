/**
 * Core type definitions for Fair Launch platform
 */

/**
 * Linera Account type - uniquely identifies a user on a chain
 * Format: { chainId: "...", owner: "..." }
 */
export interface Account {
  chainId: string;
  owner: string;
}

/**
 * Token metadata from GraphQL API
 */
export interface TokenMetadata {
  name: string;
  symbol: string;
  description: string;
  imageUrl?: string;
  twitter?: string;
  telegram?: string;
  website?: string;
}

/**
 * Bonding curve configuration
 */
export interface BondingCurveConfig {
  k: string;
  scale: string;
  targetRaise: string;
  maxSupply: string;
  creatorFeeBps: number;
}

/**
 * Token from GraphQL API - matches actual backend structure
 */
export interface Token {
  tokenId: string;
  creator: string;
  metadata: TokenMetadata;
  curveConfig: BondingCurveConfig;
  currentSupply: string;
  totalRaised: string;
  isGraduated: boolean;
  createdAt: string;
  dexPoolId?: string;
}

/**
 * Trade from GraphQL API
 */
export interface Trade {
  tokenId: string;
  trader: Account;
  isBuy: boolean;
  tokenAmount: string;
  currencyAmount: string;
  price: string;
  timestamp: string;
}

export interface PricePoint {
  timestamp: number;
  price: number;
  volume: number;
}

/**
 * Bonding curve data for chart visualization
 */
export interface BondingCurveData {
  currentSupply: string;
  currentPrice: string;
  points: Array<{
    supply: number;
    price: number;
  }>;
}

/**
 * User token balance
 */
export interface UserBalance {
  account: Account;
  tokenId: string;
  balance: string;
}

/**
 * Wallet connection state
 */
export interface WalletState {
  account: Account | null;
  isConnected: boolean;
}

/**
 * Input for creating a new token
 */
export interface CreateTokenInput {
  metadata: TokenMetadata;
  curveConfig?: BondingCurveConfig;
}

/**
 * Input for trading tokens
 */
export interface TradeInput {
  tokenId: string;
  amount: string;
  isBuy: boolean;
  maxCost?: string;
  minReturn?: string;
}

export interface GraphQLResponse<T> {
  data?: T;
  errors?: Array<{
    message: string;
    path?: string[];
    extensions?: Record<string, unknown>;
  }>;
}

/**
 * GraphQL query results
 */
export interface TokensQueryResult {
  tokens: Token[];
}

export interface TokenQueryResult {
  token: Token;
}

/**
 * Token info from token contract (separate from factory data)
 */
export interface TokenInfo {
  tokenId: string;
  creator: Account;
  name: string;
  symbol: string;
  description: string;
  currentSupply: string;
  totalRaised: string;
  currentPrice: string;
  holderCount: number;
  tradeCount: number;
  isGraduated: boolean;
  progressPercentage: number;
}

/**
 * Buy/Sell quote
 */
export interface TradeQuote {
  tokenAmount: string;
  currencyAmount: string;
  priceImpact: number;
  newPrice: string;
}

export interface PortfolioQueryResult {
  balances: UserBalance[];
}

export interface StatsData {
  totalTokens: number;
  totalVolume: string;
  totalTrades: number;
  totalRaised: string;
}
