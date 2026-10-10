'use client';
import { useEffect, useState } from 'react';
import { request } from '@/lib/api';
import { type Transfer } from '@/lib/transfers';
import { decimalAmount } from '@/lib/money';
import { TransactionTable } from './UI';
import { TransferDetails } from './TransferDetails';
export function TransactionsPage({ initialQuery = '' }: { initialQuery?: string }) {
  const [query, setQuery] = useState(initialQuery);
  const [status, setStatus] = useState('');
  const [kind, setKind] = useState('');
  const [page, setPage] = useState(1);
  const [transfers, setTransfers] = useState<Transfer[]>([]);
  const [detail, setDetail] = useState<Transfer | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    const controller = new AbortController();
    const load = async () => {
      setLoading(true); setError('');
      try {
        const params = new URLSearchParams({ page: String(page), limit: '20', q: query });
        if (status) params.set('status',status); if (kind) params.set('kind',kind);
        const result = await request<{ transactions: Transfer[] }>(`/transactions?${params}`,{signal:controller.signal});
        if (!controller.signal.aborted) setTransfers(result.transactions);
      } catch (err) { if (!controller.signal.aborted) setError(err instanceof Error ? err.message : 'Unable to load history.'); }
      finally { if (!controller.signal.aborted) setLoading(false); }
    };
    const debounce = setTimeout(() => void load(), 200);
    const timer = setInterval(() => { if (!document.hidden) void load(); },15_000);
    return () => { controller.abort(); clearTimeout(debounce); clearInterval(timer); };
  },[page,query,status,kind]);
  const exportCsv = () => {
    const escape = (value: string) => `"${value.replace(/^[=+\-@\t\r]/, "' $&").replaceAll('"', '""')}"`;
    const rows = [['ID','Type','Address','Amount USDC','Status','Date','Network','Hash'],...transfers.map(t => [t.id,t.kind,t.address,decimalAmount(t.amount_atomic),t.status,t.created_at,t.network,t.tx_hash || ''])];
    const url = URL.createObjectURL(new Blob([rows.map(row => row.map(escape).join(',')).join('\r\n')],{type:'text/csv;charset=utf-8;'}));
    const link = document.createElement('a'); link.href=url; link.download=`uwifin-transactions-page-${page}.csv`; link.click(); setTimeout(() => URL.revokeObjectURL(url),1000);
  };
  return <><section className="page-heading"><div><h1>Transactions.</h1><p>Outgoing transfers and confirmed incoming deposits. New deposits appear after blockchain verification.</p></div><button className="button secondary" disabled={loading || !!error || !transfers.length} onClick={exportCsv}>Export this page</button></section>
    <section className="card all-transactions"><div className="transaction-controls"><label>Search<input className="input" value={query} maxLength={128} onChange={e => {setQuery(e.target.value);setPage(1);}} placeholder="Address or transaction ID"/></label><label>Type<select className="input" value={kind} onChange={e => {setKind(e.target.value);setPage(1);}}><option value="">All</option><option value="sent">Sent</option><option value="received">Received</option></select></label><label>Status<select className="input" value={status} onChange={e => {setStatus(e.target.value);setPage(1);}}><option value="">All</option>{['created','validating','ready','submitted','pending','confirmed','failed'].map(value => <option key={value}>{value}</option>)}</select></label></div>
    {error && <p role="alert">{error}</p>}{loading && <p role="status">Loading history…</p>}{!error && <TransactionTable transfers={transfers} onSelect={setDetail}/>}
    <div className="table-pagination"><button className="button secondary" disabled={loading || page===1} onClick={() => setPage(page-1)}>Previous</button><span>Page {page}</span><button className="button secondary" disabled={loading || !!error || transfers.length<20} onClick={() => setPage(page+1)}>Next</button></div></section>
    {detail && <TransferDetails transfer={transfers.find(t=>t.id===detail.id) || detail} onClose={() => setDetail(null)}/>}</>;
}
