'use client';
import Link from 'next/link';
import { useRef, useState, type FormEvent } from 'react';
import { ArrowLeft, ArrowRight, ArrowUpRight, Check, CheckCircle2, ChevronDown, LockKeyhole, ShieldCheck, Wallet, Zap } from 'lucide-react';
import { useFinance } from '@/contexts/FinanceContext';
import { DEMO_MODE } from '@/contexts/AuthContext';
import { recipients, type Transfer } from '@/lib/demo';
import { decimalAmount, money, parseAmount, shortAddress, validateAddress } from '@/lib/money';
import { UsdcIcon } from './UI';

export function SendPage({ initialRecipient = '', initialAmount = '' }: { initialRecipient?: string; initialAmount?: string }) {
  const { balance, address, send, transfers } = useFinance();
  const [recipient, setRecipient] = useState(initialRecipient === 'new' ? '' : initialRecipient);
  const [amount, setAmount] = useState(initialAmount);
  const [step, setStep] = useState(1);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Transfer | null>(null);
  const key = useRef('');
  const person = recipients.find((item) => item.address.toLowerCase() === recipient.toLowerCase());
  const validate = (event: FormEvent) => {
    event.preventDefault(); setError('');
    try {
      if (!validateAddress(recipient)) throw new Error('Enter a valid, nonzero EVM wallet address (0x followed by 40 characters).');
      if (address.toLowerCase() === recipient.toLowerCase()) throw new Error('Please choose a wallet other than your own.');
      const atomic = parseAmount(amount);
      if (atomic > BigInt(balance)) throw new Error('You don’t have enough USDC for this amount.');
      key.current = crypto.randomUUID(); setStep(2);
    } catch (err) { setError(err instanceof Error ? err.message : 'Check your transfer details.'); }
  };
  const confirm = async () => {
    if (busy) return;
    setBusy(true); setError('');
    try { setResult(await send(recipient, amount, person?.name || 'Wallet transfer', key.current)); setStep(3); }
    catch (err) { setError(err instanceof Error ? err.message : 'Your transfer could not be submitted.'); }
    finally { setBusy(false); }
  };
  const currentResult = transfers.find((item) => item.id === result?.id) || result;
  return <>
    <section className="page-heading"><div><span className="eyebrow">A LITTLE LOVE GOES A LONG WAY</span><h1>Send money<span className="greeting-dot">.</span></h1><p>Simple, thoughtful transfers to the people who matter.</p></div>{DEMO_MODE && <span className="demo-pill">Demo transfer</span>}</section>
    <div className="flow-layout"><section className="card flow-card"><div className="stepper">{['Transfer details', 'Review & confirm', 'Transfer status'].map((label, index) => <div className={step >= index + 1 ? 'active' : ''} key={label}><span>{step > index + 1 ? <Check size={12}/> : index + 1}</span><small>{label}</small></div>)}</div>
      {step === 1 && <form className="stacked-form" onSubmit={validate}><h2>Where would you like to send?</h2><label htmlFor="recipient">Recipient wallet address</label><input id="recipient" className="input" placeholder="0x…" value={recipient} onChange={(event) => setRecipient(event.target.value.trim())} autoComplete="off" required/><small className="input-help">Send USDC to a wallet on {DEMO_MODE ? 'Base' : 'Base Sepolia'}. Always double-check the address.</small><div className="recipient-chips">{recipients.map((item) => <button type="button" className={person?.address === item.address ? 'selected' : ''} onClick={() => setRecipient(item.address)} key={item.address}><span className={`person-avatar tiny ${item.color}`}>{item.initials}</span>{item.relation}</button>)}</div><label htmlFor="send-amount">Amount to send</label><div className="amount-input large"><span>$</span><input id="send-amount" inputMode="decimal" placeholder="0.00" value={amount} onChange={(event) => setAmount(event.target.value)} required/><span className="input-currency"><UsdcIcon small/>USDC</span></div><div className="available-balance"><span>Available: {money(balance)} USDC</span><button type="button" onClick={() => setAmount(decimalAmount(balance))}>Use max</button></div><label htmlFor="send-network">Network</label><div className="select-wrap"><select id="send-network"><option>Base{DEMO_MODE ? ' · Demo' : ' Sepolia · Testnet'}</option></select><ChevronDown size={16}/></div><div className="info-note"><Zap size={18}/><p>{DEMO_MODE ? 'Network fees are sponsored in this demo. Real sponsorship depends on eligibility.' : 'Gas sponsorship is subject to the configured policy and your transfer’s eligibility.'}</p></div>{error && <p className="field-error" role="alert">{error}</p>}<button className="button primary full-width">Review transfer <ArrowRight size={17}/></button></form>}
      {step === 2 && <div className="review-transfer"><span className="large-icon"><ArrowUpRight size={26}/></span><h2>One last look.</h2><p>Make sure everything is right before you send.</p><div className="review-amount">{money(parseAmount(amount))}<span>USDC</span></div><dl className="detail-list"><div><dt>To</dt><dd>{person?.name || 'External wallet'}</dd></div><div className="detail-address"><dt>Recipient address</dt><dd><code>{recipient}</code></dd></div><div><dt>From your wallet</dt><dd>{shortAddress(address)}</dd></div><div><dt>Network</dt><dd>Base{DEMO_MODE ? ' · Demo' : ' Sepolia'}</dd></div><div><dt>Network fee</dt><dd className="positive">{DEMO_MODE ? '$0.00 · Sponsored (demo)' : 'Not available yet'}</dd></div><div className="detail-total"><dt>Total you send</dt><dd>{money(parseAmount(amount))} USDC</dd></div></dl><div className="info-note"><ShieldCheck size={18}/><p>{DEMO_MODE ? 'This transfer uses demo funds. No real money will be moved.' : 'Blockchain transfers cannot be reversed. Live signing is required before any funds can move.'}</p></div>{error && <p className="field-error" role="alert">{error}</p>}<button className="button primary full-width" disabled={busy} onClick={() => void confirm()}>{busy ? 'Processing…' : DEMO_MODE ? 'Confirm demo transfer' : 'Continue to wallet signing'}<ArrowRight size={17}/></button><button className="button text-button full-width" disabled={busy} onClick={() => { setStep(1); setError(''); }}><ArrowLeft size={15}/>Edit details</button></div>}
      {step === 3 && currentResult && <div className="success-panel"><span className="success-icon"><CheckCircle2 size={37}/></span><span className="eyebrow">A LITTLE CLOSER TO HOME</span><h2>{currentResult.status === 'confirmed' ? 'Transfer confirmed.' : 'Your transfer is on its way.'}</h2><p>{money(currentResult.amount_atomic)} USDC to {person?.name || shortAddress(recipient)}</p><div className="transfer-progress"><span className="complete"><Check size={14}/>Submitted</span><i/><span className={currentResult.status === 'confirmed' ? 'complete' : ''}>{currentResult.status === 'confirmed' ? <Check size={14}/> : <span className="loading-spinner small"/>}Confirmed</span></div><div className="info-note"><LockKeyhole size={17}/><p>{DEMO_MODE ? 'This is a simulation. You’ll see the demo confirmation in a few seconds.' : 'We’ll verify settlement on the blockchain and update your history.'}</p></div><Link className="button primary full-width" href="/transactions">View transactions <ArrowRight size={16}/></Link><button className="button text-button full-width" onClick={() => { setStep(1); setAmount(''); setRecipient(''); setResult(null); }}>Send another transfer</button></div>}
    </section><aside className="flow-side"><section className="card flow-wallet"><span className="subtle-icon"><Wallet size={21}/></span><h3>Your available balance</h3><strong>{money(balance)}</strong><span>USDC on Base{DEMO_MODE ? '' : ' Sepolia'}</span><div className="wallet-mini-address"><span>{shortAddress(address || '0x0000')}</span><ShieldCheck size={15}/></div></section><section className="transfer-tips"><h3>A thoughtful little checklist</h3><p><CheckCircle2 size={17}/>Confirm the recipient’s wallet address.</p><p><CheckCircle2 size={17}/>Use the same network on both sides.</p><p><CheckCircle2 size={17}/>Start small when sending to a new wallet.</p><Link href="/help">Need a hand? We’re here <ArrowUpRight size={14}/></Link></section></aside></div>
  </>;
}
