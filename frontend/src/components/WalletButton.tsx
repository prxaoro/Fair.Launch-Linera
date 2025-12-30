/**
 * Wallet connection button with Linera integration
 */

import { useState } from 'react';
import { useStore } from '@/lib/store';
import { Button } from './Button';
import { formatAccount } from '@/lib/wallet-utils';
import toast from 'react-hot-toast';

export function WalletButton() {
  const { wallet, connectWallet, disconnectWallet } = useStore();
  const [isConnecting, setIsConnecting] = useState(false);

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      await connectWallet();
      toast.success('Wallet connected successfully!');
    } catch (error) {
      console.error('Failed to connect:', error);
      toast.error('Failed to connect wallet. Please try again.');
    } finally {
      setIsConnecting(false);
    }
  };

  if (wallet.isConnected && wallet.account) {
    return (
      <div className="flex items-center gap-2">
        <div className="px-3 py-2 bg-primary-100 dark:bg-primary-900 rounded-lg text-sm font-medium text-primary-700 dark:text-primary-300">
          {formatAccount(wallet.account)}
        </div>
        <Button variant="ghost" size="sm" onClick={disconnectWallet}>
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Button
      variant="primary"
      onClick={handleConnect}
      isLoading={isConnecting}
      disabled={isConnecting}
    >
      Connect Wallet
    </Button>
  );
}
