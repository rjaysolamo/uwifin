import type { Metadata, Viewport } from 'next';
import './globals.css';
import { AuthProvider } from '@/contexts/AuthContext';
import { FinanceProvider } from '@/contexts/FinanceContext';

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  viewportFit: 'cover',
};

export const metadata: Metadata = {
  title: { default: 'UwiFin — A little closer to home', template: '%s | UwiFin' },
  description:
    'Padala para sa Pamilya. Manage your wallet, send with confidence, and stay connected to the people you love.',
  icons: { icon: '/icon.svg' },
};
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div id="email-wallet-frame" hidden />
        <AuthProvider>
          <FinanceProvider>{children}</FinanceProvider>
        </AuthProvider>
      </body>
    </html>
  );
}
