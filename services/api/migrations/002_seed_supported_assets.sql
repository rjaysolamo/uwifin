INSERT INTO assets (id, symbol, contract_address, network, decimals, is_active, created_at)
VALUES (
    UUID(),
    'USDC',
    '0x833589fCD6eDb6E08f4cD9eD7cFB5B0F5d7eFf3d',
    'base',
    6,
    TRUE,
    NOW()
)
ON DUPLICATE KEY UPDATE
    symbol = VALUES(symbol),
    contract_address = VALUES(contract_address),
    network = VALUES(network),
    decimals = VALUES(decimals),
    is_active = VALUES(is_active),
    created_at = VALUES(created_at);
