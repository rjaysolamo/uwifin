'use client';
import { useState } from 'react';
import { useAuth } from '@/contexts/AuthContext';
import { useFinance } from '@/contexts/FinanceContext';
import { emailWalletEnabled, startEmailWallet, verifyEmailWallet } from '@/lib/email-wallet';
export function EmailWallet() {
  const { user } = useAuth();
  const { connect, capabilities } = useFinance();
  const [sent, setSent] = useState(false);
  const [code, setCode] = useState('');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  if (!emailWalletEnabled) return null;
  return <section className="card settings-card"><h2>Create or unlock your wallet</h2><p>Use a code sent to {user?.email}. No wallet extension is needed. Your wallet provider manages signing securely.</p><form className="stacked-form" onSubmit={async event => {
    event.preventDefault(); if (!user) return; setBusy(true); setMessage('');
    try { if (!sent) { await startEmailWallet(user.email); setSent(true); } else { await verifyEmailWallet(code); await connect('email'); setSent(false); setCode(''); setMessage('Your wallet is ready.'); } }
    catch(err) { setMessage(err instanceof Error ? err.message : 'Wallet service unavailable.'); }
    finally { setBusy(false); }
  }}>{sent && <><label htmlFor="wallet-code">Email code</label><input id="wallet-code" className="input" inputMode="numeric" autoComplete="one-time-code" value={code} onChange={event=>setCode(event.target.value)} minLength={6} maxLength={12} required/></>}<button className="button primary" disabled={busy || !capabilities?.wallet_enabled}>{busy ? 'Please wait…' : sent ? 'Verify and connect wallet' : 'Send email code'}</button>{sent && <button type="button" className="button secondary" disabled={busy} onClick={()=>{setSent(false);setCode('');}}>Request a new code</button>}<p role="status">{message}</p></form></section>;
}
