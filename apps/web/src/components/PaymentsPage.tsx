'use client';
import { useEffect, useRef, useState, type FormEvent } from 'react';
import { useFinance } from '@/contexts/FinanceContext';
import { request } from '@/lib/api';
import { decimalAmount } from '@/lib/money';
type Payment = {
  id: string;
  status: string;
  fiat_currency: string;
  fiat_amount_minor: string;
  crypto_amount_atomic: string;
};
type Session = { client_secret: string; publishable_key: string };
function StripePurchase({ session }: { session: Session }) {
  const container = useRef<HTMLDivElement>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    let disposed = false;
    const target = container.current;
    void (async () => {
      try {
        const { loadStripeOnramp } = await import('@stripe/crypto/pure');
        const stripe = await loadStripeOnramp(session.publishable_key);
        if (!disposed && target) {
          if (!stripe) throw new Error('Stripe could not be loaded.');
          stripe
            .createSession({ clientSecret: session.client_secret, appearance: { theme: 'light' } })
            .mount(target);
        }
      } catch {
        if (!disposed)
          setError('Stripe could not be loaded. Check your connection and retry this purchase.');
      }
    })();
    return () => {
      disposed = true;
      target?.replaceChildren();
    };
  }, [session]);
  return (
    <>
      {error && (
        <p role="alert" className="field-error">
          {error}
        </p>
      )}
      <div ref={container} />
    </>
  );
}
export function PaymentsPage() {
  const { wallet, capabilities, networkLabel } = useFinance();
  const [amount, setAmount] = useState('100');
  const [currency, setCurrency] = useState('USD');
  const [session, setSession] = useState<Session | null>(null);
  const [payments, setPayments] = useState<Payment[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const key = useRef<{ fingerprint: string; value: string } | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    const refresh = () =>
      request<{ payments: Payment[] }>('/payments', { signal: controller.signal })
        .then((result) => setPayments(result.payments))
        .catch(() => {
          if (!controller.signal.aborted) setError('Payment history is temporarily unavailable.');
        });
    void refresh();
    const timer = setInterval(() => {
      if (!document.hidden) void refresh();
    }, 15_000);
    return () => {
      controller.abort();
      clearInterval(timer);
    };
  }, []);
  const buy = async (event: FormEvent) => {
    event.preventDefault();
    if (!wallet || busy) return;
    setBusy(true);
    setError('');
    try {
      if (!/^\d{1,5}(\.\d{1,2})?$/.test(amount) || Number(amount) <= 0 || Number(amount) > 10000)
        throw new Error('Enter an amount from 0.01 to 10,000 with at most two decimal places.');
      const fingerprint = `${wallet.id}:${currency}:${amount}`;
      if (key.current?.fingerprint !== fingerprint)
        key.current = { fingerprint, value: crypto.randomUUID() };
      setSession(
        await request<Session>('/payments', {
          method: 'POST',
          headers: { 'Idempotency-Key': key.current.value },
          body: JSON.stringify({ wallet_id: wallet.id, amount, currency }),
        }),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to start purchase.');
    } finally {
      setBusy(false);
    }
  };
  return (
    <>
      <section className="page-heading">
        <div>
          <h1>Add money.</h1>
          <p>Buy USDC through Stripe where supported.</p>
        </div>
      </section>
      <div className="flow-layout">
        <section className="card flow-card">
          <h2>Stripe on-ramp</h2>
          <p>
            Destination: {wallet?.address || 'Connect a wallet first'} · {networkLabel}
          </p>
          <p>Mode: {capabilities?.stripe_mode || 'Unavailable'}</p>
          {!capabilities?.onramp_enabled && (
            <p className="info-note">
              Card purchases are not available for this wallet. Stripe approval and a supported
              mainnet destination are required.
            </p>
          )}
          {error && (
            <p role="alert" className="field-error">
              {error}
            </p>
          )}
          {session ? (
            <StripePurchase session={session} />
          ) : (
            <form className="stacked-form" onSubmit={buy}>
              <label htmlFor="buy-amount">Purchase amount</label>
              <input
                id="buy-amount"
                className="input"
                inputMode="decimal"
                value={amount}
                onChange={(event) => setAmount(event.target.value)}
                required
              />
              <label htmlFor="currency">Currency</label>
              <select
                id="currency"
                className="input"
                value={currency}
                onChange={(event) => setCurrency(event.target.value)}
              >
                <option>USD</option>
                <option>EUR</option>
              </select>
              <p>
                Stripe displays the final quote, fees, eligibility checks, and amount before you
                pay. No card details are collected by UwiFin.
              </p>
              <button
                className="button primary"
                disabled={busy || !wallet || !capabilities?.onramp_enabled}
              >
                {busy ? 'Opening Stripe…' : 'Continue to Stripe'}
              </button>
            </form>
          )}
        </section>
        <aside className="flow-side">
          <h2>Cash out</h2>
          <p>
            Fiat cash-out is not available. Stripe stablecoin payouts send stablecoins to wallets
            and do not provide a generic wallet-to-PHP bank or GCash off-ramp.
          </p>
          <p>
            A supported payout provider and destination corridor must be configured before cash-out
            can be offered.
          </p>
        </aside>
      </div>
      <section className="card wallet-assets">
        <h2>Payment history</h2>
        <p>
          Status is verified by the backend with Stripe. Browser redirects do not confirm payment.
        </p>
        {payments.length ? (
          payments.map((payment) => (
            <div className="wallet-asset-row" key={payment.id}>
              <span>
                {payment.fiat_currency} {(BigInt(payment.fiat_amount_minor) / 100n).toString()}.
                {(BigInt(payment.fiat_amount_minor) % 100n).toString().padStart(2, '0')}
              </span>
              <span>{decimalAmount(payment.crypto_amount_atomic)} USDC</span>
              <strong>{payment.status}</strong>
            </div>
          ))
        ) : (
          <p>No payments yet.</p>
        )}
      </section>
    </>
  );
}
