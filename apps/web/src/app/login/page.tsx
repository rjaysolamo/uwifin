'use client';

import Link from 'next/link';
import { useEffect, useMemo } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';

export default function HomePage() {
  const router = useRouter();
  const { user, token } = useAuth();

  useEffect(() => {
    if (user && token) {
      router.replace('/dashboard');
    }
  }, [user, token, router]);

  const styles = useMemo(
    () => ({
      page: {
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #0066cc 0%, #4f46e5 100%)',
        padding: 20,
        position: 'relative',
      } as const,
      orb: {
        position: 'absolute',
        top: '10%',
        right: '-8%',
        width: 500,
        height: 500,
        borderRadius: '50%',
        background: 'rgba(255,255,255,0.12)',
        filter: 'blur(70px)',
      } as const,
      card: {
        position: 'relative',
        zIndex: 1,
        width: '100%',
        maxWidth: 680,
        background: 'rgba(255,255,255,0.96)',
        borderRadius: 26,
        border: '1px solid rgba(255,255,255,0.35)',
        padding: '48px 42px',
        boxShadow: '0 30px 60px rgba(15,23,42,0.18)',
      } as const,
      badge: {
        display: 'inline-block',
        marginBottom: 18,
        padding: '8px 12px',
        borderRadius: 999,
        letterSpacing: '0.12em',
        fontSize: 11,
        fontWeight: 800,
        textTransform: 'uppercase',
        color: '#0066cc',
        background: '#eaf3ff',
      } as const,
      title: {
        margin: '0 0 12px',
        fontSize: 52,
        lineHeight: 1.05,
        letterSpacing: '-0.06em',
        fontWeight: 900,
        color: '#0f172a',
      } as const,
      gradientText: {
        background: 'linear-gradient(135deg, #0066cc 0%, #4f46e5 100%)',
        WebkitBackgroundClip: 'text',
        WebkitTextFillColor: 'transparent',
      } as const,
      tagline: {
        margin: '0 0 14px',
        fontSize: 18,
        fontWeight: 700,
        color: '#0066cc',
      } as const,
      subtitle: {
        margin: '0 0 32px',
        fontSize: 16,
        lineHeight: 1.7,
        color: '#475569',
      } as const,
      actions: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: 12,
      } as const,
      primary: {
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: 12,
        padding: '14px 22px',
        background: 'linear-gradient(135deg, #0066cc 0%, #0a5dc2 100%)',
        color: '#fff',
        fontWeight: 800,
        textDecoration: 'none',
        boxShadow: '0 14px 24px rgba(0,102,204,0.24)',
      } as const,
      secondary: {
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: 12,
        padding: '14px 22px',
        background: '#edf2ff',
        color: '#1d4ed8',
        fontWeight: 800,
        textDecoration: 'none',
        border: '1px solid rgba(29,78,216,0.14)',
      } as const,
      features: {
        marginTop: 38,
        paddingTop: 30,
        borderTop: '1px solid #e2e8f0',
      } as const,
      featureGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
        gap: 18,
      } as const,
      feature: {
        padding: 18,
        borderRadius: 16,
        background: '#f8fafc',
        border: '1px solid #e2e8f0',
        textAlign: 'center',
      } as const,
      featureIcon: {
        fontSize: 28,
        marginBottom: 10,
      } as const,
      featureTitle: {
        margin: '0 0 6px',
        fontSize: 14,
        fontWeight: 800,
        color: '#0f172a',
      } as const,
      featureText: {
        margin: 0,
        fontSize: 12,
        color: '#64748b',
        lineHeight: 1.5,
      } as const,
    }),
    [],
  );

  return (
    <main style={styles.page}>
      <div style={styles.orb} />
      <div style={styles.card}>
        <div style={styles.badge}>Uwifin</div>
        <h1 style={styles.title}>
          <span style={styles.gradientText}>Padala para sa Pamilya.</span>
        </h1>
        <p style={styles.tagline}>Modern Technology. Global Connection.</p>
        <p style={styles.subtitle}>
          Secure cross-border transfers, wallet management, and digital finance for families and businesses who need fast, reliable movement of value.
        </p>

        <div style={styles.actions}>
          <Link href="/login" style={styles.primary}>Get Started</Link>
          <Link href="/login" style={styles.secondary}>Open Dashboard</Link>
        </div>

        <div style={styles.features}>
          <div style={styles.featureGrid}>
            <div style={styles.feature}>
              <div style={styles.featureIcon}>🔒</div>
              <div style={styles.featureTitle}>Secure</div>
              <p style={styles.featureText}>Protected wallets with strong user authentication.</p>
            </div>
            <div style={styles.feature}>
              <div style={styles.featureIcon}>⚡</div>
              <div style={styles.featureTitle}>Fast</div>
              <p style={styles.featureText}>Move funds across borders in seconds.</p>
            </div>
            <div style={styles.feature}>
              <div style={styles.featureIcon}>💸</div>
              <div style={styles.featureTitle}>Affordable</div>
              <p style={styles.featureText}>Transparent pricing with fewer delays.</p>
            </div>
            <div style={styles.feature}>
              <div style={styles.featureIcon}>🌍</div>
              <div style={styles.featureTitle}>Global</div>
              <p style={styles.featureText}>Built to support multi-network finance workflows.</p>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
