use serde_json::json;
use uwifin_api::{transaction::{parse_amount, attach_signature, validate_prepared, TransactionStatus}, alchemy::{receipt_state, contains_transfer, ReceiptState}, payments::verify_signature};
#[test]
fn exact_amounts_and_overflow() {
    assert_eq!(parse_amount("1.000001").unwrap(), 1000001);
    assert_eq!(parse_amount("9223372036854.775807").unwrap(), i64::MAX);
    for amount in ["0", "-1", "1e6", "1.0000001", "9223372036854.775808", "1.", ".1"] { assert!(parse_amount(amount).is_err(), "{amount}"); }
}
#[test]
fn terminal_states_cannot_be_reopened() {
    use TransactionStatus::*;
    assert!(Pending.can_transition_to(Confirmed));
    assert!(!Created.can_transition_to(Confirmed));
    assert!(!Confirmed.can_transition_to(Submitted));
    assert!(!Failed.can_transition_to(Pending));
}
#[test]
fn sponsorship_and_signed_payload_are_bound() {
    let sender = "0x1111111111111111111111111111111111111111";
    let mut prepared = json!({"type":"user-operation-v070", "chainId":"0x2105", "data":{"sender":sender,"paymaster":sender,"nonce":"0x0"},"signatureRequest":{"type":"personal_sign"},"feePayment":{"sponsored":true}});
    assert!(validate_prepared(&prepared, sender, 8453).is_ok());
    assert!(validate_prepared(&prepared, sender, 84532).is_err());
    let mut signed = prepared.clone(); signed["signature"] = json!({"type":"secp256k1","data":format!("0x{}", "11".repeat(65))});
    assert!(attach_signature(&prepared, &signed).is_ok());
    signed["data"]["nonce"] = json!("0x1");
    assert!(attach_signature(&prepared, &signed).is_err());
    prepared["feePayment"]["sponsored"] = json!(false);
    assert!(validate_prepared(&prepared, sender, 8453).is_err());
    prepared["data"]["paymaster"] = json!(""); prepared["data"]["paymasterAndData"] = json!("💰".repeat(30));
    assert!(validate_prepared(&prepared, sender, 8453).is_err());
}
#[test]
fn successful_http_is_not_settlement() {
    assert_eq!(receipt_state(&json!(null)).unwrap(), ReceiptState::Pending);
    assert!(receipt_state(&json!({"status":200})).is_err());
    assert_eq!(receipt_state(&json!({"status":"0x0"})).unwrap(), ReceiptState::Failed);
    let sender = "0x1111111111111111111111111111111111111111";
    let recipient = "0x2222222222222222222222222222222222222222";
    let token = "0x3333333333333333333333333333333333333333";
    let receipt = json!({"logs":[{"address": token, "topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",format!("0x{:0>64}",&sender[2..]),format!("0x{:0>64}",&recipient[2..])],"data":"0x0f4240"}]});
    assert!(contains_transfer(&receipt,token,sender,recipient,1000000));
    assert!(!contains_transfer(&receipt,token,sender,recipient,1000001));
    assert!(!contains_transfer(&receipt,recipient,sender,recipient,1000000));
    assert!(!contains_transfer(&receipt,token,"",recipient,1000000));
}
#[test]
fn webhook_signature_replay_and_tampering() {
    use hmac::{Hmac, Mac}; use sha2::Sha256;
    let secret = "synthetic-webhook-secret"; let payload = br#"{"id":"evt_fixture"}"#;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap(); mac.update(b"1000."); mac.update(payload);
    let signature = format!("t=1000,v1={}", hex::encode(mac.finalize().into_bytes()));
    assert!(verify_signature(secret,&signature,payload,1001));
    assert!(!verify_signature(secret,&signature,payload,1301));
    assert!(!verify_signature(secret,&signature,b"tampered",1001));
    assert!(!verify_signature(secret,&format!("{signature},t=1000"),payload,1001));
}
