'use client';

import './globals.css';
import { AuthProvider } from '@/contexts/AuthContext';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>UwiFin</title>
        <meta
          name="description"
          content="Padala para sa Pamilya. Modern Technology. Global Connection."
        />
      </head>
      <body>
        <AuthProvider>{children}</AuthProvider>
      </body>
    </html>
  );
}
