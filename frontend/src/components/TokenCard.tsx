/**
 * Token card component for displaying token information in grid
 */

import { Link } from 'react-router-dom';
import { Card } from './Card';
import { formatCurrency, formatNumber } from '@/lib/utils';
import { formatBasisPoints } from '@/lib/wallet-utils';
import type { Token } from '@/types';

interface TokenCardProps {
  token: Token;
}

export function TokenCard({ token }: TokenCardProps) {
  // Calculate current price from bonding curve
  const currentSupply = parseFloat(token.currentSupply);
  const k = parseFloat(token.curveConfig.k);
  const scale = parseFloat(token.curveConfig.scale);
  const currentPrice = (k * Math.pow(currentSupply / scale, 2)) / 1e6; // Approximate price

  // Calculate progress to graduation
  const progress = (parseFloat(token.totalRaised) / parseFloat(token.curveConfig.targetRaise)) * 100;

  return (
    <Link to={`/token/${token.tokenId}`}>
      <Card
        variant="bordered"
        className="group hover:border-primary-500 hover:shadow-xl transition-all cursor-pointer overflow-hidden"
      >
        {/* Token Image */}
        <div className="relative aspect-square rounded-lg overflow-hidden mb-3 bg-gradient-to-br from-primary-400 to-secondary-400">
          {token.metadata.imageUrl ? (
            <img
              src={token.metadata.imageUrl}
              alt={token.metadata.name}
              className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-300"
              onError={(e) => {
                e.currentTarget.src = `https://ui-avatars.com/api/?name=${encodeURIComponent(token.metadata.symbol)}&background=random&size=400`;
              }}
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-white text-4xl font-bold">
              {token.metadata.symbol.charAt(0)}
            </div>
          )}

          {/* Graduation Progress Badge */}
          <div className="absolute top-2 right-2 px-2 py-1 rounded-lg text-xs font-bold backdrop-blur-sm bg-primary-500/90 text-white">
            {progress.toFixed(1)}% to DEX
          </div>

          {/* Creator Fee Badge */}
          <div className="absolute top-2 left-2 px-2 py-1 rounded-lg text-xs font-bold backdrop-blur-sm bg-secondary-500/90 text-white">
            {formatBasisPoints(token.curveConfig.creatorFeeBps)} fee
          </div>
        </div>

        {/* Token Info */}
        <div className="space-y-2">
          <div>
            <h3 className="text-lg font-bold text-gray-900 dark:text-white truncate">
              {token.metadata.name}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              ${token.metadata.symbol}
            </p>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs text-gray-500 dark:text-gray-400">Price</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatCurrency(currentPrice.toString(), 6)}
              </p>
            </div>
            <div className="text-right">
              <p className="text-xs text-gray-500 dark:text-gray-400">Raised</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatCurrency(token.totalRaised)}
              </p>
            </div>
          </div>

          <div className="pt-2 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between text-xs">
            <div>
              <span className="text-gray-500 dark:text-gray-400">Supply:</span>
              <span className="ml-1 font-semibold text-gray-900 dark:text-white">
                {formatNumber(parseFloat(token.currentSupply), 0)}
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Status:</span>
              <span className="ml-1 font-semibold text-gray-900 dark:text-white">
                {token.isGraduated ? '🎓 Graduated' : '🚀 Active'}
              </span>
            </div>
          </div>
        </div>
      </Card>
    </Link>
  );
}
