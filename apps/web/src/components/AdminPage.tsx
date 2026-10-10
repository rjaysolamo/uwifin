'use client';
import { useEffect, useState } from 'react';
import { useAuth } from '@/contexts/AuthContext';
import { request } from '@/lib/api';
const sections = ['users', 'transactions', 'payments', 'events', 'errors', 'assets', 'networks'];
export function AdminPage() {
  const { user } = useAuth();
  const [section, setSection] = useState('errors');
  const [page, setPage] = useState(1);
  const [rows, setRows] = useState<Record<string, string | number | null>[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    if (user?.role !== 'admin') return;
    const controller = new AbortController();
    setBusy(true);
    setError('');
    setRows([]);
    request<{ records: typeof rows }>(`/admin/${section}?page=${page}`, {
      signal: controller.signal,
    })
      .then((result) => setRows(result.records))
      .catch((err) => {
        if (!controller.signal.aborted) setError(err.message);
      })
      .finally(() => {
        if (!controller.signal.aborted) setBusy(false);
      });
    return () => controller.abort();
  }, [user?.role, section, page, revision]);
  if (user?.role !== 'admin') return <p role="alert">Administrator access is required.</p>;
  const toggle = async (row: (typeof rows)[number]) => {
    setBusy(true);
    setError('');
    try {
      await request(`/admin/${section}/${row.id}`, {
        method: 'PATCH',
        body: JSON.stringify({ active: !row.is_active }),
      });
      setRevision((value) => value + 1);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to update setting.');
      setBusy(false);
    }
  };
  return (
    <>
      <section className="page-heading">
        <h1>Administration</h1>
      </section>
      <section className="card settings-card">
        <label htmlFor="admin-section">Records</label>
        <select
          id="admin-section"
          className="input"
          value={section}
          onChange={(event) => {
            setSection(event.target.value);
            setPage(1);
          }}
        >
          {sections.map((value) => (
            <option key={value}>{value}</option>
          ))}
        </select>
        <p>
          Disabling an asset or network pauses new financial operations. Submitted transfers
          continue to reconcile.
        </p>
        {error && <p role="alert">{error}</p>}
        {busy && <p role="status">Loading…</p>}
        {!busy && !rows.length && <p>No records.</p>}
        <div className="table-scroll">
          <table className="transaction-table">
            <thead>
              <tr>
                {Object.keys(rows[0] || {}).map((key) => (
                  <th key={key}>{key.replaceAll('_', ' ')}</th>
                ))}
                {['assets', 'networks'].includes(section) && <th>Action</th>}
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id}>
                  {Object.entries(row).map(([key, value]) => (
                    <td key={key}>{String(value ?? '—')}</td>
                  ))}
                  {['assets', 'networks'].includes(section) && (
                    <td>
                      <button
                        className="button secondary"
                        disabled={busy}
                        onClick={() => void toggle(row)}
                      >
                        {row.is_active ? 'Disable' : 'Enable'}
                      </button>
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="table-pagination">
          <button
            className="button secondary"
            disabled={busy || page === 1}
            onClick={() => setPage(page - 1)}
          >
            Previous
          </button>
          <span>Page {page}</span>
          <button
            className="button secondary"
            disabled={busy || rows.length < 50}
            onClick={() => setPage(page + 1)}
          >
            Next
          </button>
        </div>
      </section>
    </>
  );
}
