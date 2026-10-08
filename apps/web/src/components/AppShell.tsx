'use client';

import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useState, type ReactNode } from 'react';
import { ArrowDownLeft, ArrowUpRight, Bell, ChevronDown, CircleHelp, CreditCard, Heart, LayoutGrid, LogOut, Menu, Search, Settings, ShieldCheck, Wallet, X, ArrowLeftRight, Check } from 'lucide-react';
import { Brand } from './Brand';
import { DEMO_MODE, useAuth } from '@/contexts/AuthContext';

const navigation = [
  { name: 'Overview', href: '/', icon: LayoutGrid },
  { name: 'My wallet', href: '/wallet', icon: Wallet },
  { name: 'Send money', href: '/send', icon: ArrowUpRight },
  { name: 'Receive money', href: '/receive', icon: ArrowDownLeft },
  { name: 'Transactions', href: '/transactions', icon: ArrowLeftRight },
  { name: 'Add money', href: '/payments', icon: CreditCard },
];
export function AppShell({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const router = useRouter();
  const { user, loading, logout } = useAuth();
  const [mobileOpen, setMobileOpen] = useState(false);
  const [notifications, setNotifications] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [logoutError, setLogoutError] = useState('');
  useEffect(() => { if (!DEMO_MODE && !loading && !user) router.replace('/login'); }, [user, loading, router]);
  useEffect(() => { setMobileOpen(false); setNotifications(false); setProfileOpen(false); }, [pathname]);
  useEffect(() => {
    const handler = (event: KeyboardEvent) => { if (event.key === 'Escape') { setNotifications(false); setProfileOpen(false); setMobileOpen(false); } };
    window.addEventListener('keydown', handler); return () => window.removeEventListener('keydown', handler);
  }, []);
  if (!DEMO_MODE && (loading || !user)) return <div className="loading-page"><Brand/><span className="loading-spinner"/><p>Opening your account…</p></div>;
  const name = user?.name || user?.email.split('@')[0] || 'Rjay';
  return <div className="app-shell">
    <a href="#main-content" className="skip-link">Skip to content</a>
    {mobileOpen && <button className="sidebar-overlay" aria-label="Close navigation" onClick={() => setMobileOpen(false)}/>}
    <aside className={`sidebar ${mobileOpen ? 'is-open' : ''}`}>
      <div className="sidebar-brand"><Link href="/" aria-label="UwiFin overview"><Brand/></Link><button className="icon-button mobile-close" aria-label="Close navigation" onClick={() => setMobileOpen(false)}><X size={20}/></button></div>
      <span className="sidebar-eyebrow">YOUR EVERYDAY FINANCE</span>
      <nav aria-label="Main navigation">{navigation.map(({ name, href, icon: Icon }) => <Link key={href} href={href} className={`nav-link ${pathname === href || (href === '/' && pathname === '/dashboard') ? 'active' : ''}`} aria-current={pathname === href ? 'page' : undefined}><Icon size={19}/><span>{name}</span>{href === '/payments' && <span className="nav-new">NEW</span>}</Link>)}</nav>
      <div className="sidebar-bottom">
        <div className="home-note"><span className="home-note-icon"><Heart size={19}/></span><h3>A little closer to home.</h3><p>Big dreams. Everyday moments.<br/>Keep your family connected.</p><span>Padala para sa Pamilya.</span></div>
        <Link href="/settings" className={`nav-link ${pathname === '/settings' ? 'active' : ''}`}><Settings size={19}/><span>Settings</span></Link>
        <Link href="/help" className={`nav-link ${pathname === '/help' ? 'active' : ''}`}><CircleHelp size={19}/><span>Help & support</span><ArrowUpRight size={14} className="nav-end"/></Link>
        <div className="sidebar-security"><ShieldCheck size={15}/><span>Designed with security in mind</span></div>
      </div>
    </aside>
    <div className="workspace">
      <header className="topbar">
        <button className="icon-button mobile-menu" aria-label="Open navigation" aria-expanded={mobileOpen} onClick={() => setMobileOpen(true)}><Menu size={22}/></button>
        <div className="breadcrumb">Your workspace <span>/</span> <strong>{navigation.find((item) => item.href === pathname)?.name || (pathname === '/settings' ? 'Settings' : pathname === '/help' ? 'Help & support' : 'Overview')}</strong></div>
        <div className="topbar-actions">
          <form className="global-search" role="search" onSubmit={(event) => { event.preventDefault(); router.push(`/transactions?q=${encodeURIComponent(query)}`); }}><Search size={16}/><input aria-label="Search transactions" placeholder="Search anything…" value={query} onChange={(event) => setQuery(event.target.value)}/><kbd>↵</kbd></form>
          <span className="network-pill"><span/>Base{DEMO_MODE ? '' : ' Sepolia'}</span>
          <div className="popover-wrap"><button className={`icon-button notification-button ${notifications ? 'selected' : ''}`} aria-label="Notifications" aria-expanded={notifications} onClick={() => { setNotifications(!notifications); setProfileOpen(false); }}><Bell size={20}/><span className="notification-dot"/></button>{notifications && <div className="popover notification-popover"><h3>Notifications</h3><div><span className="notification-icon"><Check size={17}/></span><p><strong>Welcome to UwiFin</strong><small>{DEMO_MODE ? 'You’re exploring with demo funds. No real money is moved.' : 'Your account is ready. Connect a wallet to get started.'}</small></p></div><Link href="/transactions">View your activity <ArrowUpRight size={14}/></Link></div>}</div>
          <span className="topbar-divider"/>
          <div className="popover-wrap"><button className="profile-button" aria-label="Account menu" aria-expanded={profileOpen} onClick={() => { setProfileOpen(!profileOpen); setNotifications(false); }}><span className="avatar">{name.split(' ').map((item) => item[0]).slice(0, 2).join('').toUpperCase()}</span><ChevronDown size={14}/></button>{profileOpen && <div className="popover profile-popover"><strong>{name}</strong><small>{user?.email || 'Demo account'}</small><Link href="/settings"><Settings size={16}/>Account settings</Link><Link href="/login"><LogOut size={16}/>{DEMO_MODE ? 'Go to sign in' : 'Switch account'}</Link><button onClick={async () => { try { await logout(); router.push('/login'); } catch { setLogoutError('Unable to sign out. Please try again.'); } }}><LogOut size={16}/>Sign out</button>{logoutError && <p className="field-error">{logoutError}</p>}</div>}</div>
        </div>
      </header>
      <main id="main-content" className="main-content">{children}</main>
      <footer className="app-footer"><span>© 2026 UwiFin <span className="footer-dot">·</span> Made for the people you love.</span><span><ShieldCheck size={13}/>{DEMO_MODE ? 'Demo workspace · No real funds' : 'Base Sepolia · Test network'}</span></footer>
    </div>
  </div>;
}
