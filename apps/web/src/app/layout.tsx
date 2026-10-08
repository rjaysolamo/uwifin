'use client';

import Link from 'next/link';
import { useMemo } from 'react';

export default function HomePage() {
  const styles = useMemo(
    () => ({
      page: {
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #eef6ff 0%, #f8fafc 100%)',
        padding: 24,
      } as const,
      card: {
        maxWidth: 560,
        width: '100%',
        background: '#fff',
        borderRadius: 18,
        padding: 36,
        boxShadow: '0 18px 40px rgba(15, 23, 42, 0.08)',
      } as const,
      title: { fontSize: 40, fontWeight: 800, marginBottom: 8, color: '#0f172a' } as const,
      tagline: { fontSize: 18, color: '#475569', marginBottom: 12 } as const,
      subtitle: { fontSize: 15, color: '#64748b', marginBottom: 26 } as const,
      actions: { display: 'flex', gap: 12, flexWrap: 'wrap' } as const,
      primary: {
        borderRadius: 12,
        padding: '12px 18px',
        background: '#0066cc',
        color: '#fff',
        fontWeight: 700,
        textDecoration: 'none',
      } as const,
      secondary: {
        borderRadius: 12,
        padding: '12px 18px',
        background: '#e2e8f0',
        color: '#0f172a',
        fontWeight: 700,
        textDecoration: 'none',
      } as const,
    }),
    [],
  );

  return (
    <main style={styles.page}>
      <div style={styles.card}>
        <h1 style={styles.title}>UwiFin</h1>
        <p style={styles.tagline}>Padala para sa Pamilya.</p>
        <p style={styles.subtitle}>Modern Technology. Global Connection.</p>

        <div style={styles.actions}>
          <Link href="/login" style={styles.primary}>Login</Link>
          <Link href="/dashboard" style={styles.secondary}>Open dashboard</Link>
        </div>
      </div>
    </main>
  );
}
