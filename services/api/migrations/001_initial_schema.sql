CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_login_at TIMESTAMP NULL,
    INDEX idx_users_email (email),
    INDEX idx_users_status (status)
);

CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    session_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NULL,
    CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_sessions_user_id (user_id),
    INDEX idx_sessions_expires_at (expires_at)
);

CREATE TABLE IF NOT EXISTS wallets (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    address VARCHAR(255) NOT NULL,
    network VARCHAR(50) NOT NULL,
    wallet_type VARCHAR(50) NOT NULL DEFAULT 'smart_account',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    CONSTRAINT fk_wallets_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_wallet_user_address_network (user_id, address, network),
    INDEX idx_wallets_user_id (user_id),
    INDEX idx_wallets_address (address)
);

CREATE TABLE IF NOT EXISTS assets (
    id VARCHAR(36) PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    contract_address VARCHAR(255) NOT NULL,
    network VARCHAR(50) NOT NULL,
    decimals INT NOT NULL DEFAULT 18,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL,
    UNIQUE KEY unique_asset_network_contract (network, contract_address),
    INDEX idx_assets_symbol (symbol),
    INDEX idx_assets_network (network),
    INDEX idx_assets_active (is_active)
);

CREATE TABLE IF NOT EXISTS transactions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    wallet_id VARCHAR(36) NOT NULL,
    network VARCHAR(50) NOT NULL,
    tx_hash VARCHAR(255) NULL,
    user_operation_hash VARCHAR(255) NULL,
    sender VARCHAR(255) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    asset_id VARCHAR(36) NOT NULL,
    amount_atomic BIGINT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'created',
    gas_used BIGINT NULL,
    gas_price BIGINT NULL,
    error_code VARCHAR(100) NULL,
    error_message TEXT NULL,
    created_at TIMESTAMP NOT NULL,
    submitted_at TIMESTAMP NULL,
    confirmed_at TIMESTAMP NULL,
    updated_at TIMESTAMP NOT NULL,
    CONSTRAINT fk_transactions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_transactions_wallet FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON DELETE CASCADE,
    CONSTRAINT fk_transactions_asset FOREIGN KEY (asset_id) REFERENCES assets(id),
    INDEX idx_transactions_user_id (user_id),
    INDEX idx_transactions_tx_hash (tx_hash),
    INDEX idx_transactions_status (status),
    INDEX idx_transactions_user_created (user_id, created_at)
);

CREATE TABLE IF NOT EXISTS payments (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_reference VARCHAR(255) NOT NULL,
    payment_type VARCHAR(50) NOT NULL,
    fiat_currency VARCHAR(10) NOT NULL,
    fiat_amount_minor BIGINT NOT NULL,
    crypto_asset VARCHAR(20) NOT NULL,
    crypto_amount_atomic BIGINT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    CONSTRAINT fk_payments_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_provider_reference (provider, provider_reference),
    INDEX idx_payments_user_id (user_id),
    INDEX idx_payments_status (status)
);

CREATE TABLE IF NOT EXISTS audit_events (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NULL,
    event_type VARCHAR(100) NOT NULL,
    request_id VARCHAR(36) NOT NULL,
    metadata JSON NULL,
    created_at TIMESTAMP NOT NULL,
    CONSTRAINT fk_audit_events_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_audit_events_user_id (user_id),
    INDEX idx_audit_events_created_at (created_at)
);
