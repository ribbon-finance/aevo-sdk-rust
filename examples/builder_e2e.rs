use aevo_sdk::models::{BuilderFillsCsvRequest, BuilderFillsRequest, BuilderStatsRequest};
use aevo_sdk::signing::OrderToSign;
use aevo_sdk::{AevoClient, AuthMode, Env, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let send = std::env::var("SEND").ok().as_deref() == Some("1");
    let api_key = std::env::var("AEVO_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("AEVO_API_SECRET").unwrap_or_default();
    let wallet_key = std::env::var("AEVO_WALLET_KEY").unwrap_or_else(|_| {
        "0x5b74887f136946f14441ae2d6c23119fcf93f98221ada8e906b953b941020295".into()
    });
    let signing_key = std::env::var("AEVO_SIGNING_KEY").unwrap_or_else(|_| {
        "0xe3aed01a3fb8fa817607da5c385aa50904bf7ae47decef855e1fb66377e40ec2".into()
    });

    let client = AevoClient::builder()
        .env(Env::Testnet)
        .api_key(api_key)
        .api_secret(api_secret)
        .wallet_key(wallet_key)
        .signing_key(signing_key)
        .auth_mode(AuthMode::Hmac)
        .build()?;

    let config = client.builder_config().await;
    println!("builder_config: {config:?}");

    let builder_id =
        std::env::var("AEVO_BUILDER_ID").unwrap_or_else(|_| "builder_0123456789abcdef".to_string());
    let approval = client.sign_approve_builder(&builder_id, "0.0005")?;
    println!(
        "approve_builder payload:\n{}",
        serde_json::to_string_pretty(&approval.payload)?
    );

    let order = OrderToSign::builder()
        .maker(
            std::env::var("AEVO_ACCOUNT")
                .unwrap_or_else(|_| "0x7D19833b5aF3b4e4D75DBA556ded46930469FA27".to_string()),
        )
        .is_buy(true)
        .limit_price("2700")?
        .amount("1")?
        .instrument(std::env::var("AEVO_INSTRUMENT").unwrap_or_else(|_| "1".into()))
        .builder_fields(builder_id.clone(), "0.0003")
        .post_only(true)
        .build()?;
    let signed_order = client.sign_order(order)?;
    println!(
        "builder order payload:\n{}",
        serde_json::to_string_pretty(&signed_order.payload)?
    );

    if send {
        println!("SEND=1: submitting approval and order");
        println!("{:?}", client.approve_builder(&approval.payload).await?);
        println!("{:?}", client.create_order(&signed_order.payload).await?);
        println!(
            "{:?}",
            client
                .builder_stats(BuilderStatsRequest {
                    period: Some("24h".into()),
                    ..BuilderStatsRequest::default()
                })
                .await?
        );
        println!(
            "{:?}",
            client
                .builder_fills(BuilderFillsRequest {
                    limit: Some(10),
                    ..BuilderFillsRequest::default()
                })
                .await?
        );
        println!(
            "{}",
            client
                .download_builder_fills_csv(BuilderFillsCsvRequest::default())
                .await?
        );
    } else {
        println!("dry run only; set SEND=1 to submit requests");
    }

    Ok(())
}
