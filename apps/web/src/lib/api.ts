export type User = { role: 'user' | 'admin'; id: string; email: string; name?: string; created_at?: string };
export type Wallet = { id: string; address: string; network: string; wallet_type: string; signer_address: string | null };
export type Capabilities = { network: string; chain_id: number; usdc_address: string; wallet_enabled: boolean; sponsorship_enabled: boolean; onramp_enabled: boolean; stripe_mode: string; offramp_enabled: boolean };
export type Balance = { asset: string; balance: string; network: string };

export async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api/v1${path}`, {
    ...options, credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json', ...options.headers },
    signal: options.signal ? AbortSignal.any([options.signal, AbortSignal.timeout(30_000)]) : AbortSignal.timeout(30_000),
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) throw new Error(body?.error?.message || 'This service is temporarily unavailable. Please try again.');
  return body as T;
}
