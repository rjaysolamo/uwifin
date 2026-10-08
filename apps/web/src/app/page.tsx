'use client';

import { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';
import { apiClient, type Balance, type Wallet } from '@/lib/api';

export default function DashboardPage() {
  const router = useRouter();
  const { user, token, logout } = useAuth();
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [selectedWalletId, setSelectedWalletId] = useState('');
  const [balances, setBalances] = useState<Balance[]>([]);
  const [walletAddress, setWalletAddress] = useState('');
  const [network, setNetwork] = useState('base');
  const [walletType, setWalletType] = useState('smart_account');
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [asset, setAsset] = useState('USDC');
  const [statusMessage, setStatusMessage] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!user || !token) {
      router.replace('/login');
      return;
    }

    const loadWallets = async () => {
      try {
        const results = await apiClient.getWallets(token);
        setWallets(results);
        if (results.length > 0) {
          setSelectedWalletId(results[0].id);
        }
      } catch {
        setStatusMessage('Unable to load wallets right now.');
      }
    };

    loadWallets();
  }, [user, token, router]);

  useEffect(() => {
    if (!selectedWalletId || !token) {
      setBalances([]);
      return;
    }

    const loadBalances = async () => {
      try {
        const results = await apiClient.getWalletBalances(token, selectedWalletId);
        setBalances(results);
      } catch {
        setBalances([]);
      }
    };

    loadBalances();
  }, [selectedWalletId, token]);

  const createWallet = async () => {
    if (!token || !walletAddress.trim()) {
      setStatusMessage('Wallet address is required.');
      return;
    }

    try {
      setLoading(true);
      const created = await apiClient.createWallet(token, {
        address: walletAddress,
        network,
        wallet_type: walletType,
      });
      setWallets((current) => [created, ...current]);
      setSelectedWalletId(created.id);
      setWalletAddress('');
      setStatusMessage(`Wallet created for ${created.address}`);
    } catch (error) {
      setStatusMessage(error instanceof Error ? error.message : 'Wallet creation failed');
    } finally {
      setLoading(false);
    }
  };

  const sendTransfer = async () => {
    if (!token || !selectedWalletId || !recipient.trim() || !amount.trim()) {
      setStatusMessage('Please complete the transfer form.');
      return;
    }

    try {
      setLoading(true);
      const result = await apiClient.createTransaction(token, {
        wallet_id: selectedWalletId,
        network,
        asset,
        recipient,
        amount,
        idempotency_key: `ux-${Date.now()}`,
      });
      setStatusMessage(`Transfer started successfully. ID: ${result.id}`);
      setRecipient('');
      setAmount('');
    } catch (error) {
      setStatusMessage(error instanceof Error ? error.message : 'Transfer failed');
    } finally {
      setLoading(false);
    }
  };

  const baseStyles = useMemo(
    () => ({
      page: {
        minHeight: '100vh',
        background: '#f8fafc',
        padding: '32px 20px',
      } as const,
      shell: {
        maxWidth: 1100,
        margin: '0 auto',
        display: 'grid',
        gap: 24,
      } as const,
      topbar: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        gap: 12,
      } as const,
      card: {
        background: '#ffffff',
        borderRadius: 16,
        padding: 20,
        boxShadow: '0 8px 24px rgba(15, 23, 42, 0.06)',
      } as const,
      row: { display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' } as const,
      button: {
        borderRadius: 12,
        padding: '10px 14px',
        background: '#0066cc',
        color: '#fff',
        fontWeight: 700,
      } as const,
      secondary: {
        borderRadius: 12,
        padding: '10px 14px',
        background: '#e2e8f0',
        color: '#0f172a',
        fontWeight: 700,
      } as const,
      input: {
        border: '1px solid #dbe2ea',
        borderRadius: 12,
        padding: '10px 12px',
        minWidth: 140,
      } as const,
      label: { display: 'flex', flexDirection: 'column', gap: 8, fontSize: 13, color: '#475569' } as const,
      grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: 24 } as const,
      walletList: { display: 'grid', gap: 12 } as const,
      walletItem: {
        border: '1px solid #e2e8f0',
        borderRadius: 12,
        padding: 12,
        background: '#f8fafc',
      } as const,
      status: { color: '#0f172a', background: '#ecfeff', borderRadius: 10, padding: '10px 12px' } as const,
    }),
    [],
  );

  if (!user || !token) {
    return null;
  }

  return (
    <main style={baseStyles.page}>
      <div style={baseStyles.shell}>
        <div style={baseStyles.topbar}>
          <div>
            <h1 style={{ margin: 0, fontSize: 30, color: '#0f172a' }}>Dashboard</h1>
            <p style={{ margin: '6px 0 0', color: '#475569' }}>Signed in as {user.email}</p>
          </div>

          <div style={baseStyles.row}>
            <button type="button" style={baseStyles.secondary} onClick={() => router.push('/')}>
              Home
            </button>
            <button type="button" style={baseStyles.secondary} onClick={logout}>
              Logout
            </button>
          </div>
        </div>

        {statusMessage ? <div style={baseStyles.status}>{statusMessage}</div> : null}

        <section style={baseStyles.grid}>
          <div style={baseStyles.card}>
            <h2 style={{ marginTop: 0 }}>Wallets</h2>
            <div style={baseStyles.walletList}>
              {wallets.length === 0 ? (
                <p style={{ margin: 0, color: '#475569' }}>No wallets yet.</p>
              ) : (
                wallets.map((wallet) => (
                  <button
                    key={wallet.id}
                    type="button"
                    onClick={() => setSelectedWalletId(wallet.id)}
                    style={{
                      ...baseStyles.walletItem,
                      borderColor: selectedWalletId === wallet.id ? '#0066cc' : '#e2e8f0',
                      background: selectedWalletId === wallet.id ? '#eff6ff' : '#f8fafc',
                      textAlign: 'left',
                      cursor: 'pointer',
                    }}
                  >
                    <div style={{ fontWeight: 700 }}>{wallet.address}</div>
                    <div style={{ fontSize: 12, color: '#475569' }}>
                      {wallet.network} • {wallet.wallet_type}
                    </div>
                  </button>
                ))
              )}
            </div>

            <div style={{ marginTop: 20, display: 'grid', gap: 12 }}>
              <label style={baseStyles.label}>
                Wallet address
                <input value={walletAddress} onChange={(e) => setWalletAddress(e.target.value)} style={baseStyles.input} placeholder="0x..." />
              </label>
              <div style={baseStyles.row}>
                <label style={baseStyles.label}>
                  Network
                  <select value={network} onChange={(e) => setNetwork(e.target.value)} style={baseStyles.input}>
                    <option value="base">base</option>
                    <option value="ethereum">ethereum</option>
                    <option value="polygon">polygon</option>
                  </select>
                </label>
                <label style={baseStyles.label}>
                  Type
                  <select value={walletType} onChange={(e) => setWalletType(e.target.value)} style={baseStyles.input}>
                    <option value="smart_account">smart_account</option>
                    <option value="external">external</option>
                    <option value="embedded">embedded</option>
                  </select>
                </label>
              </div>
              <button type="button" style={baseStyles.button} onClick={createWallet} disabled={loading}>
                {loading ? 'Creating...' : 'Add wallet'}
              </button>
            </div>
          </div>

          <div style={baseStyles.card}>
            <h2 style={{ marginTop: 0 }}>Balances</h2>
            {balances.length === 0 ? (
              <p style={{ color: '#475569' }}>No asset balances available.</p>
            ) : (
              <div style={{ display: 'grid', gap: 10 }}>
                {balances.map((item) => (
                  <div key={`${item.asset}-${item.network}`} style={{ border: '1px solid #e2e8f0', borderRadius: 12, padding: 12 }}>
                    <div style={{ fontWeight: 700 }}>{item.asset}</div>
                    <div style={{ fontSize: 13, color: '#475569' }}>{item.balance} • {item.network}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>

        <section style={baseStyles.card}>
          <h2 style={{ marginTop: 0 }}>Send transfer</h2>
          <div style={{ display: 'grid', gap: 16 }}>
            <div style={baseStyles.row}>
              <label style={baseStyles.label}>
                Asset
                <select value={asset} onChange={(e) => setAsset(e.target.value)} style={baseStyles.input}>
                  <option value="USDC">USDC</option>
                  <option value="ETH">ETH</option>
                  <option value="MATIC">MATIC</option>
                </select>
              </label>
              <label style={baseStyles.label}>
                Network
                <select value={network} onChange={(e) => setNetwork(e.target.value)} style={baseStyles.input}>
                  <option value="base">base</option>
                  <option value="ethereum">ethereum</option>
                  <option value="polygon">polygon</option>
                </select>
              </label>
            </div>

            <label style={baseStyles.label}>
              Recipient wallet
              <input value={recipient} onChange={(e) => setRecipient(e.target.value)} style={baseStyles.input} placeholder="0x..." />
            </label>

            <label style={baseStyles.label}>
              Amount
              <input value={amount} onChange={(e) => setAmount(e.target.value)} style={baseStyles.input} placeholder="10.00" />
            </label>

            <button type="button" style={baseStyles.button} onClick={sendTransfer} disabled={loading}>
              {loading ? 'Submitting...' : 'Send funds'}
            </button>
          </div>
        </section>
      </div>
    </main>
  );
}
