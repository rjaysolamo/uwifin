'use client';
import { useState } from 'react';
import { ArrowDownLeft, ArrowUpRight, ChevronDown, ChevronLeft, ChevronRight, Download, Search, X } from 'lucide-react';
import { useFinance } from '@/contexts/FinanceContext';
import { DEMO_MODE } from '@/contexts/AuthContext';
import { type Transfer } from '@/lib/demo';
import { decimalAmount, money } from '@/lib/money';
import { TransactionTable } from './UI';
import { TransferDetails } from './TransferDetails';
export function TransactionsPage({ initialQuery = '' }: { initialQuery?: string }) {
  const { transfers } = useFinance();
  const [query, setQuery] = useState(initialQuery);
  const [status, setStatus] = useState('all');
  const [kind, setKind] = useState('all');
  const [page, setPage] = useState(1);
  const [detail, setDetail] = useState<Transfer | null>(null);
  const filtered = transfers.filter((item) => (status === 'all' || item.status === status) && (kind === 'all' || item.kind === kind) && `${item.name} ${item.address} ${item.id} USDC`.toLowerCase().includes(query.toLowerCase()));
  const pages = Math.max(1, Math.ceil(filtered.length / 8));
  const safePage = Math.min(page, pages);
  const sum = (type: string) => transfers.filter((item) => item.kind === type && item.status === 'confirmed').reduce((total, item) => total + BigInt(item.amount_atomic), 0n);
  const exportCsv = () => {
    const escape = (value: string) => `"${value.replace(/^[=+\-@\t\r]/, "' $&").replaceAll('"', '""')}"`;
    const rows = [['ID', 'Type', 'Recipient', 'Wallet', 'Amount USDC', 'Status', 'Date', 'Mode'], ...filtered.map((item) => [item.id, item.kind, item.name, item.address, decimalAmount(item.amount_atomic), item.status, item.created_at, DEMO_MODE ? 'demo' : 'live'])];
    const url = URL.createObjectURL(new Blob([rows.map((row) => row.map(escape).join(',')).join('\r\n')], { type: 'text/csv;charset=utf-8;' }));
    const element = document.createElement('a'); element.href = url; element.download = 'uwifin-transactions.csv'; element.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  return <><section className="page-heading"><div><span className="eyebrow">EVERY LITTLE MOMENT, TRACKED</span><h1>Transactions<span className="greeting-dot">.</span></h1><p>A clear picture of your money, from start to finish.</p></div><button className="button secondary" onClick={exportCsv} disabled={!filtered.length}><Download size={16}/>Export CSV</button></section><div className="transaction-summary"><section className="card"><span className="transaction-icon received"><ArrowDownLeft size={20}/></span><div><span>Total received</span><strong>{money(sum('received'))}</strong><small>Confirmed transfers</small></div></section><section className="card"><span className="transaction-icon sent"><ArrowUpRight size={20}/></span><div><span>Total sent</span><strong>{money(sum('sent'))}</strong><small>Confirmed transfers</small></div></section><section className="card"><span className="pending-summary-dot"/><div><span>In progress</span><strong>{transfers.filter((item) => item.status === 'pending' || item.status === 'created').length} <span>transfers</span></strong><small>Awaiting confirmation</small></div></section></div><section className="card all-transactions"><div className="transaction-controls"><div className="filter-tabs" aria-label="Transaction type">{[{ value: 'all', label: 'All transactions' }, { value: 'sent', label: 'Sent' }, { value: 'received', label: 'Received' }, { value: 'bought', label: 'Added' }].map((item) => <button key={item.value} onClick={() => { setKind(item.value); setPage(1); }} className={kind === item.value ? 'active' : ''} aria-pressed={kind === item.value}>{item.label}</button>)}</div><div className="transaction-filter-row"><div className="search-field"><Search size={16}/><input aria-label="Search transaction history" placeholder="Search name or address…" value={query} onChange={(event) => { setQuery(event.target.value); setPage(1); }}/>{query && <button className="icon-button" onClick={() => setQuery('')} aria-label="Clear search"><X size={14}/></button>}</div><div className="select-wrap status-select"><select aria-label="Filter by status" value={status} onChange={(event) => { setStatus(event.target.value); setPage(1); }}><option value="all">All statuses</option><option value="confirmed">Confirmed</option><option value="pending">Pending</option><option value="failed">Failed</option><option value="created">Created</option></select><ChevronDown size={14}/></div></div></div><TransactionTable transfers={filtered.slice((safePage - 1) * 8, safePage * 8)} onSelect={setDetail}/><div className="table-pagination"><span>{filtered.length ? `${(safePage - 1) * 8 + 1}–${Math.min(safePage * 8, filtered.length)} of ${filtered.length} transactions` : 'No matching transactions'}{DEMO_MODE && ' · Demo'}</span><div><button className="icon-button" aria-label="Previous page" disabled={safePage === 1} onClick={() => setPage(safePage - 1)}><ChevronLeft size={17}/></button><span>{safePage} / {pages}</span><button className="icon-button" aria-label="Next page" disabled={safePage === pages} onClick={() => setPage(safePage + 1)}><ChevronRight size={17}/></button></div></div></section>{detail && <TransferDetails transfer={detail} onClose={() => setDetail(null)}/>}</>;
}
