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
        background: 'linear-gradient(135deg, #eef6ff 0%, #f8fafc 100%)',
        padding: '24px',
      } as const,
      card: {
        width: '100%',
        maxWidth: 440,
        background: '#ffffff',
        borderRadius: 16,
        boxShadow: '0 18px 40px rgba(15, 23, 42, 0.08)',
        padding: 28,
      } as const,
      title: { fontSize: 28, fontWeight: 700, marginBottom: 8, color: '#0f172a' } as const,
      subtitle: { fontSize: 14, color: '#475569', marginBottom: 20 } as const,
      form: { display: 'flex', flexDirection: 'column', gap: 16 } as const,
      input: {
        border: '1px solid #dbe2ea',
        borderRadius: 12,
        padding: '12px 14px',
        fontSize: 15,
        outline: 'none',
      } as const,
      button: {
        borderRadius: 12,
        padding: '12px 16px',
        background: '#0066cc',
        color: 'white',
        fontWeight: 700,
      } as const,
      secondary: {
        borderRadius: 12,
        padding: '12px 16px',
        background: '#e2e8f0',
        color: '#0f172a',
        fontWeight: 700,
      } as const,
      error: { color: '#b91c1c', fontSize: 13, marginTop: 4 } as const,
      footer: { marginTop: 18, fontSize: 14, color: '#475569', textAlign: 'center' } as const,
    }),
    [],
  );

  return (
    <main style={styles.page}>
      <div style={styles.card}>
        <h1 style={styles.title}>{isRegister ? 'Create your account' : 'Welcome back'}</h1>
        <p style={styles.subtitle}>
          {isRegister ? 'Open a secure wallet account to send funds across borders.' : 'Sign in to continue to your UwiFin dashboard.'}
        </p>

        <form onSubmit={submit} style={styles.form}>
          <input
            type="email"
            placeholder="Email address"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            style={styles.input}
            required
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            style={styles.input}
            required
            minLength={8}
          />

          {error ? <span style={styles.error}>{error}</span> : null}

          <button type="submit" style={styles.button} disabled={loading}>
            {loading ? 'Please wait...' : isRegister ? 'Create account' : 'Sign in'}
          </button>
        </form>

        <div style={styles.footer}>
          <button
            type="button"
            onClick={() => setIsRegister(!isRegister)}
            style={{ ...styles.secondary, marginTop: 8, width: '100%' }}
          >
            {isRegister ? 'Already have an account? Sign in' : 'Need an account? Register'}
          </button>

          <div style={{ marginTop: 16 }}>
            <Link href="/" style={{ color: '#0066cc', fontWeight: 600 }}>Back home</Link>
          </div>
        </div>
      </div>
    </main>
  );
}
