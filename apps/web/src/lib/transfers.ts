export type Transfer = {
  id: string; kind: 'sent' | 'received' | 'bought'; name: string; address: string;
  amount_atomic: string; status: 'confirmed' | 'pending' | 'failed' | 'created' | 'validating' | 'ready' | 'submitted';
  network: string; created_at: string; tx_hash: string | null;
};
