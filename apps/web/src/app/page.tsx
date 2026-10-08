'use client';

import { useEffect, useState } from 'react';

export default function Home() {
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [message, setMessage] = useState<string>('');

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
        const response = await fetch(`${apiUrl}/health`, { method: 'GET' });
        if (response.ok) {
          setStatus('ready');
          setMessage('API is healthy');
        } else {
          setStatus('error');
          setMessage('API returned unhealthy status');
        }
      } catch (err) {
        setStatus('error');
        setMessage(`Could not reach API: ${err instanceof Error ? err.message : 'Unknown error'}`);
      }
    };

    checkHealth();
  }, []);

  return (
    <main style={styles.container}>
      <div style={styles.card}>
        <h1 style={styles.title}>UwiFin</h1>
        <p style={styles.tagline}>Padala para sa Pamilya.</p>
        <p style={styles.subtitle}>Modern Technology. Global Connection.</p>

        <div style={styles.statusBox}>
          <p style={styles.label}>API Status</p>
          <p style={{
            ...styles.statusText,
            color: status === 'ready' ? 'var(--success)' : status === 'error' ? 'var(--error)' : 'var(--neutral-500)'
          }}>
            {status === 'loading' ? '⏳ Checking...' : status === 'ready' ? '✓ Ready' : '✗ Error'}
          </p>
          <p style={styles.message}>{message}</p>
        </div>

        <p style={styles.footer}>Coming soon...</p>
      </div>
    </main>
  );
}

const styles = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '100vh',
    padding: '20px',
  } as React.CSSProperties,
  card: {
    background: 'white',
    borderRadius: '8px',
    padding: '40px',
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
    maxWidth: '500px',
    textAlign: 'center',
  } as React.CSSProperties,
  title: {
    fontSize: '2rem',
    fontWeight: '700',
    marginBottom: '8px',
    color: 'var(--primary)',
  } as React.CSSProperties,
  tagline: {
    fontSize: '1.1rem',
    fontWeight: '600',
    color: 'var(--neutral-700)',
    marginBottom: '4px',
  } as React.CSSProperties,
  subtitle: {
    fontSize: '0.9rem',
    color: 'var(--neutral-500)',
    marginBottom: '32px',
  } as React.CSSProperties,
  statusBox: {
    background: 'var(--neutral-50)',
    borderRadius: '6px',
    padding: '20px',
    marginBottom: '24px',
    border: '1px solid var(--neutral-200)',
  } as React.CSSProperties,
  label: {
    fontSize: '0.85rem',
    color: 'var(--neutral-500)',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    marginBottom: '8px',
  } as React.CSSProperties,
  statusText: {
    fontSize: '1.1rem',
    fontWeight: '600',
    marginBottom: '8px',
  } as React.CSSProperties,
  message: {
    fontSize: '0.85rem',
    color: 'var(--neutral-600)',
  } as React.CSSProperties,
  footer: {
    fontSize: '0.85rem',
    color: 'var(--neutral-400)',
  } as React.CSSProperties,
};
