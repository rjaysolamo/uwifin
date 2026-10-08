export type User = { id: string; email: string; name?: string; created_at?: string };
export type Wallet = { id: string; address: string; network: string; wallet_type: string };
export type Balance = { asset: string; balance: string; network: string };

export async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api/v1${path}`, {
    ...options, credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json', ...options.headers },
    signal: AbortSignal.timeout(20_000),
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) throw new Error(body?.error?.message || 'This service is temporarily unavailable. Please try again.');
  return body as T;
}
