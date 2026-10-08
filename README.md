# UwiFin

UwiFin is a Web3-enabled financial application for simple cross-border transactions, with an MVP focused on secure wallets, account abstraction, fiat onboarding, and transaction tracking.

## Stack

- Frontend: Next.js + TypeScript
- Backend: Rust + Axum
- Database: MariaDB
- Blockchain: EVM with Alchemy
- Payments: Stripe

## Monorepo structure

- `apps/web` — Next.js web client
- `services/api` — Rust Axum API
- `packages/shared` — shared types and utilities
- `docs` — product and engineering docs

## Getting started

1. Install dependencies:
   - Node.js 20.9+
   - pnpm
   - Rust
   - Docker

2. Start the database:
   ```bash
   docker compose up -d
   ```

3. Install workspace dependencies:
   ```bash
   pnpm install
   ```

4. Run the web app:
   ```bash
   pnpm dev:web
   ```

5. Run the API:
   ```bash
   cargo run --manifest-path services/api/Cargo.toml
   ```

## Notes

This repository intentionally keeps the initial architecture simple and production-minded, with no unnecessary service sprawl or custom blockchain infrastructure in the MVP.

## Real wallet setup

See [Wallet operation and activation](docs/wallet-operations.md) for environment configuration, signing, sponsorship, Stripe setup, verification commands, and current release requirements. Copy both environment examples before starting. The wallet uses real provider APIs; absent configuration is displayed as unavailable. Real Alchemy testnet and approved Stripe test-mode checks are required before live activation. Fiat cash-out requires a supported payout partner and is not currently enabled.
