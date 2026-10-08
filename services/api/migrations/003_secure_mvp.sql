ALTER TABLE users ADD COLUMN IF NOT EXISTS name VARCHAR(60) NOT NULL DEFAULT 'UwiFin user';
ALTER TABLE wallets ADD COLUMN IF NOT EXISTS verified_at TIMESTAMP NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128) CHARACTER SET ascii COLLATE ascii_bin NULL;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS request_fingerprint CHAR(64) NULL;
CREATE UNIQUE INDEX IF NOT EXISTS unique_transaction_idempotency ON transactions (user_id, idempotency_key);
ALTER TABLE payments ADD COLUMN IF NOT EXISTS last_event_created BIGINT NOT NULL DEFAULT 0;
CREATE TABLE IF NOT EXISTS webhook_events (
    id VARCHAR(255) PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Supersede the scaffold's unsupported/incorrect asset seed with one test network.
UPDATE assets SET is_active = FALSE;
INSERT INTO assets (id, symbol, contract_address, network, decimals, is_active, created_at)
VALUES ('base-sepolia-usdc', 'USDC', '0x036CbD53842c5426634e7929541eC2318f3dCF7e', 'base-sepolia', 6, TRUE, NOW())
ON DUPLICATE KEY UPDATE is_active = TRUE, decimals = 6;
