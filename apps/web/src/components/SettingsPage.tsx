'use client';
import { useState, type FormEvent } from 'react';
import { useAuth } from '@/contexts/AuthContext';
export function SettingsPage() {
  const { user, updateName } = useAuth();
  const [name, setName] = useState(user?.name || '');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const save = async (event: FormEvent) => { event.preventDefault(); setBusy(true); setMessage(''); try { await updateName(name.trim()); setMessage('Profile saved.'); } catch (err) { setMessage(err instanceof Error ? err.message : 'Unable to save profile.'); } finally { setBusy(false); } };
  return <><section className="page-heading"><h1>Settings.</h1></section><section className="card settings-card"><form className="stacked-form" onSubmit={save}><label htmlFor="name">Full name</label><input className="input" id="name" value={name} onChange={event => setName(event.target.value)} minLength={2} maxLength={60} required/><label htmlFor="email">Email</label><input className="input" id="email" value={user?.email || ''} readOnly/><p>Email changes require account verification.</p><p role="status">{message}</p><button className="button primary" disabled={busy}>Save profile</button></form></section><section className="card settings-card"><h2>Wallet security</h2><p>Signing takes place in your connected wallet. Keep your signing account and recovery phrase secure. UwiFin does not store your private keys.</p></section></>;
}
