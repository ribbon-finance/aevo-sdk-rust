use aevo_sdk::models::SignedOrder;
use aevo_sdk::ws::{
    auth_frame, create_order_frame, parse_message, subscribe_frame, unsubscribe_frame, WsChannel,
};
use serde_json::json;

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

#[test]
fn auth_and_subscription_frames_match_teller_shape() {
    assert_eq!(
        auth_frame(1, "key", "secret"),
        json!({
            "id": 1,
            "op": "auth",
            "data": {"key": "key", "secret": "secret"}
        })
    );
    assert_eq!(
        subscribe_frame(
            Some(2),
            vec![
                WsChannel::Orderbook("ETH-PERP".into()).as_string(),
                WsChannel::Fills.as_string()
            ]
        ),
        json!({
            "id": 2,
            "op": "subscribe",
            "data": ["orderbook:ETH-PERP", "fills"]
        })
    );
    assert_eq!(
        unsubscribe_frame(None, vec![WsChannel::Ticker("ETH-PERP".into()).as_string()]),
        json!({
            "op": "unsubscribe",
            "data": ["ticker:ETH-PERP"]
        })
    );
}

#[test]
fn create_order_frame_includes_builder_fields_in_data() {
    let order = sample_order();
    let frame = create_order_frame(7, &order, Some(json!({"key": "k"}))).unwrap();
    assert_eq!(frame["id"], 7);
    assert_eq!(frame["op"], "create_order");
    assert_eq!(frame["auth"], json!({"key": "k"}));
    assert_eq!(frame["data"]["builder_id"], "builder_0123456789abcdef");
    assert_eq!(frame["data"]["builder_fee_rate"], "0.0003");
}

#[test]
fn websocket_messages_parse_as_typed_envelopes() {
    let parsed = parse_message(
        r#"{"id":1,"op":"subscribe","channel":"fills","data":{"success":true},"extra":42}"#,
    )
    .unwrap();
    assert_eq!(parsed.id, Some(1));
    assert_eq!(parsed.op.as_deref(), Some("subscribe"));
    assert_eq!(parsed.channel.as_deref(), Some("fills"));
    assert_eq!(parsed.data.unwrap()["success"], true);
    assert_eq!(parsed.extra["extra"], 42);
}
