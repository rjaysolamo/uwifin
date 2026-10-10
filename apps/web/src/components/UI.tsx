'use client';
import { useEffect, useRef, type ReactNode } from 'react';
import { ArrowDownLeft, ArrowUpRight, Check, Copy, Plus, X } from 'lucide-react';
import { useState } from 'react';
import { type Transfer } from '@/lib/transfers';
import { money, shortAddress } from '@/lib/money';

export function UsdcIcon({ small = false }: { small?: boolean }) {
  return (
    <span className={`usdc-icon ${small ? 'small' : ''}`} aria-hidden="true">
      <svg viewBox="0 0 32 32" fill="none">
        <path
          d="M16 7v18M20 11.5c-1-2.8-8-2.7-8 .6 0 3.8 8 2 8 6 0 3.5-7 3.9-8 .7"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
        <path
          d="M9.5 6.8a11 11 0 0 0 0 18.4M22.5 6.8a11 11 0 0 1 0 18.4"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
        />
      </svg>
    </span>
  );
}
export function StatusBadge({ status }: { status: Transfer['status'] }) {
  return (
    <span className={`status-badge ${status}`}>
      <span />
      {status === 'created' ? 'Created' : status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}
export function CopyButton({
  value,
  label = 'Copy address',
  onCopy,
}: {
  value: string;
  label?: string;
  onCopy?: () => void;
}) {
  const [copied, setCopied] = useState(false);
  const [failed, setFailed] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  return (
    <button
      className="copy-button"
      aria-label={label}
      onClick={async () => {
        try {
          await navigator.clipboard.writeText(value);
          setCopied(true);
          setFailed(false);
          onCopy?.();
          if (timer.current) clearTimeout(timer.current);
          timer.current = setTimeout(() => setCopied(false), 2000);
        } catch {
          setFailed(true);
        }
      }}
    >
      {copied ? <Check size={16} /> : <Copy size={16} />}
      <span>{failed ? 'Select and copy manually' : copied ? 'Copied!' : label}</span>
    </button>
  );
}
export function TransactionIcon({ kind }: { kind: Transfer['kind'] }) {
  return (
    <span className={`transaction-icon ${kind}`}>
      {kind === 'sent' ? (
        <ArrowUpRight size={19} />
      ) : kind === 'received' ? (
        <ArrowDownLeft size={19} />
      ) : (
        <Plus size={19} />
      )}
    </span>
  );
}
export function TransactionTable({
  transfers,
  compact = false,
  onSelect,
}: {
  transfers: Transfer[];
  compact?: boolean;
  onSelect: (transfer: Transfer) => void;
}) {
  if (!transfers.length)
    return (
      <div className="empty-state">
        <ArrowUpRight size={28} />
        <h3>No transactions yet</h3>
        <p>Transfers initiated in UwiFin will appear here.</p>
      </div>
    );
  return (
    <div className="table-scroll">
      <table className="transaction-table">
        <thead>
          <tr>
            <th>Transaction</th>
            {!compact && <th>Date</th>}
            <th>Status</th>
            <th className="align-right">Amount</th>
          </tr>
        </thead>
        <tbody>
          {transfers.map((transfer) => (
            <tr key={transfer.id} onClick={() => onSelect(transfer)}>
              <td>
                <button
                  className="transaction-main"
                  onClick={(event) => {
                    event.stopPropagation();
                    onSelect(transfer);
                  }}
                >
                  <TransactionIcon kind={transfer.kind} />
                  <span>
                    <strong>
                      {transfer.kind === 'sent'
                        ? 'Sent to '
                        : transfer.kind === 'received'
                          ? 'Received from '
                          : ''}
                      {transfer.name}
                    </strong>
                    <small>
                      {compact
                        ? new Date(transfer.created_at).toLocaleDateString('en-US', {
                            month: 'short',
                            day: 'numeric',
                            timeZone: 'UTC',
                          })
                        : shortAddress(transfer.address)}{' '}
                      <span>·</span> USDC
                    </small>
                  </span>
                </button>
              </td>
              {!compact && (
                <td className="date-cell">
                  {new Date(transfer.created_at).toLocaleDateString('en-US', {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                    timeZone: 'UTC',
                  })}
                </td>
              )}
              <td>
                <StatusBadge status={transfer.status} />
              </td>
              <td className={`amount-cell ${transfer.kind !== 'sent' ? 'positive' : ''}`}>
                {transfer.kind === 'sent' ? '−' : '+'}
                {money(transfer.amount_atomic)}
                <small>USDC</small>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
export function Modal({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const closeRef = useRef(onClose);
  useEffect(() => {
    closeRef.current = onClose;
  }, [onClose]);
  useEffect(() => {
    const element = dialog.current;
    const focused = document.activeElement as HTMLElement | null;
    element?.showModal();
    const close = () => closeRef.current();
    element?.addEventListener('close', close);
    return () => {
      element?.removeEventListener('close', close);
      element?.close();
      focused?.focus();
    };
  }, []);
  return (
    <dialog
      ref={dialog}
      className="modal"
      onClick={(event) => {
        const rect = dialog.current?.getBoundingClientRect();
        if (
          rect &&
          (event.clientX < rect.left ||
            event.clientX > rect.right ||
            event.clientY < rect.top ||
            event.clientY > rect.bottom)
        )
          onClose();
      }}
    >
      <div className="modal-heading">
        <h2>{title}</h2>
        <button className="icon-button" onClick={onClose} aria-label="Close dialog">
          <X size={20} />
        </button>
      </div>
      {children}
    </dialog>
  );
}
