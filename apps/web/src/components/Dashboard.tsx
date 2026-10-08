'use client';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState, type FormEvent } from 'react';
import { ArrowDownLeft, ArrowRight, ArrowUpRight, ChevronDown, ChevronRight, Eye, EyeOff, Globe2, LockKeyhole, Plus, Send, ShieldCheck, Sparkles, Zap } from 'lucide-react';
import { useAuth, DEMO_MODE } from '@/contexts/AuthContext';
import { useFinance } from '@/contexts/FinanceContext';
import { recipients, type Transfer } from '@/lib/demo';
import { money, shortAddress, parseAmount } from '@/lib/money';
import { TransactionTable, UsdcIcon } from './UI';
import { FamilyIllustration } from './FamilyIllustration';
import { TransferDetails } from './TransferDetails';

export function Dashboard() {
  const { user } = useAuth();
  const { balance, transfers, address, loading, error, refresh } = useFinance();
  const router = useRouter();
  const [hidden, setHidden] = useState(false);
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [formError, setFormError] = useState('');
  const [detail, setDetail] = useState<Transfer | null>(null);
  const [currency, setCurrency] = useState<'USD' | 'PHP'>('USD');
  const firstName = user?.name?.split(' ')[0] || user?.email.split('@')[0] || 'Rjay';
  const phpAmount = (BigInt(balance) * 5684n / 100n).toString();
  const startTransfer = (event: FormEvent) => {
    event.preventDefault(); setFormError('');
    try {
      const atomic = parseAmount(amount);
      if (atomic > BigInt(balance)) throw new Error('Your amount is higher than your available balance.');
      router.push(`/send?recipient=${encodeURIComponent(recipient)}&amount=${encodeURIComponent(amount)}`);
    } catch (err) { setFormError(err instanceof Error ? err.message : 'Enter a valid amount.'); }
  };
  return <>
    <section className="page-heading"><div><div className="greeting-eyebrow"><span className="tiny-sun">☀</span> A NEW DAY, A LITTLE CLOSER</div><h1>Magandang araw, {firstName}<span className="greeting-dot">.</span></h1><p>Home is never too far away. Let’s take care of what matters.</p></div><div className="heading-right"><span className="date-label">{new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' })}</span>{DEMO_MODE && <span className="demo-pill">Demo account</span>}</div></section>
    {error && <div className="error-banner"><p>{error}</p><button onClick={() => void refresh()}>Try again</button></div>}
    <div className="dashboard-grid">
      <section className="balance-card"><div className="balance-top"><span>Total balance <button className="balance-eye" aria-label={hidden ? 'Show balance' : 'Hide balance'} onClick={() => setHidden(!hidden)}>{hidden ? <EyeOff size={16}/> : <Eye size={16}/>}</button></span><div className="balance-currency"><select aria-label="Display currency" value={currency} onChange={(event) => setCurrency(event.target.value as 'USD' | 'PHP')}><option>USD</option><option>PHP</option></select><ChevronDown size={12}/></div></div><div className="balance-value">{loading ? 'Loading…' : hidden ? '••••••' : money(currency === 'PHP' ? phpAmount : balance, currency === 'PHP' ? '₱' : '$')}<span>{currency}</span></div><div className="balance-equivalent">{hidden ? 'Balance is hidden' : <>{currency === 'USD' ? `≈ ${money(phpAmount, '₱')} PHP` : `≈ ${money(balance)} USD`} <span>· Indicative {DEMO_MODE ? 'demo ' : ''}rate</span></>}</div><div className="balance-actions"><Link className="button balance-send" href="/send"><ArrowUpRight size={17}/>Send money</Link><Link className="button balance-receive" href="/receive"><ArrowDownLeft size={17}/>Receive</Link><Link className="button balance-add" href="/payments"><Plus size={17}/>Add money</Link></div><div className="balance-bottom"><span><ShieldCheck size={14}/>Your wallet. Your control.</span><span className="balance-network"><span/>On Base</span></div><svg className="balance-art" width="400" height="330" viewBox="0 0 400 330" fill="none" aria-hidden="true"><circle cx="348" cy="172" r="130" stroke="currentColor"/><circle cx="348" cy="172" r="160" stroke="currentColor"/><circle cx="348" cy="172" r="190" stroke="currentColor"/><circle cx="348" cy="172" r="220" stroke="currentColor"/><path d="M180 290 385 49M217 317 397 108M213 34 393 278" stroke="currentColor"/></svg></section>
      <section className="card quick-send"><div className="card-heading"><h2>Quick send</h2><span className="subtle-icon"><Send size={18}/></span></div><p>A little support goes a long way.</p><form onSubmit={startTransfer}><label htmlFor="quick-recipient">Send to</label><div className="select-wrap"><select id="quick-recipient" value={recipient} onChange={(event) => setRecipient(event.target.value)} required><option value="" disabled>Choose a recipient</option>{recipients.map((item) => <option value={item.address} key={item.address}>{item.name} · {item.relation}</option>)}<option value="new">Someone new</option></select><ChevronDown size={15}/></div><label htmlFor="quick-amount">You send</label><div className="amount-input"><span>$</span><input id="quick-amount" inputMode="decimal" placeholder="0.00" value={amount} onChange={(event) => setAmount(event.target.value)} required/><span className="input-currency"><UsdcIcon small/>USDC</span></div>{formError && <p className="field-error" role="alert">{formError}</p>}<button className="button primary full-width" type="submit">Continue <ArrowRight size={16}/></button></form><div className="quick-send-note"><Zap size={13}/><span>Eligible transfers can have sponsored fees</span></div></section>
      <section className="card assets-card"><div className="card-heading"><h2>Your assets <span className="count-badge">1</span></h2><Link className="text-link" href="/wallet">Manage wallet <ChevronRight size={14}/></Link></div><div className="asset-row"><UsdcIcon/><div className="asset-name"><h3>USD Coin <span>USDC</span></h3><p><span className="base-logo"/>Base network</p></div><div className="asset-balance"><strong>{hidden ? '••••' : money(balance)}</strong><small>{hidden ? 'Hidden' : `${money(balance, '')} USDC`}</small></div></div><div className="asset-card-footer"><span><span className="green-dot"/> {DEMO_MODE ? 'Sample balances' : 'Wallet balances'}</span><Link href="/receive">{shortAddress(address || '0x0000')} <CopyButtonInline/></Link></div></section>
      <section className="family-card"><div><span className="eyebrow">DISTANCE IS JUST A NUMBER</span><h2>For the ones<br/>back home.</h2><p>Small transfers.<br/>A world of difference.</p><Link href="/send">Send a little love <ArrowUpRight size={15}/></Link></div><FamilyIllustration/></section>
      <section className="card recent-card"><div className="card-heading"><h2>Recent activity</h2><Link className="text-link" href="/transactions">View all <ArrowUpRight size={15}/></Link></div><TransactionTable transfers={transfers.slice(0, 4)} compact onSelect={setDetail}/><div className="recent-footer"><LockKeyhole size={13}/><span>Every step, clearly tracked.</span><span className="recent-footer-right">{DEMO_MODE ? 'Demo transactions' : 'Updated just now'}</span></div></section>
      <div className="dashboard-side-stack"><section className="card recipients-card"><div className="card-heading"><h2>Your people</h2><Link className="small-icon-button" href="/send" aria-label="Send to a new recipient"><Plus size={17}/></Link></div><p>Good things are better shared.</p><div className="people-list">{recipients.map((person) => <Link href={`/send?recipient=${person.address}`} key={person.address}><span className={`person-avatar ${person.color}`}>{person.initials}</span><span><strong>{person.name}</strong><small>{person.relation} <span>·</span> Philippines</small></span><ArrowUpRight size={17}/></Link>)}</div></section><section className="sponsorship-card"><span className="sponsorship-icon"><Zap size={18}/></span><div><h3>More for your family.</h3><p>We can cover network fees on eligible transfers. One less thing to think about.</p><Link href="/help#network-fees">About sponsored fees <ArrowUpRight size={13}/></Link></div><Sparkles size={15} className="sparkle"/></section></div>
    </div>
    <div className="trust-strip"><div><ShieldCheck size={17}/><span>Security comes first</span></div><div><Globe2 size={17}/><span>Connected across borders</span></div><div><HeartIcon/><span>Built around your family</span></div></div>
    {detail && <TransferDetails transfer={detail} onClose={() => setDetail(null)}/>}
  </>;
}
function CopyButtonInline() { return <svg width="13" height="13" viewBox="0 0 16 16" fill="none" aria-hidden="true"><rect x="6" y="6" width="7" height="7" rx="1.5" stroke="currentColor"/><path d="M10 4V3a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h1" stroke="currentColor"/></svg>; }
function HeartIcon() { return <svg width="17" height="17" viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z" stroke="currentColor" strokeWidth="1.6"/></svg>; }
