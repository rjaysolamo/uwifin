-- Additive remittance records. Existing wallet/transaction history is preserved.
CREATE TABLE recipients (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    relationship VARCHAR(40) NOT NULL,
    payout_method VARCHAR(30) NOT NULL,
    version INT UNSIGNED NOT NULL DEFAULT 1,
    provider_recipient_reference VARCHAR(255) NULL,
    provider_account VARCHAR(100) NULL,
    masked_destination VARCHAR(100) NULL,
    verified_until DATETIME(6) NULL,
    archived_at DATETIME(6) NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE KEY recipient_owner (id, user_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX recipient_user (user_id, created_at)
);
CREATE TABLE remittance_quotes (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    recipient_id VARCHAR(36) NOT NULL,
    recipient_version INT UNSIGNED NOT NULL,
    provider_bundle VARCHAR(100) NOT NULL,
    quote JSON NOT NULL,
    expires_at DATETIME(6) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE KEY quote_owner (id, user_id),
    FOREIGN KEY (recipient_id, user_id) REFERENCES recipients(id, user_id),
    INDEX quotes_expiry (expires_at)
);
CREATE TABLE remittances (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    quote_id VARCHAR(36) NOT NULL UNIQUE,
    recipient_id VARCHAR(36) NOT NULL,
    recipient_snapshot JSON NOT NULL,
    status VARCHAR(40) NOT NULL,
    version INT UNSIGNED NOT NULL DEFAULT 1,
    hold_reason VARCHAR(80) NULL,
    provider_bundle VARCHAR(100) NOT NULL,
    idempotency_key VARCHAR(128) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
    request_fingerprint CHAR(64) NOT NULL,
    accepted_terms_version VARCHAR(40) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE KEY remittance_idempotency (user_id, idempotency_key),
    FOREIGN KEY (quote_id, user_id) REFERENCES remittance_quotes(id, user_id),
    FOREIGN KEY (recipient_id, user_id) REFERENCES recipients(id, user_id),
    INDEX remittance_history (user_id, created_at, id),
    INDEX remittance_pending (status, updated_at)
);
CREATE TABLE remittance_events (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    remittance_id VARCHAR(36) NOT NULL,
    from_state VARCHAR(40) NULL,
    to_state VARCHAR(40) NOT NULL,
    version INT UNSIGNED NOT NULL,
    reason VARCHAR(100) NOT NULL,
    evidence_reference VARCHAR(255) NULL,
    actor VARCHAR(100) NOT NULL,
    request_id VARCHAR(36) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE KEY remittance_event_version (remittance_id, version),
    FOREIGN KEY (remittance_id) REFERENCES remittances(id)
);
CREATE TABLE remittance_operations (
    id VARCHAR(36) PRIMARY KEY,
    remittance_id VARCHAR(36) NOT NULL,
    stage VARCHAR(30) NOT NULL,
    provider_account VARCHAR(100) NOT NULL,
    provider_reference VARCHAR(255) NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'CREATED',
    request_fingerprint CHAR(64) NOT NULL,
    -- Durable command contains only internal IDs and the accepted quote, no keys or PII.
    command JSON NOT NULL,
    evidence JSON NULL,
    attempts INT UNSIGNED NOT NULL DEFAULT 0,
    retry_after DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    lease_token VARCHAR(36) NULL,
    lease_until DATETIME(6) NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE KEY operation_stage (remittance_id, stage),
    UNIQUE KEY provider_operation (provider_account, provider_reference),
    FOREIGN KEY (remittance_id) REFERENCES remittances(id),
    INDEX operation_jobs (status, retry_after, lease_until)
);
CREATE TABLE remittance_webhooks (
    id VARCHAR(36) PRIMARY KEY,
    provider_account VARCHAR(100) NOT NULL,
    provider_event_id VARCHAR(255) NOT NULL,
    payload_hash CHAR(64) NOT NULL,
    -- Only authenticated, normalized IDs are stored; raw financial/PII payload is not retained.
    provider_reference VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'RECEIVED',
    received_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    processed_at DATETIME(6) NULL,
    UNIQUE KEY webhook_dedup (provider_account, provider_event_id),
    INDEX unmatched_webhooks (status, received_at)
);
CREATE TABLE remittance_support_cases (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    remittance_id VARCHAR(36) NULL,
    category VARCHAR(40) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN',
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (remittance_id) REFERENCES remittances(id),
    INDEX support_user (user_id, created_at)
);
