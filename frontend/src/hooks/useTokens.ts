/**
 * React Query hooks for token data fetching
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { graphqlClient } from '@/lib/graphql-client';
import {
  TOKENS_QUERY,
  TOKEN_DETAIL_QUERY,
  TOKEN_INFO_QUERY,
} from '@/lib/queries';
import { config } from '@/lib/config';
import toast from 'react-hot-toast';

/**
 * Fetch all tokens with automatic polling
 */
export function useTokens(limit = 100, offset = 0) {
  return useQuery({
    queryKey: ['tokens', limit, offset],
    queryFn: async () => {
      const response = await graphqlClient.query(
        TOKENS_QUERY,
        { limit, offset },
        { endpoint: 'factory' }
      ) as { tokens?: any[] };
      return response.tokens || [];
    },
    refetchInterval: config.pollInterval,
    staleTime: config.pollInterval,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Fetch single token detail from factory
 */
export function useTokenDetail(tokenId: string) {
  return useQuery({
    queryKey: ['token', tokenId],
    queryFn: async () => {
      const response = await graphqlClient.query(
        TOKEN_DETAIL_QUERY,
        { tokenId },
        { endpoint: 'factory' }
      ) as { token?: any };
      return response.token;
    },
    enabled: !!tokenId,
    refetchInterval: config.pollInterval,
    staleTime: config.pollInterval,
    retry: 3,
  });
}

/**
 * Fetch token info from token contract (includes current price, holders, etc)
 */
export function useTokenInfo() {
  return useQuery({
    queryKey: ['token-info'],
    queryFn: async () => {
      const response = await graphqlClient.query(
        TOKEN_INFO_QUERY,
        {},
        { endpoint: 'token' }
      ) as { token_info?: any };
      return response.token_info;
    },
    refetchInterval: config.pollInterval,
    staleTime: config.pollInterval,
    retry: 3,
  });
}

/**
 * Create new token - uses Linera operations, not mutations
 * This needs to call the factory contract operation
 */
export function useCreateToken() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (_input: any) => {
      // NOTE: This needs to call Linera operation via wallet/SDK
      // For now, throw error to indicate it needs wallet integration
      throw new Error('Token creation requires Linera wallet integration');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tokens'] });
      toast.success('Token creation submitted! Waiting for confirmation...');
    },
    onError: (error) => {
      console.error('Failed to create token:', error);
      toast.error(
        error instanceof Error
          ? error.message
          : 'Failed to create token. Please try again.'
      );
    },
  });
}

/**
 * Get token statistics
 */
export function useTokenStats(tokenId: string) {
  const { data: tokenData } = useTokenDetail(tokenId);

  if (!tokenData) {
    return null;
  }

  const { token, bondingCurve, recentTrades } = tokenData;

  return {
    currentPrice: parseFloat(token.currentPrice),
    marketCap: parseFloat(token.marketCap),
    volume24h: parseFloat(token.volume24h),
    priceChange24h: token.priceChange24h,
    totalSupply: parseFloat(token.totalSupply),
    currentSupply: parseFloat(bondingCurve.currentSupply),
    holders: token.holders,
    totalTrades: recentTrades.length,
  };
}
