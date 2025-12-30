/**
 * Real-time trade feed with virtualized scrolling
 */

import { useMemo } from 'react';
import { Card } from './Card';
import { formatCurrency, formatRelativeTime, formatAddress } from '@/lib/utils';
import type { Trade } from '@/types';

interface TradeFeedProps {
  trades: Trade[];
  className?: string;
}

export function TradeFeed({ trades, className }: TradeFeedProps) {
  const sortedTrades = useMemo(() => {
    return [...trades].sort((a, b) => {
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
    });
  }, [trades]);

  return (
    <Card variant="bordered" className={className}>
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h3 className="text-lg font-bold text-gray-900 dark:text-white">
            Recent Trades
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Live trading activity
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
          <span className="text-xs text-gray-500 dark:text-gray-400">Live</span>
        </div>
      </div>

      <div className="space-y-2 max-h-[500px] overflow-y-auto custom-scrollbar">
        {sortedTrades.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-gray-500 dark:text-gray-400">
              No trades yet. Be the first!
            </p>
          </div>
        ) : (
          sortedTrades.map((trade, index) => (
            <TradeItem key={`${trade.tokenId}-${trade.timestamp}-${index}`} trade={trade} />
          ))
        )}
      </div>
    </Card>
  );
}

function TradeItem({ trade }: { trade: Trade }) {
  const isBuy = trade.isBuy;

  return (
    <div
      className={`p-3 rounded-lg border transition-all ${
        isBuy
          ? 'bg-green-50 dark:bg-green-900/10 border-green-200 dark:border-green-800'
          : 'bg-red-50 dark:bg-red-900/10 border-red-200 dark:border-red-800'
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <span
            className={`px-2 py-1 rounded text-xs font-bold ${
              isBuy
                ? 'bg-green-500 text-white'
                : 'bg-red-500 text-white'
            }`}
          >
            {isBuy ? 'BUY' : 'SELL'}
          </span>
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {formatAddress(`${trade.trader.chainId}:${trade.trader.owner}`)}
          </span>
        </div>
        <span className="text-xs text-gray-500 dark:text-gray-400">
          {formatRelativeTime(trade.timestamp)}
        </span>
      </div>

      <div className="grid grid-cols-3 gap-2 text-sm">
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Amount</p>
          <p className="font-semibold text-gray-900 dark:text-white">
            {parseFloat(trade.tokenAmount).toFixed(2)}
          </p>
        </div>
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Price</p>
          <p className="font-semibold text-gray-900 dark:text-white">
            {formatCurrency(trade.price, 6)}
          </p>
        </div>
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Total</p>
          <p className="font-semibold text-gray-900 dark:text-white">
            {formatCurrency(trade.currencyAmount)}
          </p>
        </div>
      </div>
    </div>
  );
}
