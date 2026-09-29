use std::collections::HashMap;
use std::collections::HashSet;

use aevo_sdk::config::Env;
use aevo_sdk::models::BuilderFields;
use aevo_sdk::signing::{self, OrderToSign};
use serde_json::Value;

fn vectors() -> Value {
    serde_json::from_str(include_str!("vectors/vectors.json")).unwrap()
}

fn keys(root: &Value) -> HashMap<String, (String, String)> {
    root["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|key| {
            (
                s(&key["id"]).to_string(),
                (
                    s(&key["private_key"]).to_string(),
                    s(&key["address"]).to_string(),
                ),
            )
        })
        .collect()
}

fn key_by_address<'a>(keys: &'a HashMap<String, (String, String)>, address: &str) -> &'a str {
    keys.values()
        .find(|(_, candidate)| candidate.eq_ignore_ascii_case(address))
        .map(|(private_key, _)| private_key.as_str())
        .unwrap()
}

fn env(id: &str) -> Env {
    match id {
        "mainnet" => Env::Mainnet,
        "testnet" => Env::Testnet,
        other => panic!("unknown env {other}"),
    }
}

fn s(value: &Value) -> &str {
    value.as_str().unwrap()
}

fn require_non_empty_array<'a>(value: &'a Value, path: &str) -> &'a Vec<Value> {
    let array = value
        .as_array()
        .unwrap_or_else(|| panic!("{path} must be an array"));
    assert!(!array.is_empty(), "{path} must not be empty");
    array
}

fn raw_to_rate(raw: &str) -> String {
    let mut digits = raw.to_string();
    while digits.len() <= 6 {
        digits.insert(0, '0');
    }
    let split = digits.len() - 6;
    let whole = &digits[..split];
    let mut frac = digits[split..].to_string();
    while frac.ends_with('0') {
        frac.pop();
    }
    if frac.is_empty() {
        whole.trim_start_matches('0').to_string().if_empty("0")
    } else {
        format!("{}.{}", whole.trim_start_matches('0').if_empty("0"), frac)
    }
}

trait EmptyDefault {
    fn if_empty(self, default: &str) -> String;
}

impl EmptyDefault for String {
    fn if_empty(self, default: &str) -> String {
        if self.is_empty() {
            default.to_string()
        } else {
            self
        }
    }
}

impl EmptyDefault for &str {
    fn if_empty(self, default: &str) -> String {
        if self.is_empty() {
            default.to_string()
        } else {
            self.to_string()
        }
    }
}

#[test]
fn vectors_file_contains_every_expected_domain_and_kind() {
    let root = vectors();
    assert_eq!(
        s(&root["schema_version"]),
        "aevo-sdk-signing-vectors/v1",
        "unexpected vectors schema"
    );

    let expected_domains = ["mainnet", "testnet"];
    let expected_vector_kinds = [
        "order_plain",
        "order_builder",
        "register",
        "sign_key",
        "approve_builder",
        "withdraw",
        "transfer",
    ];

    let domains = require_non_empty_array(&root["domains"], "domains");
    let domain_ids: HashSet<&str> = domains.iter().map(|domain| s(&domain["id"])).collect();
    for expected_domain in expected_domains {
        assert!(
            domain_ids.contains(expected_domain),
            "vectors.json is missing domain {expected_domain}"
        );
    }

    for domain in domains {
        let domain_id = s(&domain["id"]);
        if !expected_domains.contains(&domain_id) {
            continue;
        }
        for kind in expected_vector_kinds {
            require_non_empty_array(
                &domain["vectors"][kind],
                &format!("domains.{domain_id}.vectors.{kind}"),
            );
        }
    }

    for kind in ["rest", "websocket"] {
        require_non_empty_array(&root["hmac"][kind], &format!("hmac.{kind}"));
    }
}

