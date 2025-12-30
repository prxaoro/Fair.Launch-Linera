/**
 * Global state management using Zustand
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { WalletState } from '@/types';

interface AppState {
  // Wallet state
  wallet: WalletState;
  setWallet: (wallet: Partial<WalletState>) => void;
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;

  // UI state
  isSidebarOpen: boolean;
  toggleSidebar: () => void;

  // Theme state
  theme: 'light' | 'dark';
  toggleTheme: () => void;
}

export const useStore = create<AppState>()(
  persist(
    (set) => ({
      // Initial wallet state
      wallet: {
        account: null,
        isConnected: false,
      },

      setWallet: (walletUpdate) => {
        set((state) => ({
          wallet: { ...state.wallet, ...walletUpdate },
        }));
      },

      connectWallet: async () => {
        try {
          // Import wallet utilities dynamically to avoid circular dependencies
          const { connectLineraWallet } = await import('@/lib/wallet-utils');

          const account = await connectLineraWallet();

          set({
            wallet: {
              account,
              isConnected: true,
            },
          });

          console.log('Wallet connected:', account);
        } catch (error) {
          console.error('Failed to connect wallet:', error);
          throw new Error('Failed to connect wallet');
        }
      },

      disconnectWallet: () => {
        set({
          wallet: {
            account: null,
            isConnected: false,
          },
        });
      },

      // UI state
      isSidebarOpen: true,
      toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),

      // Theme state
      theme: 'dark',
      toggleTheme: () => {
        set((state) => ({
          theme: state.theme === 'light' ? 'dark' : 'light',
        }));
      },
    }),
    {
      name: 'fair-launch-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        wallet: state.wallet,
        theme: state.theme,
      }),
    }
  )
);
