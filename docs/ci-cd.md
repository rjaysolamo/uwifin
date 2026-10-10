# UwiFin CI/CD

## Continuous integration

The GitHub Actions workflow at [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)
runs on pull requests targeting `main`, pushes to `main`, and manual dispatches.
It separates checks into three independent jobs, using Node.js 24.14.1, the pnpm
version declared in `package.json`, and Rust 1.90.0.

### Web checks

- Install the pnpm workspace from the committed lockfile (`pnpm install --frozen-lockfile`).
- Check Prettier formatting without rewriting files (`pnpm format:web:check`).
- Run ESLint, TypeScript type checking, tests, and the Next.js production build.

### Rust API checks

- Check Rust formatting without rewriting files (`cargo fmt --manifest-path services/api/Cargo.toml -- --check`).
- Run `cargo check`, `cargo test`, and `cargo build` against the API manifest with `--locked`.

### Provider integration checks

The integration job installs the workspace, builds the API, starts the repository's
MariaDB Docker Compose service, waits for health, and prepares an isolated
`uwifin_test` database. It runs `apps/web/tests/provider-integration.mjs` from
`apps/web`, prints database diagnostics on failure, and tears down the database
and its volume even when a prior step fails.

The test harness starts a temporary API and local HTTP provider fixtures. It
overrides Alchemy and Stripe URLs with loopback endpoints and supplies synthetic
test keys. Its Base chain identifiers describe fixture data; no production
blockchain transaction or real transfer is initiated. Do not inject live provider
credentials, repository secrets, or production database URLs into these jobs.
These fixtures verify application boundaries; they do not certify real provider
compatibility. See [wallet verification](wallet-operations.md#verification).

## Dependency updates

[`.github/dependabot.yml`](../.github/dependabot.yml) checks the root pnpm workspace,
the Rust API dependencies, and GitHub Actions weekly. Review each update PR and
run the checks before merging. Do not enable automatic merging for
security-sensitive dependencies without review. Existing dependency decisions
and outstanding advisory findings are recorded in [dependency decisions](dependencies.md).

## Required repository settings

In GitHub, open **Settings → Rules → Rulesets** (or **Settings → Branches** on
repositories using classic protection rules) and protect `main`:

- Require a pull request before merging.
- Require **Web checks**, **Rust API checks**, and **Provider integration checks** to pass.
- Require branches to be up to date if appropriate for the team's workflow.
- Restrict direct pushes to `main` where practical.

Workflow files alone cannot enforce these repository-level protections. Confirm
the exact status-check names in the Actions UI after the first successful run.

## Deployment status

This configuration intentionally does not deploy to staging or production. The
repository's hosting target, deployment credentials, environment configuration,
and release/rollback process must be verified before adding a deployment workflow.
Never add production secrets to repository files or expose them to pull-request
workflows. The [deployment guide](deployment.md) and
[production operations](production.md) describe the existing deployment guidance
and release requirements.

## Troubleshooting

- If frozen installation fails, check that `pnpm-lock.yaml` matches workspace
  manifests; do not remove the frozen-lockfile check to bypass the failure.
- If formatting fails, run `pnpm format` locally (with Rust 1.90.0 available),
  review the diff, and commit intentional formatting changes.
- If provider integration tests fail, inspect service health, the database logs
  in the failed job, and test configuration. Do not switch tests to live
  payment-provider credentials.
