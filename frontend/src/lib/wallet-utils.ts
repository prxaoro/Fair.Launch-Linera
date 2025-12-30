/**
 * Wallet and Account utilities for Linera integration
 */

import type { Account } from '@/types';

/**
 * Serialize Account to JSON string for GraphQL queries
 * Required format: {"chain_id":"...","owner":"..."}
 */
export function accountToJson(account: Account): string {
  return JSON.stringify({
    chainId: account.chainId,
    owner: account.owner,
  });
}

/**
 * Parse Account from JSON string
 */
export function jsonToAccount(json: string): Account {
  const parsed = JSON.parse(json);
  return {
    chainId: parsed.chain_id,
    owner: parsed.owner,
  };
}

/**
 * Format Account for display (shortened chain_id and owner)
 */
export function formatAccount(account: Account | null | undefined): string {
  if (!account || !account.chainId || !account.owner) {
    return 'Not Connected';
  }
  const chainShort = account.chainId.substring(0, 8);
  const ownerShort = account.owner.substring(0, 10);
  return `${chainShort}...${ownerShort}`;
}

/**
 * Validate Account structure
 */
export function isValidAccount(account: unknown): account is Account {
  if (!account || typeof account !== 'object') return false;
  const a = account as Record<string, unknown>;
  return (
    typeof a.chainId === 'string' &&
    typeof a.owner === 'string' &&
    a.chainId.length > 0 &&
    a.owner.length > 0
  );
}

/**
 * Mock wallet connection for development
 * TODO: Replace with actual Linera wallet SDK integration
 */
export async function connectLineraWallet(): Promise<Account> {
  // Check if Linera wallet extension is available
  if (typeof window !== 'undefined' && (window as any).linera) {
    try {
      const lineraWallet = (window as any).linera;
      const accounts = await lineraWallet.request({ method: 'linera_accounts' });

      if (accounts && accounts.length > 0) {
        return accounts[0];
      }
    } catch (error) {
      console.error('Linera wallet error:', error);
    }
  }

  // Fallback to mock account for development
  console.warn('Using mock wallet - please install Linera wallet extension for production');

  return {
    chainId: 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65',
    owner: 'User:' + Array(64).fill(0).map(() =>
      Math.floor(Math.random() * 16).toString(16)
    ).join(''),
  };
}

/**
 * Calculate creator fee from amount
 * @param amount - The total amount
 * @param feeBps - Fee in basis points (300 = 3%)
 * @returns Fee amount as string
 */
export function calculateCreatorFee(amount: string, feeBps: number): string {
  const amountNum = parseFloat(amount);
  if (isNaN(amountNum)) return '0';

  const fee = (amountNum * feeBps) / 10000;
  return fee.toFixed(6);
}

/**
 * Calculate net amount after creator fee
 * @param amount - The total amount
 * @param feeBps - Fee in basis points (300 = 3%)
 * @returns Net amount after fee as string
 */
export function calculateNetAmount(amount: string, feeBps: number): string {
  const amountNum = parseFloat(amount);
  if (isNaN(amountNum)) return '0';

  const fee = (amountNum * feeBps) / 10000;
  const net = amountNum - fee;
  return net.toFixed(6);
}

/**
 * Format basis points as percentage string
 * @param bps - Basis points (300 = 3%)
 * @returns Formatted percentage string (e.g., "3%")
 */
export function formatBasisPoints(bps: number): string {
  return `${(bps / 100).toFixed(2)}%`;
}
