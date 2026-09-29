use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use url::Url;

use crate::config::{AuthMode, Env, Secret};
use crate::error::{AevoError, Result};
use crate::models::*;
use crate::signing;

#[derive(Debug, Clone)]
pub struct AevoClientBuilder {
    env: Env,
    base_url: Option<String>,
    ws_url: Option<String>,
    api_key: Option<String>,
    api_secret: Option<Secret>,
    signing_key: Option<Secret>,
    wallet_key: Option<Secret>,
    auth_mode: AuthMode,
}

impl Default for AevoClientBuilder {
    fn default() -> Self {
        Self {
            env: Env::Testnet,
            base_url: None,
            ws_url: None,
            api_key: None,
            api_secret: None,
            signing_key: None,
            wallet_key: None,
            auth_mode: AuthMode::Hmac,
        }
    }
}

impl AevoClientBuilder {
    pub fn env(mut self, env: Env) -> Self {
        self.env = env;
        self
    }

    pub fn api_key(mut self, value: impl Into<String>) -> Self {
        self.api_key = Some(value.into());
        self
    }

    pub fn api_secret(mut self, value: impl Into<String>) -> Self {
        self.api_secret = Some(Secret::new(value));
        self
    }

    pub fn signing_key(mut self, value: impl Into<String>) -> Self {
        self.signing_key = Some(Secret::new(value));
        self
    }

