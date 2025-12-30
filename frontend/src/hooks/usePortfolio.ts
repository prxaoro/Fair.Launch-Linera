/**
 * Portfolio management hooks with Linera wallet integration
 */

import { useQuery } from '@tanstack/react-query';
import { useStore } from '@/lib/store';
import { graphqlClient } from '@/lib/graphql-client';
import { PORTFOLIO_QUERY } from '@/lib/queries';
import { accountToJson } from '@/lib/wallet-utils';
import { config } from '@/lib/config';

/**
 * Fetch user portfolio with positions and trades
 */
export function usePortfolio() {
  const wallet = useStore((state) => state.wallet);

  return useQuery({
    queryKey: ['portfolio', wallet.account],
    queryFn: async () => {
      if (!wallet.account) {
        throw new Error('Wallet not connected');
      }

      const accountJson = accountToJson(wallet.account);

      const response = await graphqlClient.query(
        PORTFOLIO_QUERY,
        { accountJson },
        { endpoint: 'token' }
      ) as {
        balance?: string;
        user_position?: Array<{
          tokenId: string;
          balance: string;
          totalInvested: string;
          tradesCount: number;
        }>;
        user_trades?: Array<{
          tokenId: string;
          isBuy: boolean;
          token_amount: string;
          currency_amount: string;
          price: string;
        }>;
      };

      return {
        balance: response.balance || '0',
        positions: response.user_position || [],
        trades: response.user_trades || [],
      };
    },
    enabled: !!wallet.account && wallet.isConnected,
    refetchInterval: config.pollInterval,
    staleTime: config.pollInterval,
    retry: 3,
  });
}

/**
 * Get portfolio summary with calculated metrics
 */
export function usePortfolioSummary() {
  const { data } = usePortfolio();

  if (!data) {
    return null;
  }

  // Calculate total value of all positions
  const totalValue = data.positions.reduce((sum, position) => {
    return sum + parseFloat(position.balance);
  }, 0);

  // Calculate total invested across all positions
  const totalInvested = data.positions.reduce((sum, position) => {
    return sum + parseFloat(position.totalInvested);
  }, 0);

  // Calculate profit/loss
  const totalProfitLoss = totalValue - totalInvested;
  const totalProfitLossPercent = totalInvested > 0
    ? (totalProfitLoss / totalInvested) * 100
    : 0;

  // Find top performers (by profit %)
  const positionsWithPL = data.positions.map(position => {
    const value = parseFloat(position.balance);
    const invested = parseFloat(position.totalInvested);
    const pl = value - invested;
    const plPercent = invested > 0 ? (pl / invested) * 100 : 0;

    return {
      ...position,
      currentValue: value,
      profitLoss: pl,
      profitLossPercent: plPercent,
    };
  });

  const sorted = [...positionsWithPL].sort((a, b) =>
    b.profitLossPercent - a.profitLossPercent
  );

  return {
    totalValue: totalValue.toFixed(2),
    totalInvested: totalInvested.toFixed(2),
    totalProfitLoss: totalProfitLoss.toFixed(2),
    totalProfitLossPercent,
    topPerformers: sorted.slice(0, 5),
    worstPerformers: sorted.slice(-5).reverse(),
    totalTrades: data.trades.length,
    nativeBalance: data.balance,
  };
}
