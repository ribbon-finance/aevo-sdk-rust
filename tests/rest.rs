use aevo_sdk::models::*;
use aevo_sdk::{AevoClient, AuthMode, BuilderErrorCode};
use serde_json::json;
use wiremock::matchers::{
    body_json, header, header_exists, method, path, query_param, query_param_is_missing,
};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> AevoClient {
    AevoClient::builder()
        .base_url(server.uri())
        .api_key("key")
        .api_secret("secret")
        .auth_mode(AuthMode::SecretHeader)
        .build()
        .unwrap()
}

fn ok_json() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"ok": true}))
}

fn private(method_name: &'static str, path_name: &'static str) -> MockBuilder {
    Mock::given(method(method_name))
        .and(path(path_name))
        .and(header("AEVO-KEY", "key"))
        .and(header("AEVO-SECRET", "secret"))
}

fn sample_order() -> SignedOrder {
    SignedOrder {
        maker: "0x7D19833b5aF3b4e4D75DBA556ded46930469FA27".into(),
        is_buy: true,
        limit_price: "1000000".into(),
        amount: "2000000".into(),
        instrument: "1".into(),
        salt: "10".into(),
        timestamp: "1700000000".into(),
        signature: "0xsig".into(),
        post_only: Some(true),
        reduce_only: None,
        close_position: None,
        partial_position: None,
        stop: None,
        trigger: None,
        time_in_force: None,
        mmp: None,
        builder_id: Some("builder_0123456789abcdef".into()),
        builder_fee_rate: Some("0.0003".into()),
    }
}

fn sample_withdraw() -> SignedWithdraw {
    SignedWithdraw {
        account: "0x7D19833b5aF3b4e4D75DBA556ded46930469FA27".into(),
        collateral: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
        to: "0xEB2E316a1112f40e7b3c9019BC19B6009eC661AF".into(),
        amount: "1000000".into(),
        salt: "1".into(),
        signature: "0xsig".into(),
        recipient: None,
        label: None,
        reference_id: None,
        data: None,
    }
}

