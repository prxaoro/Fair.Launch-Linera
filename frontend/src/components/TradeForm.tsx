/**
 * Trading form component for buying and selling tokens
 */

import { useState, useMemo } from 'react';
import { Card } from './Card';
import { Button } from './Button';
import { Input } from './Input';
import { formatCurrency, formatPercentage } from '@/lib/utils';
import { useTrade, useTradePreview } from '@/hooks/useTrades';
import { useStore } from '@/lib/store';
import { calculateCreatorFee, formatBasisPoints } from '@/lib/wallet-utils';
import toast from 'react-hot-toast';
import type { Token } from '@/types';

interface TradeFormProps {
  token: Token;
  className?: string;
}

export function TradeForm({ token, className }: TradeFormProps) {
  const [tradeType, setTradeType] = useState<'BUY' | 'SELL'>('BUY');
  const [amount, setAmount] = useState('');
  const wallet = useStore((state) => state.wallet);
  const tradeMutation = useTrade();

  // Calculate current price from bonding curve
  const currentSupply = parseFloat(token.currentSupply);
  const k = parseFloat(token.curveConfig.k);
  const scale = parseFloat(token.curveConfig.scale);
  const currentPrice = (k * Math.pow(currentSupply / scale, 2)) / 1e6;

  const preview = useTradePreview(token.tokenId, amount, tradeType === 'BUY', currentPrice);

  const handleTrade = async () => {
    if (!wallet.isConnected) {
      toast.error('Please connect your wallet first');
      return;
    }

    if (!amount || parseFloat(amount) <= 0) {
      toast.error('Please enter a valid amount');
      return;
    }

    try {
      await tradeMutation.mutateAsync({
        tokenId: token.tokenId,
        amount,
        isBuy: tradeType === 'BUY',
      });

      setAmount('');
    } catch (error) {
      console.error('Trade error:', error);
    }
  };

  const quickAmounts = useMemo(() => {
    if (tradeType === 'BUY') {
      return [10, 50, 100, 500];
    } else {
      return [25, 50, 75, 100]; // Percentages for selling
    }
  }, [tradeType]);

  return (
    <Card variant="bordered" className={className}>
      <div className="mb-4">
        <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-2">
          Trade {token.metadata.symbol}
        </h3>

        {/* Buy/Sell Toggle */}
        <div className="flex gap-2">
          <button
            onClick={() => setTradeType('BUY')}
            className={`flex-1 py-2 px-4 rounded-lg font-semibold transition-all ${
              tradeType === 'BUY'
                ? 'bg-green-500 text-white shadow-lg'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
          >
            Buy
          </button>
          <button
            onClick={() => setTradeType('SELL')}
            className={`flex-1 py-2 px-4 rounded-lg font-semibold transition-all ${
              tradeType === 'SELL'
                ? 'bg-red-500 text-white shadow-lg'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
          >
            Sell
          </button>
        </div>
      </div>

      {/* Amount Input */}
      <div className="space-y-3">
        <Input
          type="number"
          label="Amount"
          placeholder="0.00"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          step="0.000001"
          min="0"
        />

        {/* Quick Amount Buttons */}
        <div className="grid grid-cols-4 gap-2">
          {quickAmounts.map((value) => (
            <button
              key={value}
              onClick={() => {
                if (tradeType === 'BUY') {
                  const amountToBuy = value / currentPrice;
                  setAmount(amountToBuy.toFixed(6));
                } else {
                  // For sell, this would be percentage of holdings
                  setAmount('0');
                }
              }}
              className="py-1.5 px-2 text-xs font-semibold rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-primary-100 dark:hover:bg-primary-900 text-gray-700 dark:text-gray-300 transition-colors"
            >
              {tradeType === 'BUY' ? `$${value}` : `${value}%`}
            </button>
          ))}
        </div>

        {/* Trade Preview */}
        {preview && (
          <div className="p-3 bg-gradient-to-r from-primary-50 to-secondary-50 dark:from-primary-950 dark:to-secondary-950 rounded-lg space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-600 dark:text-gray-400">
                Estimated Price:
              </span>
              <span className="font-semibold text-gray-900 dark:text-white">
                {formatCurrency(preview.estimatedPrice, 6)}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-600 dark:text-gray-400">
                Price Impact:
              </span>
              <span
                className={`font-semibold ${
                  preview.priceImpact > 1
                    ? 'text-red-600 dark:text-red-400'
                    : 'text-green-600 dark:text-green-400'
                }`}
              >
                {formatPercentage(preview.priceImpact)}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-600 dark:text-gray-400">
                Creator Fee ({formatBasisPoints(token.curveConfig.creatorFeeBps)}):
              </span>
              <span className="font-semibold text-gray-900 dark:text-white">
                {formatCurrency(calculateCreatorFee(preview.total.toString(), token.curveConfig.creatorFeeBps))}
              </span>
            </div>
            <div className="pt-2 border-t border-gray-300 dark:border-gray-600 flex justify-between">
              <span className="font-semibold text-gray-900 dark:text-white">
                Total:
              </span>
              <span className="font-bold text-lg text-gray-900 dark:text-white">
                {formatCurrency(preview.total)}
              </span>
            </div>
          </div>
        )}

        {/* Trade Button */}
        <Button
          variant={tradeType === 'BUY' ? 'primary' : 'danger'}
          fullWidth
          size="lg"
          onClick={handleTrade}
          disabled={!wallet.isConnected || !amount || parseFloat(amount) <= 0}
          isLoading={tradeMutation.isPending}
        >
          {!wallet.isConnected
            ? 'Connect Wallet'
            : tradeType === 'BUY'
            ? 'Buy Tokens'
            : 'Sell Tokens'}
        </Button>
      </div>
    </Card>
  );
}
