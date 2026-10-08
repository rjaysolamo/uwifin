# Dependency decisions

Exact JavaScript versions are pinned in apps/web/package.json and pnpm-lock.yaml; Rust resolutions are pinned in services/api/Cargo.lock. Install with locked resolution. Provider API compatibility must also be verified against an enabled account before release.

| Library | Tested version | Purpose and choice | Official source / alternative | Security and license notes |
| --- | --- | --- | --- | --- |
| Next.js / React | 16.4.0 / 19.3.0 | App Router, session BFF, UI | vercel/next.js; facebook/react | MIT; keep server-only secrets outside public variables |
| viem | 2.57.4 | EIP-1193 wallet client, signing account and chain selection | wevm/viem; ethers is an alternative | MIT; no custom private-key store |
| Alchemy wallet-apis | 5.3.0 | Official Wallet API signature handling | alchemyplatform/aa-sdk; supersedes Account Kit v4 for this flow | MIT; provider credentials stay server-side; fixed transfer intents |
| Stripe crypto | 1.1.3 | Official embedded Crypto Onramp loader | stripe/crypto-js; hosted flow is an alternative | MIT; requires approved provider access |
| Stripe.js | 1.54.2 | Satisfies crypto package's declared ^1.46.0 peer | stripe/stripe-js | MIT; pinned to compatible peer, not independent latest; runtime scripts load from Stripe |
| QRCode | 1.5.4 | Encode actual receive address | soldair/node-qrcode | MIT; no network calls |
| Alloy primitives | 0.8.26 | EIP-191 signature recovery | alloy-rs/core | MIT/Apache-2.0; k256 feature; no custom cryptography |
| Axum / Tokio / SQLx / Reqwest | Cargo.lock | One HTTP API, MariaDB persistence, provider HTTP | tokio-rs/axum; tokio-rs/tokio; launchbadge/sqlx; seanmonstar/reqwest | Use lockfile; runtime SQL tests cover migrations and queries |
| Argon2 / HMAC / SHA2 | Cargo.lock | Password hashes, webhook signatures, session hashes | RustCrypto | Standard library implementations; never log keys or tokens |

Official provider docs and installed SDK source were inspected for request/response shapes and signing behavior. `pnpm audit --prod --json` reported zero known advisories in this sandbox on 2026-10-08. This is a point-in-time registry result, not a security certification. A Rust advisory scan and independent security review remain production gates. No custom Solidity contract is needed for direct USDC transfers.
