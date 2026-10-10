export type Transfer = {
  id: string;
  kind: 'sent' | 'received' | 'bought';
  name: string;
  address: string;
  amount_atomic: string;
  status: 'confirmed' | 'pending' | 'failed' | 'created' | 'validating' | 'ready' | 'submitted';
  user_operation_hash?: string | null;
  gas_used?: string | null;
  gas_price?: string | null;
  error_code?: string | null;
  network: string;
  created_at: string;
  tx_hash: string | null;
};