#[test]
fn every_eip712_vector_matches_hash_and_signature() {
    let root = vectors();
    let keys = keys(&root);
    let mut checked = 0usize;

    for domain in root["domains"].as_array().unwrap() {
        let env = env(s(&domain["id"]));
        assert_eq!(
            signing::domain_separator(env),
            s(&domain["domain_separator"])
        );
        let vectors = &domain["vectors"];

        for case in vectors["order_plain"].as_array().unwrap() {
            let api = &case["api"];
            let key = &keys[s(&case["signer_key"])].0;
            let order = OrderToSign {
                maker: s(&api["maker"]).to_string(),
                is_buy: api["is_buy"].as_bool().unwrap(),
                limit_price: s(&api["limit_price"]).to_string(),
                amount: s(&api["amount"]).to_string(),
                instrument: s(&api["instrument"]).to_string(),
                salt: s(&api["salt"]).to_string(),
                timestamp: s(&api["timestamp"]).to_string(),
                post_only: false,
                reduce_only: false,
                close_position: false,
                partial_position: false,
                stop: None,
                trigger: None,
                time_in_force: None,
                mmp: false,
                builder: None,
            };
            let signed = signing::sign_order(env, key, order).unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["order_builder"].as_array().unwrap() {
            let api = &case["api"];
            let key = &keys[s(&case["signer_key"])].0;
            let order = OrderToSign {
                maker: s(&api["maker"]).to_string(),
                is_buy: api["is_buy"].as_bool().unwrap(),
                limit_price: s(&api["limit_price"]).to_string(),
                amount: s(&api["amount"]).to_string(),
                instrument: s(&api["instrument"]).to_string(),
                salt: s(&api["salt"]).to_string(),
                timestamp: s(&api["timestamp"]).to_string(),
                post_only: false,
                reduce_only: false,
                close_position: false,
                partial_position: false,
                stop: None,
                trigger: None,
                time_in_force: None,
                mmp: false,
                builder: Some(BuilderFields {
                    builder_id: s(&api["builder_id"]).to_string(),
                    builder_fee_rate: s(&api["builder_fee_rate"]).to_string(),
                }),
            };
            let signed = signing::sign_order(env, key, order).unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["register"].as_array().unwrap() {
            let wallet_key = &keys[s(&case["signer_key"])].0;
            let signing_key = key_by_address(&keys, s(&case["key_address"]));
            let hashed = s(&case["variant"]) == "hashed_key";
            let signed = signing::sign_register_for_key(
                env,
                s(&case["signer"]),
                s(&case["key_address"]),
                wallet_key,
                signing_key,
                Some(s(&case["message"]["expiry"])),
                hashed,
            )
            .unwrap();
            assert_eq!(
                signed.register_hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.register_signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.register_personal_signature,
                s(&case["personal_signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["sign_key"].as_array().unwrap() {
            let key = &keys[s(&case["signer_key"])].0;
            let signed = signing::sign_sign_key(env, key, s(&case["message"]["account"])).unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["approve_builder"].as_array().unwrap() {
            let key = &keys[s(&case["signer_key"])].0;
            let max_fee_rate_raw = s(&case["message"]["maxFeeRate"]);
            let signed = signing::sign_approve_builder_raw(
                env,
                key,
                s(&case["message"]["account"]),
                s(&case["message"]["builderId"]),
                max_fee_rate_raw,
                s(&case["message"]["nonce"]),
                &raw_to_rate(max_fee_rate_raw),
            )
            .unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["withdraw"].as_array().unwrap() {
            let api = &case["api"];
            let key = &keys[s(&case["signer_key"])].0;
            let signed = signing::sign_withdraw_raw(
                env,
                key,
                s(&api["account"]),
                s(&api["collateral"]),
                s(&api["to"]),
                s(&api["amount"]),
                s(&api["salt"]),
                Some(s(&api["data"])),
                api.get("recipient").and_then(Value::as_str),
            )
            .unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }

        for case in vectors["transfer"].as_array().unwrap() {
            let api = &case["api"];
            let key = &keys[s(&case["signer_key"])].0;
            let signed = signing::sign_transfer_raw(
                env,
                key,
                s(&api["account"]),
                s(&api["collateral"]),
                s(&api["to"]),
                s(&api["amount"]),
                s(&api["salt"]),
                None,
                None,
            )
            .unwrap();
            assert_eq!(
                signed.hash,
                s(&case["eip712"]["digest"]),
                "{}",
                s(&case["id"])
            );
            assert_eq!(
                signed.signature,
                s(&case["signature"]),
                "{}",
                s(&case["id"])
            );
            checked += 1;
        }
    }

    assert_eq!(checked, 34);
}

#[test]
fn every_hmac_vector_matches() {
    let root = vectors();
    let hmac = &root["hmac"];
    let mut checked = 0usize;

    for case in hmac["rest"].as_array().unwrap() {
        let signature = signing::hmac_signature(
            s(&case["key"]),
            s(&case["secret"]),
            s(&case["timestamp"]),
            s(&case["method"]),
            s(&case["signed_path"]),
            s(&case["body"]),
        )
        .unwrap();
        assert_eq!(signature, s(&case["signature"]), "{}", s(&case["id"]));
        checked += 1;
    }

    for case in hmac["websocket"].as_array().unwrap() {
        let signature = signing::websocket_hmac_signature(
            s(&case["key"]),
            s(&case["secret"]),
            s(&case["timestamp"]),
            s(&case["signed_path"]),
            s(&case["body"]),
        )
        .unwrap();
        assert_eq!(signature, s(&case["signature"]), "{}", s(&case["id"]));
        checked += 1;
    }

    assert_eq!(checked, 6);
}
