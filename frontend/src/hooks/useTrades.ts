/**
 * React Query hooks for trading operations
 */

import { useMutation, useQueryClient } from '@tanstack/react-query';
import type { TradeInput } from '@/types';
import toast from 'react-hot-toast';

/**
 * Execute trade operation (buy or sell)
 * NOTE: This requires Linera wallet integration to call token contract operations
 */
export function useTrade() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (_input: TradeInput) => {
      // NOTE: This needs to call Linera operation via wallet/SDK
      // For now, throw error to indicate it needs wallet integration
      throw new Error('Trading requires Linera wallet integration. Coming soon!');
    },
    onMutate: async (input) => {
      // Optimistic update
      toast.loading(
        `${input.isBuy ? 'Buying' : 'Selling'} ${input.amount} tokens...`,
        { id: 'trade-pending' }
      );
    },
    onSuccess: () => {
      // Invalidate relevant queries
      queryClient.invalidateQueries({ queryKey: ['tokens'] });
      queryClient.invalidateQueries({ queryKey: ['token-info'] });

      toast.success(
        'Trade submitted! Waiting for confirmation...',
        { id: 'trade-pending' }
      );
    },
    onError: (error, input) => {
      console.error('Trade failed:', error);

      toast.error(
        error instanceof Error
          ? error.message
          : `Failed to ${input.isBuy ? 'buy' : 'sell'} tokens. Please try again.`,
        { id: 'trade-pending' }
      );
    },
  });
}

/**
 * Calculate trade preview (price impact, slippage, etc.)
 * TODO: This should query BUY_QUOTE_QUERY or SELL_QUOTE_QUERY for accurate pricing
 */
export function useTradePreview(
  _tokenId: string,
  amount: string,
  isBuy: boolean,
  currentPrice: number
) {
  const amountNum = parseFloat(amount) || 0;

  if (amountNum <= 0) {
    return null;
  }

  // Simplified price impact calculation
  // In production, this would query the bonding curve via BUY_QUOTE_QUERY/SELL_QUOTE_QUERY
  const priceImpact = (amountNum / 1000) * (isBuy ? 1 : -1);
  const estimatedPrice = currentPrice * (1 + priceImpact);
  const total = amountNum * estimatedPrice;
  const fee = total * 0.03; // 3% creator fee

  return {
    amount: amountNum,
    estimatedPrice,
    priceImpact: Math.abs(priceImpact * 100),
    total,
    fee,
  };
}
