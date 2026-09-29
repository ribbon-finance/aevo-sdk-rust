use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ApiResponse = Value;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MarketsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CancelAllOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BatchCancelOrdersRequest {
    pub order_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct HistoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiCredentials {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuilderFields {
    pub builder_id: String,
    pub builder_fee_rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedOrder {
    pub maker: String,
    pub is_buy: bool,
    pub limit_price: String,
    pub amount: String,
    pub instrument: String,
    pub salt: String,
    pub timestamp: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_fee_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterRequest {
    pub account: String,
    pub signing_key: String,
    pub expiry: String,
    pub account_signature: String,
    pub signing_key_signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_api_key: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedWithdraw {
    pub account: String,
    pub collateral: String,
    pub to: String,
    pub amount: String,
    pub salt: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "referenceId", skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

pub type SignedTransfer = SignedWithdraw;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApproveBuilderRequest {
    pub builder_id: String,
    pub max_fee_rate: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterBuilderRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevokeBuilderRequest {
    pub builder_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BuilderStatsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BuilderMarketsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BuilderUsersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BuilderFillsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BuilderFillsCsvRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<String>,
}
