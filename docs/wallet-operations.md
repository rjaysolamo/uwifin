# Wallet operation and activation

## What is implemented

Email/password sessions use an HttpOnly browser cookie via the Next.js BFF. Connecting an injected EIP-1193 Ethereum wallet requests a short-lived challenge bound to the application user, session, origin, and network. The backend verifies the EIP-191 signature before associating an Alchemy MAv2 (`sma-b`) smart account. Reconnecting the same verified signer returns its existing account. The signing wallet retains its keys; UwiFin cannot recover a lost signing account. Mobile requires a wallet's built-in browser; WalletConnect and embedded signer onboarding are not included.

The receiving address is the smart account address, not the signing account. Balances read only the configured USDC contract. Base Sepolia (84532) and Base (8453) are supported configurations; only one is enabled per deployment. No sample balances, addresses, recipients, exchange rates, or timed confirmations are used.

The backend stores an idempotent transfer intent, constructs only USDC `transfer(address,uint256)` calldata, and requires approved sponsorship. The browser uses the official Alchemy SDK to sign the returned signature request through the user's wallet. The exact prepared envelope is preserved when submitting. Signed bytes are persisted before sending to Alchemy; an ambiguous retry resends the same operation, never a fresh nonce. One active operation per wallet is enforced by MariaDB. The worker verifies chain, canonical receipt block, confirmation depth, and the expected token Transfer event before confirmation. A receipt failure marks the transfer failed. Pending operations must be reconciled before another operation uses the wallet.

History currently shows the latest 100 UwiFin-originated transfers, with local filtering and CSV export. Incoming deposits affect balances but are not indexed as history entries. The receive screen explicitly identifies testnet. No USDC/USD peg or PHP exchange-rate quote is assumed.

## Local startup

Tested toolchain: Node 24.14.1, pnpm 10.34.5, Rust 1.90.0, MariaDB 11.4. Install locked dependencies with `pnpm install --frozen-lockfile`. Copy root `.env.example` to `.env` and `apps/web/.env.example` to `apps/web/.env.local`; set secrets in those ignored files or your secret manager. Start `docker compose up -d`, then `cargo run --manifest-path services/api/Cargo.toml`, then `pnpm dev:web --hostname 0.0.0.0`. In environments using mise, prefix Rust commands with `mise exec rust@1.90.0 --`.

`/health` is process liveness. `/ready` checks MariaDB and required wallet configuration; it does not certify external provider availability. With no provider keys, registration/login work, and financial features show unavailable states.

## Alchemy activation

1. Create an Alchemy application with Wallet APIs access and RPC access for the chosen network. Configure `ALCHEMY_API_KEY` and `ALCHEMY_POLICY_ID` on the Rust server only. Explicit URLs may be used for account-specific endpoints; never expose them in the browser.
2. Configure Gas Manager restrictions for the one chain, the exact USDC contract, and selector `0xa9059cbb`. Set per-user limits, aggregate daily/monthly budgets, and alerts in Alchemy. The server amount cap is additional protection, not a replacement for provider budgets.
3. Connect a wallet on Base Sepolia; fund the smart account with test USDC; send a small amount to a second wallet. Verify the returned user operation and receipt against the explorer. Test rejection, sponsorship denial, provider interruption and retry before switching to Base.
4. For live operation, configure HTTPS `APP_BASE_URL`, HTTPS provider URLs, production database credentials, backups with restore drills, monitoring for stale submitted/pending rows, and protected API routing. Never deploy the compose development passwords.

The API currently uses in-memory rate limits and runs its reconciliation worker in-process. Run a single API instance. Apply edge rate limits before the BFF; the API peer-based auth limit sees the BFF as one peer. Configure timeouts above provider request duration. Do not trust client-supplied forwarding headers. Account recovery, email verification, and production security review remain release requirements. No funds were moved during local fixture tests.

## Stripe

Configure approved Crypto Onramp access, matching test or live secret/publishable keys, `STRIPE_WEBHOOK_SECRET`, and `STRIPE_ONRAMP_ENABLED=true`. Purchases are restricted to USDC on Base mainnet; test mode still requires Stripe eligibility and supported destination settings. Stripe renders the quote and payment UI. Its final source amount may differ from the initial requested amount; records are reconciled against the retrieved session.

Route Stripe directly to the Rust `POST /api/v1/payments/stripe/webhook` endpoint through your HTTPS ingress, preserving the raw body and `Stripe-Signature`. The browser BFF intentionally does not proxy this webhook. Subscribe to `crypto.onramp_session_updated`. Signature checks, mode checks, current-session retrieval, deduplication and locked-wallet validation protect updates. Browser events never mark payment complete. Ambiguous creation retries use the same Stripe key for at most 23 hours; older unresolved records require operational reconciliation.

Stripe Connect stablecoin payouts are fiat-platform-balance-to-crypto-wallet payouts. They are not a generic wallet-to-fiat off-ramp. PHP/GCash/bank cash-out is unavailable until an eligible payout provider and destination corridor are selected and integrated. Do not present this as working Stripe cash-out.

## Verification

- `pnpm --dir apps/web type-check`
- `pnpm --dir apps/web lint`
- `pnpm --dir apps/web test`
- `pnpm --dir apps/web build`
- `cargo test --manifest-path services/api/Cargo.toml`
- `cargo build --manifest-path services/api/Cargo.toml`
- With an **empty disposable** MariaDB database named `uwifin_test` and port 8080 free: `cd apps/web && node tests/provider-integration.mjs`. Override `TEST_DATABASE_URL` if needed. The test creates synthetic users/payments/wallets and runs a temporary API plus local provider transport fixtures, then stops them. Reset the test database before repeating. Never point this test at production or the development preview database.

The integration fixture checks request/response contracts and our security boundaries. It does not replace Alchemy testnet or Stripe approved test-mode verification. The CodeRabbit emulate v0.0.1 catalog was inspected: it provides no Alchemy or Stripe Crypto Onramp service, so those operations cannot be certified using that emulator.
