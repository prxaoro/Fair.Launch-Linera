/**
 * Application configuration with environment variable validation
 */

const getEnvVar = (key: string, defaultValue?: string): string => {
  const value = import.meta.env[key] || defaultValue;
  if (!value) {
    throw new Error(`Missing required environment variable: ${key}`);
  }
  return value;
};

// Linera Application IDs from deployment
const DEFAULT_CHAIN_ID = 'dfada58d53643b15bf79b9ceaeb0e57a00b801ad8d6ba657e2c1e8f3b1e38ac9';
const FACTORY_APP_ID = 'ba329760710cc839fc3f99f0853c21861b11449b2f723e7c397fcb6ef24222d5';
const TOKEN_APP_ID = 'f08476beb66ad4128904dd0d1ae0809f31de70801107be7f24a28941f570f014';
const SWAP_APP_ID = '70cca1cad5a260b6440b3b74d40924a8491e23d8bbc4dc53669450d7d391d65d';

export const config = {
  // Linera service base URL
  baseUrl: getEnvVar('VITE_BASE_URL', 'http://localhost:8080'),

  // Application-specific GraphQL endpoints
  graphqlEndpoint: getEnvVar('VITE_GRAPHQL_ENDPOINT', 'http://localhost:8080'),
  factoryEndpoint: getEnvVar(
    'VITE_FACTORY_ENDPOINT',
    `http://localhost:8080/chains/${DEFAULT_CHAIN_ID}/applications/${FACTORY_APP_ID}`
  ),
  tokenEndpoint: getEnvVar(
    'VITE_TOKEN_ENDPOINT',
    `http://localhost:8080/chains/${DEFAULT_CHAIN_ID}/applications/${TOKEN_APP_ID}`
  ),
  swapEndpoint: getEnvVar(
    'VITE_SWAP_ENDPOINT',
    `http://localhost:8080/chains/${DEFAULT_CHAIN_ID}/applications/${SWAP_APP_ID}`
  ),

  // Chain and application IDs
  chainId: getEnvVar('VITE_CHAIN_ID', DEFAULT_CHAIN_ID),
  factoryAppId: getEnvVar('VITE_FACTORY_APP_ID', FACTORY_APP_ID),
  tokenAppId: getEnvVar('VITE_TOKEN_APP_ID', TOKEN_APP_ID),
  swapAppId: getEnvVar('VITE_SWAP_APP_ID', SWAP_APP_ID),

  pollInterval: parseInt(getEnvVar('VITE_POLL_INTERVAL', '2000'), 10),

  // Feature flags
  features: {
    realTimeUpdates: true,
    chartAnimations: true,
  },

  // UI Configuration
  ui: {
    tokensPerPage: 12,
    tradesPerPage: 50,
    chartDataPoints: 100,
  },

  // Trading Configuration
  trading: {
    defaultSlippage: 0.5, // 0.5%
    maxSlippage: 5.0, // 5%
    minTradeAmount: '0.000001',
  },
} as const;

export type Config = typeof config;
