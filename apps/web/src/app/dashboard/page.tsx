'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { FormEvent, useMemo, useState } from 'react';
import { useAuth } from '@/contexts/AuthContext';

export default function LoginPage() {
  const router = useRouter();
  const { login, register, user } = useAuth();
  const [isRegister, setIsRegister] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  if (user) {
    router.replace('/dashboard');
    return null;
  }

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setLoading(true);
    setError('');

    try {
      if (isRegister) {
        await register(email, password);
      } else {
        await login(email, password);
      }
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  const styles = useMemo(
    () => ({
      page: {
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #dfeefd 0%, #f8fafc 100%)',
        padding: 20,
      } as const,
      shell: {
        width: '100%',
        maxWidth: 1080,
        display: 'grid',
        gap: 24,
        gridTemplateColumns: '1.05fr 0.95fr',
      } as const,
      panel: {
        background: '#ffffff',
        borderRadius: 24,
        padding: 36,
        boxShadow: '0 30px 60px rgba(15, 23, 42, 0.10)',
        border: '1px solid rgba(148, 163, 184, 0.2)',
      } as const,
      side: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #0f172a 0%, #1e3a8a 100%)',
        borderRadius: 24,
        padding: 32,
        color: '#fff',
        boxShadow: '0 30px 60px rgba(30, 58, 138, 0.25)',
      } as const,
      logo: {
        fontSize: 12,
        fontWeight: 800,
        letterSpacing: '0.12em',
        textTransform: 'uppercase',
        color: '#0066cc',
        marginBottom: 18,
      } as const,
      title: {
        margin: 0,
        fontSize: 34,
        fontWeight: 900,
        color: '#0f172a',
      } as const,
      subtitle: {
        margin: '10px 0 24px',
        lineHeight: 1.7,
        color: '#64748b',
        fontSize: 15,
      } as const,
      form: {
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
      } as const,
      label: {
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        fontSize: 13,
        fontWeight: 700,
        color: '#475569',
      } as const,
      input: {
        border: '1px solid #dbe2ea',
        borderRadius: 12,
        padding: '12px 14px',
        fontSize: 15,
        outline: 'none',
        background: '#f8fafc',
      } as const,
      button: {
        borderRadius: 12,
        padding: '12px 16px',
        background: 'linear-gradient(135deg, #0066cc 0%, #0052a3 100%)',
        color: '#ffffff',
        fontWeight: 800,
        fontSize: 15,
      } as const,
      secondary: {
        borderRadius: 12,
        padding: '12px 16px',
        background: '#edf2ff',
        color: '#1d4ed8',
        fontWeight: 800,
      } as const,
      error: {
        color: '#b91c1c',
        background: '#fef2f2',
        borderRadius: 10,
        padding: '10px 12px',
        border: '1px solid #fecaca',
        fontSize: 13,
        fontWeight: 600,
      } as const,
      footer: {
        marginTop: 16,
        textAlign: 'center',
        fontSize: 14,
        color: '#475569',
      } as const,
      link: {
        color: '#0066cc',
        fontWeight: 700,
        textDecoration: 'none',
      } as const,
      heroCard: {
        maxWidth: 340,
      } as const,
      heroTitle: {
        fontSize: 44,
        lineHeight: 1.05,
        fontWeight: 900,
        marginBottom: 18,
      } as const,
      heroText: {
        margin: 0,
        color: 'rgba(255,255,255,0.8)',
        lineHeight: 1.8,
        fontSize: 15,
      } as const,
    }),
    [],
  );

  return (
    <main style={styles.page}>
      <div style={styles.shell}>
        <div style={styles.side}>
          <div style={styles.heroCard}>
            <div style={{ fontSize: 12, letterSpacing: '0.12em', textTransform: 'uppercase', fontWeight: 800, opacity: 0.9, marginBottom: 18 }}>
              Grow your wallet
            </div>
            <h2 style={styles.heroTitle}>Move money with confidence.</h2>
            <p style={styles.heroText}>
              UwiFin helps families and businesses send funds across borders securely, fast, and with full visibility into balances and transfers.
            </p>
          </div>
        </div>

        <div style={styles.panel}>
          <div style={styles.logo}>Uwifin</div>
          <h1 style={styles.title}>{isRegister ? 'Create your account' : 'Welcome back'}</h1>
          <p style={styles.subtitle}>
            {isRegister
              ? 'Open a secure wallet and start moving funds across networks.'
              : 'Sign in to continue to your UwiFin dashboard.'}
          </p>

          <form onSubmit={submit} style={styles.form}>
            <label style={styles.label}>
              Email
              <input
                type="email"
                placeholder="you@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                style={styles.input}
                required
              />
            </label>

            <label style={styles.label}>
              Password
              <input
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                style={styles.input}
                required
                minLength={8}
              />
            </label>

            {error ? <div style={styles.error}>{error}</div> : null}

            <button type="submit" style={styles.button} disabled={loading}>
              {loading ? 'Please wait...' : isRegister ? 'Create account' : 'Sign in'}
            </button>
          </form>

          <div style={{ marginTop: 18 }}>
            <button
              type="button"
              style={{ ...styles.secondary, width: '100%' }}
              onClick={() => setIsRegister((value) => !value)}
            >
              {isRegister ? 'Already have an account? Sign in' : 'Need an account? Register'}
            </button>
          </div>

          <div style={styles.footer}>
            <Link href="/" style={styles.link}>Back home</Link>
          </div>
        </div>
      </div>
    </main>
  );
}
