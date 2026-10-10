import { notFound } from 'next/navigation';
import { AppShell } from '@/components/AppShell';
import { SendPage } from '@/components/SendPage';
import { ReceivePage, WalletPage } from '@/components/WalletPages';
import { TransactionsPage } from '@/components/TransactionsPage';
import { PaymentsPage } from '@/components/PaymentsPage';
import { SettingsPage } from '@/components/SettingsPage';
import { AdminPage } from '@/components/AdminPage';
import { HelpPage } from '@/components/HelpPage';
const sections = [
  'wallet',
  'send',
  'receive',
  'transactions',
  'payments',
  'settings',
  'help',
  'admin',
];
export default async function SectionPage({
  params,
  searchParams,
}: {
  params: Promise<{ section: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { section } = await params;
  if (!sections.includes(section)) notFound();
  const query = await searchParams;
  const param = (key: string) => (typeof query[key] === 'string' ? (query[key] as string) : '');
  const views: Record<string, React.ReactNode> = {
    admin: <AdminPage />,
    wallet: <WalletPage />,
    send: (
      <SendPage
        key={`${param('recipient')}-${param('amount')}`}
        initialRecipient={param('recipient')}
        initialAmount={param('amount')}
      />
    ),
    receive: <ReceivePage />,
    transactions: <TransactionsPage key={param('q')} initialQuery={param('q')} />,
    payments: <PaymentsPage />,
    settings: <SettingsPage />,
    help: <HelpPage />,
  };
  return <AppShell>{views[section]}</AppShell>;
}
