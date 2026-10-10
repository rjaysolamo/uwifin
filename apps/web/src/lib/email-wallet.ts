import type { AlchemyWebSigner } from '@account-kit/signer';
let signer: AlchemyWebSigner | null = null;
export const emailWalletEnabled = !!process.env.NEXT_PUBLIC_ALCHEMY_SIGNER_KEY;
export async function startEmailWallet(email: string): Promise<void> {
  if (!emailWalletEnabled) throw new Error('Email wallets are not available yet.');
  await disconnectEmailWallet();
  const { AlchemyWebSigner } = await import('@account-kit/signer');
  const current = signer || new AlchemyWebSigner({ client: { connection: { apiKey: process.env.NEXT_PUBLIC_ALCHEMY_SIGNER_KEY! }, iframeConfig: { iframeContainerId: 'email-wallet-frame' } } });
  signer = current;
  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => { unsubscribe(); reject(new Error('Email wallet service timed out. Please retry.')); }, 30_000);
    const unsubscribe = current.on('statusChanged', status => {
      if (status === 'AWAITING_EMAIL_AUTH' || status === 'AWAITING_OTP_AUTH') { clearTimeout(timer); unsubscribe(); resolve(); }
    });
    // Authentication completes after the user enters the email code.
    void current.authenticate({ type: 'email', email, emailMode: 'otp' }).catch(() => { clearTimeout(timer); unsubscribe(); reject(new Error('Unable to send a wallet code. Please retry.')); });
  });
}
export async function verifyEmailWallet(code: string) {
  if (!signer) throw new Error('Request an email code first.');
  try { await signer.authenticate({ type: 'otp', otpCode: code }); }
  catch { throw new Error('The wallet code could not be verified. Check the code or request another.'); }
}
export function emailWalletAccount() {
  try { return signer?.toViemAccount() || null; } catch { return null; }
}
export async function disconnectEmailWallet() {
  if (signer) await signer.disconnect();
}
