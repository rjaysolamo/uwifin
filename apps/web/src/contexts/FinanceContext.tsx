'use client';

import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { DEMO_MODE, useAuth } from './AuthContext';
import { request, type Wallet, type Balance } from '@/lib/api';
import { DEMO_ADDRESS, initialTransfers, type Transfer } from '@/lib/demo';
import { parseAmount, validateAddress } from '@/lib/money';

type FinanceState = {
  balance: string; address: string; transfers: Transfer[]; loading: boolean; error: string;
  send: (recipient: string, amount: string, name: string, key: string) => Promise<Transfer>;
  refresh: () => Promise<void>; resetDemo: () => void;
};
const FinanceContext = createContext<FinanceState | null>(null);
export function FinanceProvider({ children }: { children: ReactNode }) {
  const { user } = useAuth();
  const [balance, setBalance] = useState(DEMO_MODE ? '1240520000' : '0');
  const [address, setAddress] = useState(DEMO_MODE ? DEMO_ADDRESS : '');
  const [transfers, setTransfers] = useState<Transfer[]>(DEMO_MODE ? initialTransfers : []);
  const [loading, setLoading] = useState(!DEMO_MODE);
  const [error, setError] = useState('');
  const mutationLock = useRef(false);
  const timers = useRef<ReturnType<typeof setTimeout>[]>([]);
  const persist = (nextBalance: string, nextTransfers: Transfer[]) => {
    localStorage.setItem('uwifin-demo-finances-v1', JSON.stringify({ balance: nextBalance, transfers: nextTransfers }));
  };
  useEffect(() => {
    if (!DEMO_MODE) return;
    try {
      const saved = JSON.parse(localStorage.getItem('uwifin-demo-finances-v1') || 'null');
      if (saved && /^\d{1,19}$/.test(saved.balance) && Array.isArray(saved.transfers) && saved.transfers.length <= 500 && saved.transfers.every((item: Transfer) =>
        typeof item.id === 'string' && typeof item.name === 'string' && typeof item.created_at === 'string' &&
        /^\d{1,19}$/.test(item.amount_atomic) && validateAddress(item.address) &&
        ['sent', 'received', 'bought'].includes(item.kind) && ['created', 'pending', 'confirmed', 'failed'].includes(item.status))) {
        setBalance(saved.balance); setTransfers(saved.transfers);
      }
    } catch { localStorage.removeItem('uwifin-demo-finances-v1'); }
    return () => timers.current.forEach(clearTimeout);
  }, []);

  const refresh = async () => {
    if (DEMO_MODE || !user) return;
    setLoading(true); setError('');
    try {
      const wallets = await request<Wallet[]>('/wallets');
      const wallet = wallets.find((item) => item.network === 'base-sepolia');
      if (wallet) {
        setAddress(wallet.address);
        const balances = await request<Balance[]>(`/wallets/${wallet.id}/balances`);
        const usdc = balances.find((item) => item.asset === 'USDC');
        setBalance(usdc && usdc.balance !== '0.000000' ? parseAmount(usdc.balance).toString() : '0');
      }
      const result = await request<{ transactions: Transfer[] }>('/transactions');
      setTransfers(result.transactions);
    } catch (err) { setError(err instanceof Error ? err.message : 'Unable to load your wallet.'); }
    finally { setLoading(false); }
  };
  useEffect(() => {
    if (!DEMO_MODE) void refresh();
    // Refresh on identity changes; manual refresh handles provider updates.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [user?.id]);

  const send = async (recipient: string, amount: string, name: string, key: string): Promise<Transfer> => {
    if (!validateAddress(recipient)) throw new Error('Enter a valid recipient wallet address.');
    if (recipient.toLowerCase() === address.toLowerCase()) throw new Error('Choose a different wallet from your own.');
    const atomic = parseAmount(amount);
    const existing = transfers.find((item) => item.idempotency_key === key);
    if (existing) return existing;
    if (mutationLock.current) throw new Error('A transfer is already being processed.');
    if (atomic > BigInt(balance)) throw new Error('You don’t have enough USDC for this transfer.');
    mutationLock.current = true;
    try {
      if (!DEMO_MODE) {
        // Sending stays gated until user signing and receipt verification are configured.
        throw new Error('Live transfers are not enabled yet. Connect the approved wallet signing provider first.');
      }
      const transfer: Transfer = {
        id: `demo-${crypto.randomUUID()}`, kind: 'sent', name: name || 'Wallet transfer', address: recipient,
        amount_atomic: atomic.toString(), status: 'pending', created_at: new Date().toISOString(), tx_hash: null, idempotency_key: key,
      };
      const nextBalance = (BigInt(balance) - atomic).toString();
      const nextTransfers = [transfer, ...transfers];
      persist(nextBalance, nextTransfers); setBalance(nextBalance); setTransfers(nextTransfers);
      // Demonstrate asynchronous settlement only in explicit demo mode.
      timers.current.push(setTimeout(() => {
        setTransfers((current) => {
          const settled = current.map((item) => item.id === transfer.id ? { ...item, status: 'confirmed' as const } : item);
          const stored = JSON.parse(localStorage.getItem('uwifin-demo-finances-v1') || '{}');
          persist(stored.balance || nextBalance, settled);
          return settled;
        });
      }, 7000));
      return transfer;
    } finally { mutationLock.current = false; }
  };
  const resetDemo = () => {
    if (!DEMO_MODE) return;
    timers.current.forEach(clearTimeout); timers.current = [];
    persist('1240520000', initialTransfers); setBalance('1240520000'); setTransfers(initialTransfers);
  };
  return <FinanceContext.Provider value={{ balance, address, transfers, loading, error, send, refresh, resetDemo }}>
    {children}
  </FinanceContext.Provider>;
}
export function useFinance() {
  const value = useContext(FinanceContext);
  if (!value) throw new Error('FinanceProvider is required.');
  return value;
}