#[tokio::test]
async fn public_methods_send_expected_paths_and_queries() {
    let server = MockServer::start().await;
    let client = client(&server);

    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("asset", "ETH"))
        .and(query_param("instrument_type", "PERPETUAL"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/instrument/ETH-PERP"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orderbook"))
        .and(query_param("instrument_name", "ETH-PERP"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .and(query_param("asset", "ETH"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/builder-config"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/builders/builder_0123456789abcdef"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;

    client
        .markets(MarketsRequest {
            asset: Some("ETH".into()),
            instrument_type: Some("PERPETUAL".into()),
        })
        .await
        .unwrap();
    client.instrument("ETH-PERP").await.unwrap();
    client.orderbook("ETH-PERP").await.unwrap();
    client.index("ETH").await.unwrap();
    client.time().await.unwrap();
    client.builder_config().await.unwrap();
    client
        .builder_profile("builder_0123456789abcdef")
        .await
        .unwrap();
}

#[tokio::test]
async fn private_read_and_builder_report_methods_send_expected_requests() {
    let server = MockServer::start().await;
    let client = client(&server);

    for path_name in ["/account", "/portfolio", "/positions", "/orders"] {
        private("GET", path_name)
            .respond_with(ok_json())
            .expect(1)
            .mount(&server)
            .await;
    }
    private("GET", "/trade-history")
        .and(query_param("instrument_name", "ETH-PERP"))
        .and(query_param("limit", "25"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/account/builder-approvals")
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/approval")
        .and(query_param(
            "account",
            "0x7D19833b5aF3b4e4D75DBA556ded46930469FA27",
        ))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/stats")
        .and(query_param("period", "7d"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/markets")
        .and(query_param("limit", "10"))
        .and(query_param("offset", "5"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/users")
        .and(query_param("cursor", "abc"))
        .and(query_param_is_missing("offset"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/fills")
        .and(query_param("instrument", "ETH-PERP"))
        .and(query_param("limit", "50"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("GET", "/builder/fills")
        .and(query_param("format", "csv"))
        .respond_with(ResponseTemplate::new(200).set_body_string("a,b\n1,2\n"))
        .expect(1)
        .mount(&server)
        .await;

    client.account().await.unwrap();
    client.portfolio().await.unwrap();
    client.positions().await.unwrap();
    client.orders().await.unwrap();
    client
        .trade_history(HistoryRequest {
            instrument_name: Some("ETH-PERP".into()),
            limit: Some(25),
            ..HistoryRequest::default()
        })
        .await
        .unwrap();
    client.builder_approvals().await.unwrap();
    client
        .builder_approval("0x7D19833b5aF3b4e4D75DBA556ded46930469FA27")
        .await
        .unwrap();
    client
        .builder_stats(BuilderStatsRequest {
            period: Some("7d".into()),
            ..BuilderStatsRequest::default()
        })
        .await
        .unwrap();
    client
        .builder_markets(BuilderMarketsRequest {
            limit: Some(10),
            offset: Some(5),
            ..BuilderMarketsRequest::default()
        })
        .await
        .unwrap();
    client
        .builder_users(BuilderUsersRequest {
            cursor: Some("abc".into()),
            ..BuilderUsersRequest::default()
        })
        .await
        .unwrap();
    client
        .builder_fills(BuilderFillsRequest {
            instrument: Some("ETH-PERP".into()),
            limit: Some(50),
            ..BuilderFillsRequest::default()
        })
        .await
        .unwrap();
    assert_eq!(
        client
            .download_builder_fills_csv(BuilderFillsCsvRequest::default())
            .await
            .unwrap(),
        "a,b\n1,2\n"
    );
}

#[tokio::test]
async fn write_methods_send_expected_methods_paths_bodies_and_auth_headers() {
    let server = MockServer::start().await;
    let client = client(&server);
    let order = sample_order();
    let withdraw = sample_withdraw();
    let transfer = SignedWithdraw {
        label: Some("label".into()),
        reference_id: Some("ref".into()),
        ..withdraw.clone()
    };

    private("POST", "/orders")
        .and(body_json(&order))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/orders/0xorder")
        .and(body_json(&order))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("DELETE", "/orders/0xorder")
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("DELETE", "/orders-all")
        .and(body_json(
            json!({"instrument_type": "PERPETUAL", "asset": "ETH"}),
        ))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/batch-orders")
        .and(body_json(json!({"orders": [&order]})))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("DELETE", "/orders")
        .and(body_json(
            json!({"order_ids": ["1", "2"], "instrument_type": "OPTION"}),
        ))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/register"))
        .and(body_json(json!({
            "account": "0xabc",
            "signing_key": "0xdef",
            "expiry": "1",
            "account_signature": "0xaaa",
            "signing_key_signature": "0xbbb"
        })))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/withdraw")
        .and(body_json(&withdraw))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/transfer")
        .and(body_json(&transfer))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/builder/register")
        .and(body_json(json!({"name": "Builder"})))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/builder/profile")
        .and(body_json(json!({"name": "Renamed"})))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/builder/approve")
        .and(body_json(json!({
            "builder_id": "builder_0123456789abcdef",
            "max_fee_rate": "0.0005",
            "nonce": "1700000000000",
            "signature": "0xsig"
        })))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;
    private("POST", "/builder/revoke")
        .and(body_json(json!({"builder_id": "builder_0123456789abcdef"})))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;

    client.create_order(&order).await.unwrap();
    client.edit_order("0xorder", &order).await.unwrap();
    client.cancel_order("0xorder").await.unwrap();
    client
        .cancel_all_orders(CancelAllOrdersRequest {
            instrument_type: Some("PERPETUAL".into()),
            asset: Some("ETH".into()),
        })
        .await
        .unwrap();
    client.batch_create_orders(&[order]).await.unwrap();
    client
        .batch_cancel_orders(BatchCancelOrdersRequest {
            order_ids: vec!["1".into(), "2".into()],
            instrument_type: Some("OPTION".into()),
        })
        .await
        .unwrap();
    client
        .register(&RegisterRequest {
            account: "0xabc".into(),
            signing_key: "0xdef".into(),
            expiry: "1".into(),
            account_signature: "0xaaa".into(),
            signing_key_signature: "0xbbb".into(),
            referral_code: None,
            no_api_key: None,
        })
        .await
        .unwrap();
    client.withdraw(&withdraw).await.unwrap();
    client.transfer(&transfer).await.unwrap();
    client.register_builder("Builder").await.unwrap();
    client.update_builder_profile("Renamed").await.unwrap();
    client
        .approve_builder(&ApproveBuilderRequest {
            builder_id: "builder_0123456789abcdef".into(),
            max_fee_rate: "0.0005".into(),
            nonce: "1700000000000".into(),
            signature: "0xsig".into(),
        })
        .await
        .unwrap();
    client
        .revoke_builder("builder_0123456789abcdef")
        .await
        .unwrap();
}

#[tokio::test]
async fn hmac_auth_mode_sends_signed_headers_without_secret_header() {
    let server = MockServer::start().await;
    let client = AevoClient::builder()
        .base_url(server.uri())
        .api_key("key")
        .api_secret("secret")
        .auth_mode(AuthMode::Hmac)
        .build()
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/account"))
        .and(header("AEVO-KEY", "key"))
        .and(header_exists("AEVO-TIMESTAMP"))
        .and(header_exists("AEVO-SIGNATURE"))
        .respond_with(ok_json())
        .expect(1)
        .mount(&server)
        .await;

    client.account().await.unwrap();
}

#[tokio::test]
async fn api_error_mapping_keeps_builder_codes() {
    let server = MockServer::start().await;
    let client = client(&server);

    private("GET", "/account")
        .respond_with(
            ResponseTemplate::new(400).set_body_json(json!({"error": "BUILDER_NOT_APPROVED"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = client.account().await.unwrap_err();
    match err {
        aevo_sdk::AevoError::Api { status, code, .. } => {
            assert_eq!(status, 400);
            assert_eq!(code, Some(BuilderErrorCode::BuilderNotApproved));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn builder_cursor_and_offset_are_rejected_client_side() {
    let server = MockServer::start().await;
    let client = client(&server);

    assert!(client
        .builder_users(BuilderUsersRequest {
            cursor: Some("abc".into()),
            offset: Some(1),
            ..BuilderUsersRequest::default()
        })
        .await
        .is_err());
    assert!(client
        .builder_fills(BuilderFillsRequest {
            cursor: Some("abc".into()),
            offset: Some(1),
            ..BuilderFillsRequest::default()
        })
        .await
        .is_err());
}
