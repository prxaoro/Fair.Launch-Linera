/**
 * Home page - Pump.fun style token launchpad
 */

import { useState } from 'react';
import { Flame } from 'lucide-react';
import { useTokens } from '@/hooks/useTokens';
import { LoadingSpinner } from '@/components/LoadingSpinner';

export function HomePage() {
  const [activeFilter, setActiveFilter] = useState('Trending');
  const { data: tokens, isLoading } = useTokens();

  // Calculate platform stats
  const stats = tokens
    ? {
        totalTokens: tokens.length,
        totalLiquidity: tokens.reduce((sum: number, t: any) => sum + parseFloat(t.totalRaised || '0'), 0),
      }
    : { totalTokens: 0, totalLiquidity: 0 };

  const filters = ['Trending', 'Terminal', 'New', 'Graduated'];

  return (
    <div className="space-y-8">
      {/* Hero Section */}
      <div className="text-center py-12 space-y-4">
        <h1 className="text-5xl md:text-7xl font-black tracking-tighter bg-gradient-to-r from-white via-purple-200 to-purple-400 bg-clip-text text-transparent">
          Launch in seconds.<br/>Trade in real-time.
        </h1>
        <p className="text-gray-400 text-lg max-w-2xl mx-auto">
          The first fair-launch platform powered by Linera Microchains. No gas wars. Instant finality. Zero upfront cost.
        </p>

        {/* Platform Stats */}
        <div className="flex justify-center gap-8 pt-4 text-sm font-mono text-gray-500">
          <div className="flex flex-col items-center">
            <span className="text-white font-bold text-xl">
              {stats.totalTokens.toLocaleString()}
            </span>
            <span>TOKENS CREATED</span>
          </div>
          <div className="flex flex-col items-center">
            <span className="text-purple-400 font-bold text-xl">
              ${(stats.totalLiquidity / 1000).toFixed(1)}K
            </span>
            <span>TOTAL LIQUIDITY</span>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-4 overflow-x-auto pb-2 scrollbar-hide">
        {filters.map((filter) => (
          <button
            key={filter}
            onClick={() => setActiveFilter(filter)}
            className={`px-4 py-1.5 rounded-full text-sm font-medium whitespace-nowrap border transition-all ${
              activeFilter === filter
                ? 'bg-white text-black border-white'
                : 'bg-transparent text-gray-400 border-gray-800 hover:border-gray-600'
            }`}
          >
            {filter}
          </button>
        ))}
      </div>

      {/* Token Grid */}
      {isLoading ? (
        <div className="py-16 text-center">
          <LoadingSpinner size="xl" />
          <p className="text-gray-500 mt-4">Loading tokens...</p>
        </div>
      ) : tokens && tokens.length > 0 ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {tokens.map((token: any) => {
            const progress = (parseFloat(token.totalRaised || '0') / parseFloat(token.curveConfig?.targetRaise || '1')) * 100;

            return (
              <div
                key={token.tokenId}
                className="group bg-[#16171D] hover:bg-[#1E1F26] border border-white/5 hover:border-purple-500/30 rounded-xl p-4 cursor-pointer transition-all hover:-translate-y-1 relative overflow-hidden"
              >
                {progress > 90 && (
                  <div className="absolute top-0 right-0 bg-yellow-500/10 text-yellow-500 text-[10px] font-bold px-2 py-1 rounded-bl-lg border-l border-b border-yellow-500/20 flex items-center gap-1">
                    <Flame className="w-3 h-3 fill-current" /> HOT
                  </div>
                )}

                <div className="flex gap-4 items-start mb-3">
                  <img
                    src={token.metadata?.imageUrl || `https://ui-avatars.com/api/?name=${encodeURIComponent(token.metadata?.symbol || 'T')}&background=random&size=400`}
                    alt={token.metadata?.name || 'Token'}
                    className="w-14 h-14 rounded-lg object-cover bg-gray-800"
                  />
                  <div>
                    <h3 className="font-bold text-sm text-gray-200 leading-tight">
                      {token.metadata?.name || 'Unknown Token'}
                    </h3>
                    <div className="text-xs text-purple-400 font-mono mt-0.5">
                      Ticker: ${token.metadata?.symbol || 'UNKNOWN'}
                    </div>
                    <div className="text-[10px] text-gray-500 mt-1">
                      MCap: ${(parseFloat(token.totalRaised || '0') / 1000).toFixed(1)}k
                    </div>
                  </div>
                </div>

                <p className="text-xs text-gray-400 line-clamp-2 mb-4 h-8">
                  {token.metadata?.description || 'No description available'}
                </p>

                <div className="space-y-2">
                  <div className="flex justify-between text-[10px] font-mono text-gray-500">
                    <span>Bonding Curve</span>
                    <span className={progress >= 100 ? 'text-green-400' : 'text-purple-400'}>
                      {Math.floor(progress)}%
                    </span>
                  </div>
                  <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
                    <div
                      className={`h-full rounded-full ${
                        progress >= 100
                          ? 'bg-green-500'
                          : 'bg-gradient-to-r from-purple-600 to-pink-500'
                      }`}
                      style={{ width: `${Math.min(progress, 100)}%` }}
                    ></div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="py-16 text-center">
          <p className="text-gray-400">No tokens available yet. Be the first to create one!</p>
        </div>
      )}
    </div>
  );
}
