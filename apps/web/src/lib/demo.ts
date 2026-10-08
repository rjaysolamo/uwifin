export type Transfer = {
  id: string; kind: 'sent' | 'received' | 'bought'; name: string; address: string;
  amount_atomic: string; status: 'confirmed' | 'pending' | 'failed' | 'created';
  created_at: string; tx_hash: string | null; idempotency_key?: string;
};
export type Recipient = { name: string; initials: string; relation: string; address: string; color: string };
export const DEMO_ADDRESS = '0x71C7656EC7ab88b098defB751B7401B5f6d8976F';
export const recipients: Recipient[] = [
  { name: 'Maria Solamo', initials: 'MS', relation: 'Mama', address: '0x0D8a31b3D8Acbe85C972Cc17921791773c521e13', color: 'peach' },
  { name: 'Juan Solamo', initials: 'JS', relation: 'Kuya', address: '0x8ba1f109551bD432803012645Ac136ddd64DBA72', color: 'lavender' },
  { name: 'Ana Reyes', initials: 'AR', relation: 'Ate', address: '0x6f46CF5569aEFa1AcD7840448f829165751457c3', color: 'sage' },
];
// Synthetic demo records. No real settlement hashes are fabricated.
export const initialTransfers: Transfer[] = [
  { id: 'demo-tx-1', kind: 'sent', name: 'Maria Solamo', address: recipients[0].address, amount_atomic: '200000000', status: 'confirmed', created_at: '2026-10-08T09:42:00Z', tx_hash: null },
  { id: 'demo-tx-2', kind: 'received', name: 'Alex Morgan', address: '0x1234567890AbcdEF1234567890aBcdef12345678', amount_atomic: '450000000', status: 'confirmed', created_at: '2026-10-07T16:24:00Z', tx_hash: null },
  { id: 'demo-tx-3', kind: 'sent', name: 'Juan Solamo', address: recipients[1].address, amount_atomic: '85000000', status: 'pending', created_at: '2026-10-07T12:15:00Z', tx_hash: null },
  { id: 'demo-tx-4', kind: 'bought', name: 'Added to wallet', address: DEMO_ADDRESS, amount_atomic: '500000000', status: 'confirmed', created_at: '2026-10-06T10:30:00Z', tx_hash: null },
  { id: 'demo-tx-5', kind: 'sent', name: 'Ana Reyes', address: recipients[2].address, amount_atomic: '125000000', status: 'confirmed', created_at: '2026-10-05T14:00:00Z', tx_hash: null },
  { id: 'demo-tx-6', kind: 'sent', name: 'Maria Solamo', address: recipients[0].address, amount_atomic: '75000000', status: 'failed', created_at: '2026-10-03T11:45:00Z', tx_hash: null },
];
