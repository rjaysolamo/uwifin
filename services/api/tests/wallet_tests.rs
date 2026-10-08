use uwifin_api::{wallet_validation::{validate_wallet_address,validate_network}, wallet::verify_ownership};
#[test]
fn rejects_invalid_addresses_and_networks() {
    assert!(validate_wallet_address("0x1111111111111111111111111111111111111111").is_ok());
    for value in ["", "0x0000000000000000000000000000000000000000", "0x123", "💰"] { assert!(validate_wallet_address(value).is_err()); }
    assert!(validate_network("base", "base").is_ok());
    assert!(validate_network("base", "base-sepolia").is_err());
    assert!(!verify_ownership("challenge", "0xdead", "0x1111111111111111111111111111111111111111"));
}
