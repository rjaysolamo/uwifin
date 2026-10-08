ALTER TABLE wallets ADD COLUMN IF NOT EXISTS signer_address VARCHAR(42) NULL;
ALTER TABLE wallets ADD COLUMN IF NOT EXISTS provider_account_id VARCHAR(64) NULL;
CREATE UNIQUE INDEX IF NOT EXISTS unique_verified_wallet ON wallets (address, network);
CREATE TABLE wallet_challenges (
    id CHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    session_hash CHAR(64) NOT NULL,
    signer_address VARCHAR(42) NOT NULL,
    message TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    consumed_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_wallet_challenge_expiry (expires_at)
);
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS prepared_call JSON NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS signed_call JSON NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS prepared_expires_at TIMESTAMP NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS call_id VARCHAR(255) NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS active_wallet_id VARCHAR(36)
    GENERATED ALWAYS AS (CASE WHEN status IN ('VALIDATING','READY','SUBMITTED','PENDING') THEN wallet_id ELSE NULL END) PERSISTENT;
CREATE UNIQUE INDEX IF NOT EXISTS unique_active_wallet_transfer ON transactions (active_wallet_id);
CREATE UNIQUE INDEX IF NOT EXISTS unique_provider_call ON transactions (call_id);
ALTER TABLE payments MODIFY provider_reference VARCHAR(255) NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS wallet_id VARCHAR(36) NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS network VARCHAR(50) NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128) CHARACTER SET ascii COLLATE ascii_bin NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS request_fingerprint CHAR(64) NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS requested_amount_minor BIGINT NULL;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS error_code VARCHAR(100) NULL;
CREATE UNIQUE INDEX IF NOT EXISTS unique_payment_idempotency ON payments (user_id, idempotency_key);
INSERT INTO assets (id, symbol, contract_address, network, decimals, is_active, created_at)
VALUES ('base-usdc', 'USDC', '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913', 'base', 6, TRUE, NOW());
