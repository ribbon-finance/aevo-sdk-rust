use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ethers_core::abi::{encode, Token};
use ethers_core::types::transaction::eip712::{Eip712, TypedData};
use ethers_core::types::{Address, H256, U256};
use ethers_core::utils::{hash_message, keccak256, to_checksum};
use ethers_signers::{LocalWallet, Signer};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;

use crate::config::Env;
use crate::error::{AevoError, Result};
use crate::models::{
    ApproveBuilderRequest, BuilderFields, RegisterRequest, SignedOrder, SignedTransfer,
    SignedWithdraw,
};

type HmacSha256 = Hmac<Sha256>;

const SCALE_6: u64 = 1_000_000;
pub const MAX_UINT256_STR: &str =
    "115792089237316195423570985008687907853269984665640564039457584007913129639935";

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureResult<T> {
    pub payload: T,
    pub hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSignature {
    pub payload: RegisterRequest,
    pub register_hash: String,
    pub sign_key_hash: String,
    pub register_signature: String,
    pub register_personal_signature: String,
    pub sign_key_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderToSign {
    pub maker: String,
    pub is_buy: bool,
    pub limit_price: String,
    pub amount: String,
    pub instrument: String,
    pub salt: String,
    pub timestamp: String,
    pub post_only: bool,
    pub reduce_only: bool,
    pub close_position: bool,
    pub partial_position: bool,
    pub stop: Option<String>,
    pub trigger: Option<String>,
    pub time_in_force: Option<String>,
    pub mmp: bool,
    pub builder: Option<BuilderFields>,
}

impl OrderToSign {
    pub fn builder() -> OrderBuilder {
        OrderBuilder::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderBuilder {
    maker: Option<String>,
    is_buy: Option<bool>,
    limit_price: Option<String>,
    amount: Option<String>,
    instrument: Option<String>,
    salt: Option<String>,
    timestamp: Option<String>,
    post_only: bool,
    reduce_only: bool,
    close_position: bool,
    partial_position: bool,
    stop: Option<String>,
    trigger: Option<String>,
    time_in_force: Option<String>,
    mmp: bool,
    builder: Option<BuilderFields>,
}

impl OrderBuilder {
    pub fn maker(mut self, value: impl Into<String>) -> Self {
        self.maker = Some(value.into());
        self
    }

    pub fn is_buy(mut self, value: bool) -> Self {
        self.is_buy = Some(value);
        self
    }

    pub fn limit_price_raw(mut self, value: impl Into<String>) -> Self {
        self.limit_price = Some(value.into());
        self
    }

    pub fn limit_price(mut self, value: &str) -> Result<Self> {
        self.limit_price = Some(to_raw6(value)?);
        Ok(self)
    }

    pub fn amount_raw(mut self, value: impl Into<String>) -> Self {
        self.amount = Some(value.into());
        self
    }

    pub fn amount(mut self, value: &str) -> Result<Self> {
        self.amount = Some(to_raw6(value)?);
        Ok(self)
    }

    pub fn instrument(mut self, value: impl Into<String>) -> Self {
        self.instrument = Some(value.into());
        self
    }

    pub fn salt(mut self, value: impl Into<String>) -> Self {
        self.salt = Some(value.into());
        self
    }

    pub fn timestamp(mut self, value: impl Into<String>) -> Self {
        self.timestamp = Some(value.into());
        self
    }

    pub fn post_only(mut self, value: bool) -> Self {
        self.post_only = value;
        self
    }

    pub fn reduce_only(mut self, value: bool) -> Self {
        self.reduce_only = value;
        self
    }

    pub fn time_in_force(mut self, value: impl Into<String>) -> Self {
        self.time_in_force = Some(value.into());
        self
    }

    pub fn mmp(mut self, value: bool) -> Self {
        self.mmp = value;
        self
    }

    pub fn stop(mut self, value: impl Into<String>, trigger: impl Into<String>) -> Self {
        self.stop = Some(value.into());
        self.trigger = Some(trigger.into());
        self
    }

    pub fn close_position(mut self, value: bool) -> Self {
        self.close_position = value;
        self
    }

    pub fn partial_position(mut self, value: bool) -> Self {
        self.partial_position = value;
        self
    }

    pub fn builder_fields(
        mut self,
        builder_id: impl Into<String>,
        builder_fee_rate: impl Into<String>,
    ) -> Self {
        self.builder = Some(BuilderFields {
            builder_id: builder_id.into(),
            builder_fee_rate: builder_fee_rate.into(),
        });
        self
    }

    pub fn build(self) -> Result<OrderToSign> {
        Ok(OrderToSign {
            maker: self
                .maker
                .ok_or_else(|| AevoError::InvalidInput("maker is required".into()))?,
            is_buy: self
                .is_buy
                .ok_or_else(|| AevoError::InvalidInput("is_buy is required".into()))?,
            limit_price: self
                .limit_price
                .ok_or_else(|| AevoError::InvalidInput("limit_price is required".into()))?,
            amount: self
                .amount
                .ok_or_else(|| AevoError::InvalidInput("amount is required".into()))?,
            instrument: self
                .instrument
                .ok_or_else(|| AevoError::InvalidInput("instrument is required".into()))?,
            salt: self.salt.unwrap_or_else(|| generate_nonce().to_string()),
            timestamp: self
                .timestamp
                .unwrap_or_else(|| current_unix_timestamp_secs().to_string()),
            post_only: self.post_only,
            reduce_only: self.reduce_only,
            close_position: self.close_position,
            partial_position: self.partial_position,
            stop: self.stop,
            trigger: self.trigger,
            time_in_force: self.time_in_force,
            mmp: self.mmp,
            builder: self.builder,
        })
    }
}

pub fn generate_nonce() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);

    loop {
        let current = NONCE_COUNTER.load(Ordering::SeqCst);
        let next = now.max(current.saturating_add(1));
        if NONCE_COUNTER
            .compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return next;
        }
    }
}

pub fn current_unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub fn current_unix_timestamp_ns() -> u64 {
    generate_nonce()
}

pub fn derive_address(private_key: &str) -> Result<String> {
    let wallet = wallet(private_key)?;
    Ok(to_checksum(&wallet.address(), None))
}

pub fn to_raw6(value: &str) -> Result<String> {
    decimal_to_scaled(value, "value", SCALE_6)
}

pub fn rate_to_raw(value: &str) -> Result<String> {
    decimal_to_scaled(value, "rate", SCALE_6)
}

pub fn bps_to_rate(bps: u32) -> Result<String> {
    let raw = U256::from(bps) * U256::from(100u64);
    raw6_to_decimal(raw)
}

pub fn hmac_signature(
    key: &str,
    secret: &str,
    timestamp_ns: &str,
    method: &str,
    signed_path: &str,
    body: &str,
) -> Result<String> {
    let canonical = hmac_canonical_string(key, timestamp_ns, method, signed_path, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| AevoError::Signing(format!("HMAC key error: {err}")))?;
    mac.update(canonical.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn hmac_canonical_string(
    key: &str,
    timestamp_ns: &str,
    method: &str,
    signed_path: &str,
    body: &str,
) -> String {
    format!(
        "{},{},{},{},{}",
        key, timestamp_ns, method, signed_path, body
    )
}

pub fn websocket_hmac_signature(
    key: &str,
    secret: &str,
    timestamp_ns: &str,
    operation: &str,
    body: &str,
) -> Result<String> {
    hmac_signature(key, secret, timestamp_ns, "ws", operation, body)
}

pub fn domain_separator(env: Env) -> String {
    let domain_type_hash = keccak256(b"EIP712Domain(string name,string version,uint256 chainId)");
    let separator = keccak256(encode(&[
        Token::FixedBytes(domain_type_hash.to_vec()),
        Token::FixedBytes(keccak256(env.domain_name().as_bytes()).to_vec()),
        Token::FixedBytes(keccak256(b"1").to_vec()),
        Token::Uint(U256::from(env.chain_id())),
    ]));
    hex0x(separator)
}

pub fn sign_order(
    env: Env,
    private_key: &str,
    order: OrderToSign,
) -> Result<SignatureResult<SignedOrder>> {
    let maker = normalize_address(&order.maker)?;
    let limit_price = parse_u256(&order.limit_price, "limit_price")?.to_string();
    let amount = parse_u256(&order.amount, "amount")?.to_string();
    let salt = parse_u256(&order.salt, "salt")?.to_string();
    let instrument = parse_u256(&order.instrument, "instrument")?.to_string();
    let timestamp = parse_u256(&order.timestamp, "timestamp")?.to_string();

    let (fields, message) = if let Some(builder) = &order.builder {
        let builder_fee_rate_raw = rate_to_raw(&builder.builder_fee_rate)?;
        (
            builder_order_fields(),
            json!({
                "maker": maker,
                "isBuy": order.is_buy,
                "limitPrice": limit_price,
                "amount": amount,
                "salt": salt,
                "instrument": instrument,
                "timestamp": timestamp,
                "builderId": builder.builder_id,
                "builderFeeRate": builder_fee_rate_raw,
            }),
        )
    } else {
        (
            order_fields(),
            json!({
                "maker": maker,
                "isBuy": order.is_buy,
                "limitPrice": limit_price,
                "amount": amount,
                "salt": salt,
                "instrument": instrument,
                "timestamp": timestamp,
            }),
        )
    };

    let hash = hash_typed_data(env, "Order", fields, message)?;
    let signature = sign_digest(private_key, hash)?;
    let trigger = order.trigger.as_deref().map(to_raw6).transpose()?;

    Ok(SignatureResult {
        payload: SignedOrder {
            maker,
            is_buy: order.is_buy,
            limit_price,
            amount,
            instrument,
            salt,
            timestamp,
            signature: signature.clone(),
            post_only: order.post_only.then_some(true),
            reduce_only: order.reduce_only.then_some(true),
            close_position: order.close_position.then_some(true),
            partial_position: order.partial_position.then_some(true),
            stop: order.stop,
            trigger,
            time_in_force: normalize_tif(order.time_in_force.as_deref()),
            mmp: order.mmp.then_some(true),
            builder_id: order
                .builder
                .as_ref()
                .map(|builder| builder.builder_id.clone()),
            builder_fee_rate: order
                .builder
                .as_ref()
                .map(|builder| builder.builder_fee_rate.clone()),
        },
        hash: hex0x(hash),
        signature,
    })
}

pub fn sign_register(
    env: Env,
    wallet_private_key: &str,
    signing_key_private_key: &str,
    expiry: Option<&str>,
) -> Result<RegisterSignature> {
    let account = derive_address(wallet_private_key)?;
    let signing_key = derive_address(signing_key_private_key)?;
    sign_register_for_key(
        env,
        &account,
        &signing_key,
        wallet_private_key,
        signing_key_private_key,
        expiry,
        false,
    )
}

pub fn sign_register_for_key(
    env: Env,
    account: &str,
    signing_key: &str,
    wallet_private_key: &str,
    signing_key_private_key: &str,
    expiry: Option<&str>,
    hashed_key: bool,
) -> Result<RegisterSignature> {
    let account = normalize_address(account)?;
    let signing_key = normalize_address(signing_key)?;
    let expiry = parse_u256(expiry.unwrap_or(MAX_UINT256_STR), "expiry")?.to_string();
    let register_fields = if hashed_key {
        register_hashed_fields()
    } else {
        register_fields()
    };
    let key_value = if hashed_key {
        hex0x(keccak256(parse_address(&signing_key)?.as_bytes()))
    } else {
        signing_key.clone()
    };
    let register_hash = hash_typed_data(
        env,
        "Register",
        register_fields,
        json!({
            "key": key_value,
            "expiry": expiry,
        }),
    )?;
    let sign_key_hash = sign_key_hash(env, &account)?;
    let register_signature = sign_digest(wallet_private_key, register_hash)?;
    let register_personal_signature = sign_personal_digest(wallet_private_key, register_hash)?;
    let sign_key_signature = sign_digest(signing_key_private_key, sign_key_hash)?;

    Ok(RegisterSignature {
        payload: RegisterRequest {
            account,
            signing_key,
            expiry,
            account_signature: register_personal_signature.clone(),
            signing_key_signature: sign_key_signature.clone(),
            referral_code: None,
            no_api_key: None,
        },
        register_hash: hex0x(register_hash),
        sign_key_hash: hex0x(sign_key_hash),
        register_signature,
        register_personal_signature,
        sign_key_signature,
    })
}

pub fn sign_sign_key(
    env: Env,
    signing_key_private_key: &str,
    account: &str,
) -> Result<SignatureResult<String>> {
    let hash = sign_key_hash(env, account)?;
    Ok(SignatureResult {
        payload: normalize_address(account)?,
        hash: hex0x(hash),
        signature: sign_digest(signing_key_private_key, hash)?,
    })
}

pub fn sign_approve_builder(
    env: Env,
    wallet_private_key: &str,
    builder_id: &str,
    max_fee_rate: &str,
    nonce: Option<u64>,
) -> Result<SignatureResult<ApproveBuilderRequest>> {
    let nonce = nonce.unwrap_or_else(current_unix_timestamp_ms).to_string();
    let account = derive_address(wallet_private_key)?;
    let max_fee_rate_raw = rate_to_raw(max_fee_rate)?;
    sign_approve_builder_raw(
        env,
        wallet_private_key,
        &account,
        builder_id,
        &max_fee_rate_raw,
        &nonce,
        max_fee_rate,
    )
}

pub fn sign_approve_builder_raw(
    env: Env,
    wallet_private_key: &str,
    account: &str,
    builder_id: &str,
    max_fee_rate_raw: &str,
    nonce: &str,
    api_max_fee_rate: &str,
) -> Result<SignatureResult<ApproveBuilderRequest>> {
    let account = normalize_address(account)?;
    let max_fee_rate_raw = parse_u256(max_fee_rate_raw, "max_fee_rate")?.to_string();
    let nonce = parse_u256(nonce, "nonce")?.to_string();
    let hash = hash_typed_data(
        env,
        "ApproveBuilder",
        approve_builder_fields(),
        json!({
            "account": account,
            "builderId": builder_id,
            "maxFeeRate": max_fee_rate_raw,
            "nonce": nonce,
        }),
    )?;
    let signature = sign_digest(wallet_private_key, hash)?;
    Ok(SignatureResult {
        payload: ApproveBuilderRequest {
            builder_id: builder_id.to_string(),
            max_fee_rate: api_max_fee_rate.to_string(),
            nonce,
            signature: signature.clone(),
        },
        hash: hex0x(hash),
        signature,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn sign_withdraw(
    env: Env,
    wallet_private_key: &str,
    account: &str,
    collateral: &str,
    to: &str,
    amount: &str,
    data: Option<&str>,
    recipient: Option<&str>,
) -> Result<SignatureResult<SignedWithdraw>> {
    sign_withdraw_raw(
        env,
        wallet_private_key,
        account,
        collateral,
        to,
        &to_raw6(amount)?,
        &generate_nonce().to_string(),
        data,
        recipient,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn sign_withdraw_raw(
    env: Env,
    wallet_private_key: &str,
    account: &str,
    collateral: &str,
    to: &str,
    amount_raw: &str,
    salt: &str,
    data: Option<&str>,
    recipient: Option<&str>,
) -> Result<SignatureResult<SignedWithdraw>> {
    let account = normalize_address(account)?;
    let collateral = normalize_address(collateral)?;
    let to = normalize_address(to)?;
    let amount = parse_u256(amount_raw, "amount")?.to_string();
    let salt = parse_u256(salt, "salt")?.to_string();
    let data_bytes = decode_hex_bytes(data.unwrap_or(""))?;
    let data_hash = hex0x(keccak256(&data_bytes));
    let hash = hash_typed_data(
        env,
        "Withdraw",
        withdraw_fields(),
        json!({
            "collateral": collateral,
            "to": to,
            "amount": amount,
            "salt": salt,
            "data": data_hash,
        }),
    )?;
    let signature = sign_digest(wallet_private_key, hash)?;
    Ok(SignatureResult {
        payload: SignedWithdraw {
            account,
            collateral,
            to,
            amount,
            salt,
            signature: signature.clone(),
            recipient: recipient.map(normalize_address).transpose()?,
            label: None,
            reference_id: None,
            data: data.map(ToOwned::to_owned),
        },
        hash: hex0x(hash),
        signature,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn sign_transfer(
    env: Env,
    wallet_private_key: &str,
    account: &str,
    collateral: &str,
    to: &str,
    amount: &str,
    label: Option<String>,
    reference_id: Option<String>,
) -> Result<SignatureResult<SignedTransfer>> {
    sign_transfer_raw(
        env,
        wallet_private_key,
        account,
        collateral,
        to,
        &to_raw6(amount)?,
        &generate_nonce().to_string(),
        label,
        reference_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn sign_transfer_raw(
    env: Env,
    wallet_private_key: &str,
    account: &str,
    collateral: &str,
    to: &str,
    amount_raw: &str,
    salt: &str,
    label: Option<String>,
    reference_id: Option<String>,
) -> Result<SignatureResult<SignedTransfer>> {
    let account = normalize_address(account)?;
    let collateral = normalize_address(collateral)?;
    let to = normalize_address(to)?;
    let amount = parse_u256(amount_raw, "amount")?.to_string();
    let salt = parse_u256(salt, "salt")?.to_string();
    let hash = hash_typed_data(
        env,
        "Transfer",
        transfer_fields(),
        json!({
            "collateral": collateral,
            "to": to,
            "amount": amount,
            "salt": salt,
        }),
    )?;
    let signature = sign_digest(wallet_private_key, hash)?;
    Ok(SignatureResult {
        payload: SignedTransfer {
            account,
            collateral,
            to,
            amount,
            salt,
            signature: signature.clone(),
            recipient: None,
            label,
            reference_id,
            data: None,
        },
        hash: hex0x(hash),
        signature,
    })
}

fn sign_key_hash(env: Env, account: &str) -> Result<[u8; 32]> {
    hash_typed_data(
        env,
        "SignKey",
        sign_key_fields(),
        json!({ "account": normalize_address(account)? }),
    )
}

fn hash_typed_data(
    env: Env,
    primary_type: &str,
    fields: Value,
    message: Value,
) -> Result<[u8; 32]> {
    let typed_data: TypedData = serde_json::from_value(json!({
        "types": {
            "EIP712Domain": domain_fields(),
            primary_type: fields,
        },
        "primaryType": primary_type,
        "domain": {
            "name": env.domain_name(),
            "version": "1",
            "chainId": env.chain_id().to_string(),
        },
        "message": message,
    }))?;
    typed_data
        .encode_eip712()
        .map_err(|err| AevoError::Signing(format!("failed to encode EIP-712 data: {err}")))
}

fn sign_digest(private_key: &str, digest: [u8; 32]) -> Result<String> {
    let signature = wallet(private_key)?
        .sign_hash(H256::from(digest))
        .map_err(|err| AevoError::Signing(format!("failed to sign digest: {err}")))?;
    Ok(format!("0x{}", hex::encode(signature.to_vec())))
}

fn sign_personal_digest(private_key: &str, digest: [u8; 32]) -> Result<String> {
    let personal_hash = hash_message(digest);
    sign_digest(private_key, personal_hash.0)
}

fn wallet(private_key: &str) -> Result<LocalWallet> {
    LocalWallet::from_str(&normalize_private_key(private_key)?)
        .map_err(|err| AevoError::Signing(format!("invalid private key: {err}")))
}

fn normalize_private_key(private_key: &str) -> Result<String> {
    let normalized = private_key.trim().trim_start_matches("0x");
    if normalized.len() != 64 || !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(AevoError::InvalidInput(
            "private key must be a 32-byte hex string".into(),
        ));
    }
    Ok(format!("0x{}", normalized.to_ascii_lowercase()))
}

fn parse_address(address: &str) -> Result<Address> {
    Address::from_str(address.trim())
        .map_err(|err| AevoError::InvalidInput(format!("invalid address: {err}")))
}

fn normalize_address(address: &str) -> Result<String> {
    Ok(to_checksum(&parse_address(address)?, None))
}

fn parse_u256(value: &str, field: &str) -> Result<U256> {
    U256::from_dec_str(value.trim())
        .map_err(|_| AevoError::InvalidInput(format!("{field} must be an integer string")))
}

fn decimal_to_scaled(value: &str, field: &str, scale: u64) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AevoError::InvalidInput(format!("{field} is required")));
    }
    if trimmed.starts_with('-') {
        return Err(AevoError::InvalidInput(format!(
            "{field} must be non-negative"
        )));
    }
    if trimmed.contains('e') || trimmed.contains('E') || trimmed.starts_with('+') {
        return Err(AevoError::InvalidInput(format!(
            "{field} must be a plain decimal"
        )));
    }

    let mut parts = trimmed.split('.');
    let whole = parts.next().unwrap_or("0");
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return Err(AevoError::InvalidInput(format!(
            "{field} must be a valid decimal"
        )));
    }
    if whole.is_empty() && fraction.is_empty() {
        return Err(AevoError::InvalidInput(format!(
            "{field} must be a valid decimal"
        )));
    }
    if !whole.chars().all(|ch| ch.is_ascii_digit())
        || !fraction.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(AevoError::InvalidInput(format!(
            "{field} must be a valid decimal"
        )));
    }

    let precision = scale.to_string().len() - 1;
    if fraction.len() > precision {
        return Err(AevoError::InvalidInput(format!(
            "{field} exceeds {precision} decimal places"
        )));
    }

    let whole = if whole.is_empty() { "0" } else { whole };
    // Checked arithmetic: U256 operators panic on overflow; a too-large input is an error.
    let overflow = || AevoError::InvalidInput(format!("{field} is too large"));
    let whole_scaled = parse_u256(whole, field)?
        .checked_mul(U256::from(scale))
        .ok_or_else(overflow)?;
    let mut padded = fraction.to_string();
    while padded.len() < precision {
        padded.push('0');
    }
    let fraction_scaled = if padded.is_empty() {
        U256::zero()
    } else {
        parse_u256(&padded, field)?
    };
    Ok(whole_scaled
        .checked_add(fraction_scaled)
        .ok_or_else(overflow)?
        .to_string())
}

fn raw6_to_decimal(raw: U256) -> Result<String> {
    let scale = U256::from(SCALE_6);
    let whole = raw / scale;
    let fraction = raw % scale;
    if fraction.is_zero() {
        return Ok(whole.to_string());
    }
    let mut fraction_text = format!("{:06}", fraction.as_u64());
    while fraction_text.ends_with('0') {
        fraction_text.pop();
    }
    Ok(format!("{whole}.{fraction_text}"))
}

fn decode_hex_bytes(value: &str) -> Result<Vec<u8>> {
    let value = value.trim().trim_start_matches("0x");
    if value.is_empty() {
        return Ok(Vec::new());
    }
    hex::decode(value).map_err(|err| AevoError::InvalidInput(format!("invalid hex data: {err}")))
}

fn normalize_tif(value: Option<&str>) -> Option<String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.eq_ignore_ascii_case("GTC") => None,
        Some(value) => Some(value.to_ascii_uppercase()),
        None => None,
    }
}

fn hex0x(bytes: [u8; 32]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn domain_fields() -> Value {
    json!([
        {"name": "name", "type": "string"},
        {"name": "version", "type": "string"},
        {"name": "chainId", "type": "uint256"}
    ])
}

fn order_fields() -> Value {
    json!([
        {"name": "maker", "type": "address"},
        {"name": "isBuy", "type": "bool"},
        {"name": "limitPrice", "type": "uint256"},
        {"name": "amount", "type": "uint256"},
        {"name": "salt", "type": "uint256"},
        {"name": "instrument", "type": "uint256"},
        {"name": "timestamp", "type": "uint256"}
    ])
}

fn builder_order_fields() -> Value {
    json!([
        {"name": "maker", "type": "address"},
        {"name": "isBuy", "type": "bool"},
        {"name": "limitPrice", "type": "uint256"},
        {"name": "amount", "type": "uint256"},
        {"name": "salt", "type": "uint256"},
        {"name": "instrument", "type": "uint256"},
        {"name": "timestamp", "type": "uint256"},
        {"name": "builderId", "type": "string"},
        {"name": "builderFeeRate", "type": "uint256"}
    ])
}

fn register_fields() -> Value {
    json!([
        {"name": "key", "type": "address"},
        {"name": "expiry", "type": "uint256"}
    ])
}

fn register_hashed_fields() -> Value {
    json!([
        {"name": "key", "type": "bytes32"},
        {"name": "expiry", "type": "uint256"}
    ])
}

fn sign_key_fields() -> Value {
    json!([
        {"name": "account", "type": "address"}
    ])
}

fn approve_builder_fields() -> Value {
    json!([
        {"name": "account", "type": "address"},
        {"name": "builderId", "type": "string"},
        {"name": "maxFeeRate", "type": "uint256"},
        {"name": "nonce", "type": "uint256"}
    ])
}

fn withdraw_fields() -> Value {
    json!([
        {"name": "collateral", "type": "address"},
        {"name": "to", "type": "address"},
        {"name": "amount", "type": "uint256"},
        {"name": "salt", "type": "uint256"},
        {"name": "data", "type": "bytes32"}
    ])
}

fn transfer_fields() -> Value {
    json!([
        {"name": "collateral", "type": "address"},
        {"name": "to", "type": "address"},
        {"name": "amount", "type": "uint256"},
        {"name": "salt", "type": "uint256"}
    ])
}