    pub fn wallet_key(mut self, value: impl Into<String>) -> Self {
        self.wallet_key = Some(Secret::new(value));
        self
    }

    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = Some(value.into());
        self
    }

    pub fn ws_url(mut self, value: impl Into<String>) -> Self {
        self.ws_url = Some(value.into());
        self
    }

    pub fn auth_mode(mut self, value: AuthMode) -> Self {
        self.auth_mode = value;
        self
    }

    pub fn build(self) -> Result<AevoClient> {
        let base_url = normalize_base_url(self.base_url.as_deref().unwrap_or(self.env.rest_url()))?;
        let ws_url = normalize_ws_url(self.ws_url.as_deref().unwrap_or(self.env.ws_url()))?;
        Ok(AevoClient {
            http: build_http_client()?,
            env: self.env,
            base_url,
            ws_url,
            api_key: self.api_key,
            api_secret: self.api_secret,
            signing_key: self.signing_key,
            wallet_key: self.wallet_key,
            auth_mode: self.auth_mode,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AevoClient {
    http: reqwest::Client,
    env: Env,
    base_url: String,
    ws_url: String,
    api_key: Option<String>,
    api_secret: Option<Secret>,
    signing_key: Option<Secret>,
    wallet_key: Option<Secret>,
    auth_mode: AuthMode,
}

impl AevoClient {
    pub fn builder() -> AevoClientBuilder {
        AevoClientBuilder::default()
    }

    pub fn env(&self) -> Env {
        self.env
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn signing_key(&self) -> Option<&str> {
        self.signing_key.as_ref().map(Secret::expose)
    }

    pub fn wallet_key(&self) -> Option<&str> {
        self.wallet_key.as_ref().map(Secret::expose)
    }

    pub async fn markets(&self, request: MarketsRequest) -> Result<ApiResponse> {
        self.public_get("/markets", &request).await
    }

    pub async fn instrument(&self, instrument_name: &str) -> Result<ApiResponse> {
        self.public_get(&format!("/instrument/{instrument_name}"), &())
            .await
    }

    pub async fn orderbook(&self, instrument_name: &str) -> Result<ApiResponse> {
        self.public_get(
            "/orderbook",
            &[("instrument_name", instrument_name.to_string())],
        )
        .await
    }

    pub async fn index(&self, asset: &str) -> Result<ApiResponse> {
        self.public_get("/index", &[("asset", asset.to_string())])
            .await
    }

    pub async fn time(&self) -> Result<ApiResponse> {
        self.public_get("/time", &()).await
    }

    pub async fn builder_config(&self) -> Result<ApiResponse> {
        self.public_get("/builder-config", &()).await
    }

    pub async fn builder_profile(&self, builder_id: &str) -> Result<ApiResponse> {
        self.public_get(&format!("/builders/{builder_id}"), &())
            .await
    }

    pub async fn account(&self) -> Result<ApiResponse> {
        self.private_get("/account", &()).await
    }

    pub async fn portfolio(&self) -> Result<ApiResponse> {
        self.private_get("/portfolio", &()).await
    }

    pub async fn positions(&self) -> Result<ApiResponse> {
        self.private_get("/positions", &()).await
    }

    pub async fn orders(&self) -> Result<ApiResponse> {
        self.private_get("/orders", &()).await
    }

    pub async fn create_order(&self, order: &SignedOrder) -> Result<ApiResponse> {
        self.private_json(Method::POST, "/orders", Some(order))
            .await
    }

    pub async fn edit_order(&self, order_id: &str, order: &SignedOrder) -> Result<ApiResponse> {
        self.private_json(Method::POST, &format!("/orders/{order_id}"), Some(order))
            .await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<ApiResponse> {
        self.private_json::<()>(Method::DELETE, &format!("/orders/{order_id}"), None)
            .await
    }

    pub async fn cancel_all_orders(&self, request: CancelAllOrdersRequest) -> Result<ApiResponse> {
        if request.instrument_type.is_none() && request.asset.is_none() {
            self.private_json::<()>(Method::DELETE, "/orders-all", None)
                .await
        } else {
            self.private_json(Method::DELETE, "/orders-all", Some(&request))
                .await
        }
    }

    pub async fn batch_create_orders(&self, orders: &[SignedOrder]) -> Result<ApiResponse> {
        self.private_json(
            Method::POST,
            "/batch-orders",
            Some(&json!({ "orders": orders })),
        )
        .await
    }

    pub async fn batch_cancel_orders(
        &self,
        request: BatchCancelOrdersRequest,
    ) -> Result<ApiResponse> {
        self.private_json(Method::DELETE, "/orders", Some(&request))
            .await
    }

    pub async fn trade_history(&self, request: HistoryRequest) -> Result<ApiResponse> {
        self.private_get("/trade-history", &request).await
    }

    pub async fn register(&self, request: &RegisterRequest) -> Result<ApiResponse> {
        self.public_json(Method::POST, "/register", Some(request))
            .await
    }

    pub async fn withdraw(&self, request: &SignedWithdraw) -> Result<ApiResponse> {
        self.private_json(Method::POST, "/withdraw", Some(request))
            .await
    }

    pub async fn transfer(&self, request: &SignedTransfer) -> Result<ApiResponse> {
        self.private_json(Method::POST, "/transfer", Some(request))
            .await
    }

    pub async fn register_builder(&self, name: impl Into<String>) -> Result<ApiResponse> {
        self.private_json(
            Method::POST,
            "/builder/register",
            Some(&RegisterBuilderRequest { name: name.into() }),
        )
        .await
    }

    pub async fn update_builder_profile(&self, name: impl Into<String>) -> Result<ApiResponse> {
        self.private_json(
            Method::POST,
            "/builder/profile",
            Some(&RegisterBuilderRequest { name: name.into() }),
        )
        .await
    }

    pub async fn approve_builder(&self, request: &ApproveBuilderRequest) -> Result<ApiResponse> {
        self.private_json(Method::POST, "/builder/approve", Some(request))
            .await
    }

    pub async fn revoke_builder(&self, builder_id: impl Into<String>) -> Result<ApiResponse> {
        self.private_json(
            Method::POST,
            "/builder/revoke",
            Some(&RevokeBuilderRequest {
                builder_id: builder_id.into(),
            }),
        )
        .await
    }

    pub async fn builder_approvals(&self) -> Result<ApiResponse> {
        self.private_get("/account/builder-approvals", &()).await
    }

    pub async fn builder_approval(&self, account: &str) -> Result<ApiResponse> {
        self.private_get("/builder/approval", &[("account", account.to_string())])
            .await
    }

    pub async fn builder_stats(&self, request: BuilderStatsRequest) -> Result<ApiResponse> {
        self.private_get("/builder/stats", &request).await
    }

    pub async fn builder_markets(&self, request: BuilderMarketsRequest) -> Result<ApiResponse> {
        self.private_get("/builder/markets", &request).await
    }

    pub async fn builder_users(&self, request: BuilderUsersRequest) -> Result<ApiResponse> {
        ensure_cursor_offset_guard(request.cursor.as_ref(), request.offset)?;
        self.private_get("/builder/users", &request).await
    }

    pub async fn builder_fills(&self, request: BuilderFillsRequest) -> Result<ApiResponse> {
        ensure_cursor_offset_guard(request.cursor.as_ref(), request.offset)?;
        self.private_get("/builder/fills", &request).await
    }

    pub async fn download_builder_fills_csv(
        &self,
        request: BuilderFillsCsvRequest,
    ) -> Result<String> {
        let mut query = serde_json::to_value(request)?;
        if let Value::Object(map) = &mut query {
            map.insert("format".into(), Value::String("csv".into()));
        }
        self.private_text(
            Method::GET,
            "/builder/fills",
            Some(&query),
            Option::<&()>::None,
        )
        .await
    }

    pub fn sign_order(
        &self,
        order: crate::signing::OrderToSign,
    ) -> Result<crate::signing::SignatureResult<SignedOrder>> {
        let key = self.signing_key.as_ref().ok_or_else(|| {
            AevoError::InvalidInput("signing_key is required to sign orders".into())
        })?;
        signing::sign_order(self.env, key.expose(), order)
    }

    pub fn sign_approve_builder(
        &self,
        builder_id: &str,
        max_fee_rate: &str,
    ) -> Result<crate::signing::SignatureResult<ApproveBuilderRequest>> {
        let key = self.wallet_key.as_ref().ok_or_else(|| {
            AevoError::InvalidInput("wallet_key is required to approve builders".into())
        })?;
        signing::sign_approve_builder(self.env, key.expose(), builder_id, max_fee_rate, None)
    }

    async fn public_get<Q: Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<ApiResponse> {
        self.request(Method::GET, path, Some(query), Option::<&()>::None, false)
            .await
    }

    async fn private_get<Q: Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<ApiResponse> {
        self.request(Method::GET, path, Some(query), Option::<&()>::None, true)
            .await
    }

    async fn public_json<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<ApiResponse> {
        self.request(method, path, Option::<&()>::None, body, false)
            .await
    }

    async fn private_json<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<ApiResponse> {
        self.request(method, path, Option::<&()>::None, body, true)
            .await
    }

    async fn private_text<Q: Serialize + ?Sized, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<String> {
        let response = self.raw_request(method, path, query, body, true).await?;
        parse_text_response(response).await
    }

    async fn request<Q: Serialize + ?Sized, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
        auth: bool,
    ) -> Result<ApiResponse> {
        let response = self.raw_request(method, path, query, body, auth).await?;
        parse_json_response(response).await
    }

    async fn raw_request<Q: Serialize + ?Sized, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
        auth: bool,
    ) -> Result<reqwest::Response> {
        let path = normalize_path(path);
        let url = format!("{}{}", self.base_url, path);
        let signed_path = Url::parse(&url)?.path().to_string();
        let body_string = body
            .map(serde_json::to_string)
            .transpose()?
            .unwrap_or_default();
        let mut request = self.http.request(method.clone(), url);
        if let Some(query) = query {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        if auth {
            request = self.apply_auth(request, method.as_str(), &signed_path, &body_string)?;
        }
        Ok(request.send().await?)
    }

    fn apply_auth(
        &self,
        request: reqwest::RequestBuilder,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<reqwest::RequestBuilder> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AevoError::InvalidInput("api_key is required".into()))?;
        let api_secret = self
            .api_secret
            .as_ref()
            .ok_or_else(|| AevoError::InvalidInput("api_secret is required".into()))?;
        match self.auth_mode {
            AuthMode::SecretHeader => Ok(request
                .header("AEVO-KEY", api_key)
                .header("AEVO-SECRET", api_secret.expose())),
            AuthMode::Hmac => {
                let timestamp = signing::current_unix_timestamp_ns().to_string();
                let signature = signing::hmac_signature(
                    api_key,
                    api_secret.expose(),
                    &timestamp,
                    method,
                    path,
                    body,
                )?;
                Ok(request
                    .header("AEVO-KEY", api_key)
                    .header("AEVO-TIMESTAMP", timestamp)
                    .header("AEVO-SIGNATURE", signature))
            }
        }
    }
}

fn ensure_cursor_offset_guard(cursor: Option<&String>, offset: Option<i64>) -> Result<()> {
    if cursor.is_some() && offset.is_some() {
        return Err(AevoError::InvalidInput(
            "pass either cursor or offset, not both".into(),
        ));
    }
    Ok(())
}

fn build_http_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("X-Aevo-Client", HeaderValue::from_static("aevo-sdk-rust"));
    headers.insert(
        "X-Aevo-Client-Version",
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );
    Ok(reqwest::Client::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .build()?)
}

async fn parse_json_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let text = response.text().await?;
    if status.is_success() {
        if text.trim().is_empty() {
            return Ok(serde_json::from_value(json!({}))?);
        }
        return Ok(serde_json::from_str(&text)?);
    }
    Err(parse_api_error(status.as_u16(), &text))
}

async fn parse_text_response(response: reqwest::Response) -> Result<String> {
    let status = response.status();
    let text = response.text().await?;
    if status.is_success() {
        return Ok(text);
    }
    Err(parse_api_error(status.as_u16(), &text))
}

fn parse_api_error(status: u16, text: &str) -> AevoError {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return AevoError::api(status, &value);
    }
    AevoError::Api {
        status,
        code: None,
        message: text.trim().to_string(),
    }
}

fn normalize_base_url(value: &str) -> Result<String> {
    let url = Url::parse(value)?;
    match url.scheme() {
        "https" => Ok(value.trim_end_matches('/').to_string()),
        "http" if is_local_url(&url) => Ok(value.trim_end_matches('/').to_string()),
        _ => Err(AevoError::InvalidInput(format!(
            "base_url must use https or local http: {value}"
        ))),
    }
}

fn normalize_ws_url(value: &str) -> Result<String> {
    let url = Url::parse(value)?;
    match url.scheme() {
        "wss" => Ok(value.trim_end_matches('/').to_string()),
        "ws" if is_local_url(&url) => Ok(value.trim_end_matches('/').to_string()),
        _ => Err(AevoError::InvalidInput(format!(
            "ws_url must use wss or local ws: {value}"
        ))),
    }
}

fn is_local_url(url: &Url) -> bool {
    url.host_str()
        .is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "::1"))
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}
