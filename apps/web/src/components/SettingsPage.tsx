'use client';
import { useState, type FormEvent } from 'react';
import { Bell, Check, ChevronDown, LockKeyhole, RotateCcw, ShieldCheck, UserRound } from 'lucide-react';
import { DEMO_MODE, useAuth } from '@/contexts/AuthContext';
import { useFinance } from '@/contexts/FinanceContext';
import { request } from '@/lib/api';
import { Modal } from './UI';
export function SettingsPage() {
  const { user, updateName } = useAuth();
  const { resetDemo } = useFinance();
  const [name, setName] = useState(user?.name || '');
  const [saved, setSaved] = useState('');
  const [error, setError] = useState('');
  const [reset, setReset] = useState(false);
  const [notify, setNotify] = useState(false);
  const save = async (event: FormEvent) => {
    event.preventDefault(); setSaved(''); setError('');
    const cleaned = name.trim();
    if (cleaned.length < 2 || cleaned.length > 60) { setError('Enter a name between 2 and 60 characters.'); return; }
    try {
      if (DEMO_MODE) updateName(cleaned);
      else await request('/users/me', { method: 'PATCH', body: JSON.stringify({ name: cleaned }) });
      setSaved('Your profile has been saved.');
    } catch (err) { setError(err instanceof Error ? err.message : 'Unable to update your profile.'); }
  };
  return <><section className="page-heading"><div><span className="eyebrow">MAKE YOURSELF AT HOME</span><h1>Settings<span className="greeting-dot">.</span></h1><p>A few little details that make UwiFin yours.</p></div>{DEMO_MODE && <span className="demo-pill">Demo preferences</span>}</section><div className="settings-grid"><section className="card settings-card"><div className="settings-title"><UserRound size={20}/><div><h2>Your profile</h2><p>The details behind your account.</p></div></div><form className="stacked-form" onSubmit={save}><label htmlFor="profile-name">Full name</label><input className="input" id="profile-name" value={name} onChange={(event) => setName(event.target.value)} minLength={2} maxLength={60} required/><label htmlFor="profile-email">Email address</label><input className="input" id="profile-email" type="email" value={user?.email || 'rjay@example.com'} readOnly/><small className="input-help">Email changes require account verification.</small>{saved && <p className="saved-message" role="status"><Check size={16}/>{saved}</p>}{error && <p className="field-error" role="alert">{error}</p>}<button className="button primary" type="submit">Save changes</button></form></section><div className="settings-side"><section className="card settings-card"><div className="settings-title"><ShieldCheck size={20}/><div><h2>Account security</h2><p>A little peace of mind.</p></div></div><div className="security-item"><LockKeyhole size={19}/><div><strong>Secure sessions</strong><p>{DEMO_MODE ? 'Account authentication is unavailable in demo mode.' : 'Your session is protected with an HttpOnly cookie.'}</p></div><span className="mini-badge">{DEMO_MODE ? 'Demo' : 'Enabled'}</span></div><div className="security-item"><ShieldCheck size={19}/><div><strong>Private keys stay private</strong><p>We never request or store your seed phrase.</p></div></div></section><section className="card settings-card"><div className="settings-title"><Bell size={20}/><div><h2>Preferences</h2><p>Keep things comfortable.</p></div></div><div className="preference-row"><label htmlFor="currency-preference">Display currency</label><div className="select-wrap"><select id="currency-preference" defaultValue="USD"><option>USD</option></select><ChevronDown size={14}/></div></div><div className="preference-row"><div><strong>Transfer notifications</strong><small>{notify ? 'Preview preference enabled. Delivery is not configured.' : 'Notification delivery is not configured yet.'}</small></div><button className={`toggle ${notify ? 'on' : ''}`} role="switch" aria-checked={notify} aria-label="Preview transfer notifications preference" onClick={() => setNotify(!notify)}><span/></button></div></section></div>{DEMO_MODE && <section className="card demo-reset"><div><h3>Start fresh in the demo</h3><p>Restore sample funds and transaction history. Your profile stays the same.</p></div><button className="button secondary" onClick={() => setReset(true)}><RotateCcw size={15}/>Reset demo</button></section>}</div>{reset && <Modal title="Reset demo workspace?" onClose={() => setReset(false)}><p className="modal-description">Your simulated transfers will be removed and the balance restored to $1,240.52. Your profile is kept.</p><div className="modal-actions"><button className="button secondary" onClick={() => setReset(false)}>Keep exploring</button><button className="button primary" onClick={() => { resetDemo(); setReset(false); setSaved('Demo balance and transactions restored.'); }}>Reset demo</button></div></Modal>}</>;
}
