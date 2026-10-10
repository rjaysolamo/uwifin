ALTER TABLE users ADD COLUMN role VARCHAR(10) NOT NULL DEFAULT 'user';
CREATE TABLE networks (
    id VARCHAR(50) PRIMARY KEY,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);
INSERT INTO networks (id) VALUES ('base'), ('base-sepolia');
ALTER TABLE transactions ADD COLUMN kind VARCHAR(10) NOT NULL DEFAULT 'sent';
ALTER TABLE transactions ADD COLUMN external_reference VARCHAR(255) NULL;
CREATE UNIQUE INDEX unique_incoming_transfer ON transactions (wallet_id, external_reference);
ALTER TABLE wallets ADD COLUMN history_block BIGINT UNSIGNED NOT NULL DEFAULT 0;
ALTER TABLE wallets ADD COLUMN history_end BIGINT UNSIGNED NULL;
ALTER TABLE wallets ADD COLUMN history_page VARCHAR(255) NULL;
ALTER TABLE wallets ADD COLUMN history_checked_at TIMESTAMP NULL;
