import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'UwiFin',
  description: 'Padala para sa Pamilya. Modern Technology. Global Connection.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
