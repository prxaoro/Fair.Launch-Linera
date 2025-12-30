/**
 * Token detail page - Pump.fun style with trading interface
 */

import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { ArrowLeft, TrendingUp, TrendingDown } from 'lucide-react';
import { useTokenDetail } from '@/hooks/useTokens';
import { BondingCurveChart } from '@/components/BondingCurveChart';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { formatNumber, formatRelativeTime } from '@/lib/utils';

export function TokenDetailPage() {
  const { tokenId } = useParams<{ tokenId: string }>();
  const [tradeMode, setTradeMode] = useState<'buy' | 'sell'>('buy');
  const [amount, setAmount] = useState('');

  const { data: token, isLoading, error } = useTokenDetail(tokenId!);

  if (isLoading) {
    return (
      <div className="py-16 text-center">
        <LoadingSpinner size="xl" />
        <p className="text-gray-500 mt-4">Loading token...</p>
      </div>
    );
  }

  if (error || !token) {
    return (
      <div className="max-w-2xl mx-auto py-16">
        <div className="bg-[#16171D] border border-white/5 rounded-xl p-8 text-center">
          <h3 className="text-xl font-bold mb-2">Token not found</h3>
          <p className="text-gray-400 mb-6">
            {error instanceof Error ? error.message : 'This token does not exist'}
          </p>
          <Link
            to="/"
            className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg text-white text-sm font-bold inline-block"
          >
            Back to Home
          </Link>
        </div>
      </div>
    );
  }

  // Calculate bonding curve progress
  const progress = (parseFloat(token.totalRaised) / parseFloat(token.curveConfig.targetRaise)) * 100;
  const currentSupply = parseFloat(token.currentSupply);
  const k = parseFloat(token.curveConfig.k);
  const scale = parseFloat(token.curveConfig.scale);
  const currentPrice = (k * Math.pow(currentSupply / scale, 2)) / 1e6;

  // Mock trades data (replace with real data from GraphQL)
  const recentTrades = [
    { type: 'buy', account: 'e0544b...a5a3c7', amount: '1,000', value: '25.00', time: '2m ago' },
    { type: 'sell', account: 'f1655c...b6b4d8', amount: '500', value: '12.50', time: '5m ago' },
    { type: 'buy', account: 'a2766d...c7c5e9', amount: '2,500', value: '62.50', time: '8m ago' },
  ];

  return (
    <div className="space-y-6">
      {/* Back Button */}
      <Link to="/" className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
        <ArrowLeft className="w-4 h-4" />
        <span className="text-sm">Back</span>
      </Link>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column - Token Info & Chart */}
        <div className="lg:col-span-2 space-y-6">
          {/* Token Header */}
          <div className="bg-[#16171D] border border-white/5 rounded-xl p-6">
            <div className="flex gap-4 items-start mb-4">
              <img
                src={token.metadata.imageUrl || `https://ui-avatars.com/api/?name=${encodeURIComponent(token.metadata.symbol)}&background=random&size=400`}
                alt={token.metadata.name}
                className="w-20 h-20 rounded-lg object-cover bg-gray-800"
              />
              <div className="flex-1">
                <div className="flex items-start justify-between">
                  <div>
                    <h1 className="text-2xl font-bold mb-1">{token.metadata.name}</h1>
                    <div className="text-purple-400 font-mono text-sm">Ticker: ${token.metadata.symbol}</div>
                  </div>
                  {token.isGraduated && (
                    <div className="px-3 py-1 bg-green-500/10 text-green-500 text-xs font-bold rounded-lg border border-green-500/20">
                      🎓 GRADUATED
                    </div>
                  )}
                </div>
                <p className="text-gray-400 text-sm mt-3">{token.metadata.description}</p>
              </div>
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 pt-4 border-t border-white/5">
              <div>
                <div className="text-xs text-gray-500 mb-1">Market Cap</div>
                <div className="text-white font-bold">${(parseFloat(token.totalRaised) / 1000).toFixed(1)}k</div>
              </div>
              <div>
                <div className="text-xs text-gray-500 mb-1">Price</div>
                <div className="text-white font-bold">${currentPrice.toFixed(6)}</div>
              </div>
              <div>
                <div className="text-xs text-gray-500 mb-1">Supply</div>
                <div className="text-white font-bold">{formatNumber(currentSupply, 0)}</div>
              </div>
              <div>
                <div className="text-xs text-gray-500 mb-1">Created</div>
                <div className="text-white font-bold">{formatRelativeTime(token.createdAt)}</div>
              </div>
            </div>
          </div>

          {/* Bonding Curve Chart */}
          <BondingCurveChart progress={progress} />

          {/* Trades Table */}
          <div className="bg-[#16171D] border border-white/5 rounded-xl p-6">
            <h3 className="text-lg font-bold mb-4">Recent Trades</h3>
            <div className="space-y-2">
              {recentTrades.map((trade, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between py-2 border-b border-white/5 last:border-0"
                >
                  <div className="flex items-center gap-3">
                    {trade.type === 'buy' ? (
                      <div className="w-6 h-6 rounded-full bg-green-500/10 flex items-center justify-center">
                        <TrendingUp className="w-3 h-3 text-green-500" />
                      </div>
                    ) : (
                      <div className="w-6 h-6 rounded-full bg-red-500/10 flex items-center justify-center">
                        <TrendingDown className="w-3 h-3 text-red-500" />
                      </div>
                    )}
                    <div>
                      <div className="text-sm font-mono text-gray-400">{trade.account}</div>
                      <div className="text-xs text-gray-500">{trade.time}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`text-sm font-bold ${trade.type === 'buy' ? 'text-green-500' : 'text-red-500'}`}>
                      {trade.type === 'buy' ? '+' : '-'}{trade.amount}
                    </div>
                    <div className="text-xs text-gray-500">${trade.value}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Right Column - Trading Panel */}
        <div className="lg:col-span-1">
          <div className="bg-[#16171D] border border-white/5 rounded-xl p-6 sticky top-24">
            <h3 className="text-lg font-bold mb-4">Trade ${token.metadata.symbol}</h3>

            {/* Buy/Sell Toggle */}
            <div className="grid grid-cols-2 gap-2 mb-6">
              <button
                onClick={() => setTradeMode('buy')}
                className={`py-2 rounded-lg text-sm font-bold transition-all ${
                  tradeMode === 'buy'
                    ? 'bg-green-600 text-white'
                    : 'bg-white/5 text-gray-400 hover:bg-white/10'
                }`}
              >
                Buy
              </button>
              <button
                onClick={() => setTradeMode('sell')}
                className={`py-2 rounded-lg text-sm font-bold transition-all ${
                  tradeMode === 'sell'
                    ? 'bg-red-600 text-white'
                    : 'bg-white/5 text-gray-400 hover:bg-white/10'
                }`}
              >
                Sell
              </button>
            </div>

            {/* Amount Input */}
            <div className="mb-4">
              <label className="text-xs text-gray-500 mb-2 block">Amount (TLIN)</label>
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                className="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
              />
            </div>

            {/* Info Grid */}
            <div className="space-y-2 mb-6 p-4 bg-white/5 rounded-lg">
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">You pay</span>
                <span className="text-white font-mono">{amount || '0.00'} TLIN</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">You receive</span>
                <span className="text-white font-mono">≈ 0 ${token.metadata.symbol}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">Price impact</span>
                <span className="text-yellow-500 font-mono">~0.0%</span>
              </div>
            </div>

            {/* Trade Button */}
            <button
              className={`w-full py-3 rounded-lg font-bold transition-all ${
                tradeMode === 'buy'
                  ? 'bg-green-600 hover:bg-green-700 text-white'
                  : 'bg-red-600 hover:bg-red-700 text-white'
              }`}
            >
              {tradeMode === 'buy' ? 'Buy' : 'Sell'} ${token.metadata.symbol}
            </button>

            {/* Bonding Curve Progress */}
            <div className="mt-6 pt-6 border-t border-white/5">
              <div className="flex justify-between text-xs text-gray-500 mb-2">
                <span>Bonding Curve Progress</span>
                <span className="text-purple-400">{Math.floor(progress)}%</span>
              </div>
              <div className="h-2 bg-gray-800 rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full ${
                    progress >= 100
                      ? 'bg-green-500'
                      : 'bg-gradient-to-r from-purple-600 to-pink-500'
                  }`}
                  style={{ width: `${Math.min(progress, 100)}%` }}
                ></div>
              </div>
              <div className="text-xs text-gray-500 mt-2">
                ${(parseFloat(token.totalRaised) / 1000).toFixed(1)}k / ${(parseFloat(token.curveConfig.targetRaise) / 1000).toFixed(1)}k raised
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
