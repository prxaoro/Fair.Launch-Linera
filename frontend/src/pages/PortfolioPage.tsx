/**
 * Portfolio page - User holdings and performance (placeholder)
 */

import { usePortfolio, usePortfolioSummary } from '@/hooks/usePortfolio';
import { Card } from '@/components/Card';
import { LoadingScreen } from '@/components/LoadingSpinner';
import { useStore } from '@/lib/store';
import { Button } from '@/components/Button';

export function PortfolioPage() {
  const wallet = useStore((state) => state.wallet);
  const { data: portfolio, isLoading } = usePortfolio();
  const summary = usePortfolioSummary();

  if (!wallet.isConnected) {
    return (
      <div className="max-w-2xl mx-auto py-16">
        <Card variant="bordered" className="p-8 text-center">
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
            Connect Wallet
          </h3>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Connect your wallet to view your portfolio
          </p>
        </Card>
      </div>
    );
  }

  if (isLoading) {
    return <LoadingScreen message="Loading portfolio..." />;
  }

  if (!portfolio || portfolio.positions.length === 0) {
    return (
      <div className="max-w-2xl mx-auto py-16">
        <Card variant="bordered" className="p-8 text-center">
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
            No Holdings Yet
          </h3>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Start trading tokens to build your portfolio!
          </p>
          <Button variant="primary" onClick={() => window.location.href = '/'}>
            Explore Tokens
          </Button>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
        Portfolio
      </h1>

      {/* Summary Cards */}
      {summary && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card variant="bordered" className="p-6">
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">
              Total Value
            </p>
            <p className="text-3xl font-bold text-gray-900 dark:text-white">
              ${summary.totalValue}
            </p>
          </Card>

          <Card variant="bordered" className="p-6">
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">
              Total P&L
            </p>
            <p
              className={`text-3xl font-bold ${
                summary.totalProfitLossPercent >= 0
                  ? 'text-green-600'
                  : 'text-red-600'
              }`}
            >
              ${summary.totalProfitLoss}
            </p>
          </Card>

          <Card variant="bordered" className="p-6">
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">
              P&L %
            </p>
            <p
              className={`text-3xl font-bold ${
                summary.totalProfitLossPercent >= 0
                  ? 'text-green-600'
                  : 'text-red-600'
              }`}
            >
              {summary.totalProfitLossPercent.toFixed(2)}%
            </p>
          </Card>
        </div>
      )}

      {/* Holdings Table */}
      <Card variant="bordered">
        <div className="p-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
            Holdings
          </h2>
          <p className="text-gray-600 dark:text-gray-400">
            Portfolio tracking coming soon...
          </p>
        </div>
      </Card>
    </div>
  );
}
