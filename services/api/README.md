# Rust API

UwiFin backend API built with Axum and Tokio.

## Development

```bash
cargo run
```

Server runs on `http://localhost:8080`

## Structure

- `src/main.rs` — entry point and routing
- `src/config.rs` — configuration from environment
- `src/state.rs` — shared application state
- `src/error.rs` — error types and responses
- `src/handlers/` — HTTP request handlers
- `src/db.rs` — database migrations and initialization

## Database

MariaDB is required. Start with:

```bash
docker compose up -d
```

Migrations run automatically on startup.
