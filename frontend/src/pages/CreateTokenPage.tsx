/**
 * Create token page - Pump.fun style with live preview
 */

import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Upload, Sparkles } from 'lucide-react';
import { useCreateToken } from '@/hooks/useTokens';
import { useStore } from '@/lib/store';
import { isValidSymbol, isValidTokenName, isValidUrl } from '@/lib/utils';
import toast from 'react-hot-toast';

export function CreateTokenPage() {
  const navigate = useNavigate();
  const wallet = useStore((state) => state.wallet);
  const createTokenMutation = useCreateToken();

  const [formData, setFormData] = useState({
    name: '',
    symbol: '',
    description: '',
    imageUrl: '',
    initialSupply: '1000000',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!isValidTokenName(formData.name)) {
      newErrors.name = 'Name must be 2-50 characters';
    }

    if (!isValidSymbol(formData.symbol)) {
      newErrors.symbol = 'Symbol must be 2-10 uppercase alphanumeric characters';
    }

    if (formData.description.length < 10) {
      newErrors.description = 'Description must be at least 10 characters';
    }

    if (formData.imageUrl && !isValidUrl(formData.imageUrl)) {
      newErrors.imageUrl = 'Invalid URL format';
    }

    const supply = parseFloat(formData.initialSupply);
    if (isNaN(supply) || supply <= 0) {
      newErrors.initialSupply = 'Initial supply must be greater than 0';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!wallet.isConnected) {
      toast.error('Please connect your wallet first');
      return;
    }

    if (!validateForm()) {
      toast.error('Please fix validation errors');
      return;
    }

    try {
      await createTokenMutation.mutateAsync(formData);
      // Navigate to home page after token creation (token ID will be available after confirmation)
      navigate('/');
      toast.success('Token creation submitted! Waiting for confirmation...');
    } catch (error) {
      console.error('Failed to create token:', error);
    }
  };

  const handleChange = (
    field: keyof typeof formData,
    value: string
  ) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    // Clear error for this field
    if (errors[field]) {
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      {/* Back Button */}
      <Link to="/" className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
        <ArrowLeft className="w-4 h-4" />
        <span className="text-sm">Back</span>
      </Link>

      {/* Header */}
      <div className="text-center space-y-2">
        <h1 className="text-4xl md:text-5xl font-black bg-gradient-to-r from-white via-purple-200 to-purple-400 bg-clip-text text-transparent">
          Create a new coin
        </h1>
        <p className="text-gray-400">
          Launch your token in seconds. Fair, transparent, instant.
        </p>
      </div>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Column - Form */}
        <div className="bg-[#16171D] border border-white/5 rounded-xl p-6">
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Token Name */}
            <div>
              <label className="text-xs text-gray-500 mb-2 block font-medium">Name</label>
              <input
                type="text"
                placeholder="My Awesome Token"
                value={formData.name}
                onChange={(e) => handleChange('name', e.target.value)}
                className="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                required
              />
              {errors.name && (
                <p className="mt-1.5 text-xs text-red-400">{errors.name}</p>
              )}
            </div>

            {/* Token Symbol */}
            <div>
              <label className="text-xs text-gray-500 mb-2 block font-medium">Ticker</label>
              <input
                type="text"
                placeholder="MAT"
                value={formData.symbol}
                onChange={(e) => handleChange('symbol', e.target.value.toUpperCase())}
                className="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 uppercase"
                maxLength={10}
                required
              />
              {errors.symbol && (
                <p className="mt-1.5 text-xs text-red-400">{errors.symbol}</p>
              )}
              <p className="mt-1.5 text-xs text-gray-500">2-10 uppercase letters</p>
            </div>

            {/* Description */}
            <div>
              <label className="text-xs text-gray-500 mb-2 block font-medium">Description</label>
              <textarea
                placeholder="Tell us about your token..."
                value={formData.description}
                onChange={(e) => handleChange('description', e.target.value)}
                rows={4}
                className="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 resize-none"
                required
              />
              {errors.description && (
                <p className="mt-1.5 text-xs text-red-400">{errors.description}</p>
              )}
            </div>

            {/* Image Upload */}
            <div>
              <label className="text-xs text-gray-500 mb-2 block font-medium">Image (optional)</label>
              <div className="flex gap-3">
                <input
                  type="text"
                  placeholder="https://example.com/image.png"
                  value={formData.imageUrl}
                  onChange={(e) => handleChange('imageUrl', e.target.value)}
                  className="flex-1 bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                />
                <button
                  type="button"
                  className="px-4 py-3 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 transition-colors"
                >
                  <Upload className="w-4 h-4 text-gray-400" />
                </button>
              </div>
              {errors.imageUrl && (
                <p className="mt-1.5 text-xs text-red-400">{errors.imageUrl}</p>
              )}
            </div>

            {/* Info Box */}
            <div className="p-4 bg-purple-500/10 rounded-lg border border-purple-500/20">
              <div className="flex items-start gap-2 mb-2">
                <Sparkles className="w-4 h-4 text-purple-400 flex-shrink-0 mt-0.5" />
                <h4 className="font-bold text-purple-300 text-sm">Fair Launch Guaranteed</h4>
              </div>
              <ul className="text-xs text-gray-400 space-y-1 ml-6">
                <li>• No presales or VC allocations</li>
                <li>• Instant liquidity via bonding curve</li>
                <li>• Transparent pricing for everyone</li>
                <li>• Automatic DEX listing at graduation</li>
              </ul>
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              disabled={!wallet.isConnected || createTokenMutation.isPending}
              className="w-full py-3 rounded-lg bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white font-bold transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {!wallet.isConnected
                ? 'Connect Wallet to Create'
                : createTokenMutation.isPending
                ? 'Creating...'
                : 'Create Token'}
            </button>
          </form>
        </div>

        {/* Right Column - Live Preview */}
        <div className="space-y-6">
          <div className="bg-[#16171D] border border-white/5 rounded-xl p-6">
            <h3 className="text-sm font-bold text-gray-500 mb-4">LIVE PREVIEW</h3>

            {/* Token Card Preview */}
            <div className="bg-[#0F1014] border border-white/5 rounded-xl p-4">
              <div className="flex gap-4 items-start mb-3">
                <div className="w-14 h-14 rounded-lg bg-gray-800 overflow-hidden flex-shrink-0">
                  {formData.imageUrl && !errors.imageUrl ? (
                    <img
                      src={formData.imageUrl}
                      alt="Preview"
                      className="w-full h-full object-cover"
                      onError={(e) => {
                        e.currentTarget.src = `https://ui-avatars.com/api/?name=${encodeURIComponent(formData.symbol || 'T')}&background=random&size=400`;
                      }}
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-gray-600 text-2xl font-bold">
                      {formData.symbol.charAt(0) || '?'}
                    </div>
                  )}
                </div>
                <div className="flex-1">
                  <h3 className="font-bold text-sm text-gray-200 leading-tight">
                    {formData.name || 'Token Name'}
                  </h3>
                  <div className="text-xs text-purple-400 font-mono mt-0.5">
                    Ticker: ${formData.symbol || 'SYMBOL'}
                  </div>
                  <div className="text-[10px] text-gray-500 mt-1">
                    MCap: $0.0k
                  </div>
                </div>
              </div>

              <p className="text-xs text-gray-400 line-clamp-2 mb-4 h-8">
                {formData.description || 'Token description will appear here...'}
              </p>

              <div className="space-y-2">
                <div className="flex justify-between text-[10px] font-mono text-gray-500">
                  <span>Bonding Curve</span>
                  <span className="text-purple-400">0%</span>
                </div>
                <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
                  <div className="h-full w-0 rounded-full bg-gradient-to-r from-purple-600 to-pink-500"></div>
                </div>
              </div>
            </div>
          </div>

          {/* Tips */}
          <div className="bg-[#16171D] border border-white/5 rounded-xl p-6">
            <h3 className="text-sm font-bold mb-3">💡 Pro Tips</h3>
            <ul className="text-xs text-gray-400 space-y-2">
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Use a catchy, memorable name and ticker</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Write a clear description explaining your token's purpose</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Upload a unique image (recommended 400x400px)</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Share your token link on social media for traction</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
