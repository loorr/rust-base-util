use std::str::FromStr;
use std::sync::Arc;
use log::info;
use rust_decimal::Decimal;

use binance_future_api::model::{ApiResponse, OrderSide, PositionSide, ResultType, TimeInForce};
use binance_future_api::req_param::{KeyPair, OrderPlace, WsReqMethod};
use binance_future_api::ws_fapi::start_ws_fapi;

#[tokio::main]
async fn main() {
    env_logger::init();
    let key_pair = Arc::new(KeyPair {
        api_key: "3ec340c3bf6399c711d2b0e34f8cefff48c2aed984c7c5157c813810a027ee45".to_string(),
        secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477".to_string(),
    });
    let base_url = "wss://testnet.binancefuture.com/ws-fapi/v1";
    let (mut resp_rx, req_tx) = start_ws_fapi(base_url).await;

    info!("WebSocket connection established. Listening for messages...");
    let place_market = OrderPlace::place_market(
        key_pair.clone(),
        "BTCUSDT",
        &OrderSide::Buy,
        &PositionSide::Long,
        &Decimal::from_str("0.01").unwrap(),
        Some("test_order_id"),
    );
    let place_limit = OrderPlace::place_limit(
        key_pair.clone(),
        "BTCUSDT",
        &OrderSide::Buy,
        &PositionSide::Long,
        &Decimal::from_str("0.01").unwrap(),
        &Decimal::from_str("80000").unwrap(),
        Some("test_order_id"),
        TimeInForce::GTC,
        Some(9999_9999),
    );
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    req_tx
        .send(WsReqMethod::OrderPlace(place_market))
        .await
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    req_tx
        .send(WsReqMethod::OrderPlace(place_limit))
        .await
        .unwrap();
    req_tx.send(WsReqMethod::Ping).await.unwrap();

    tokio::spawn(async move {
        while let Some(resp) = resp_rx.recv().await {
            match resp {
                ApiResponse::Error(err) => {
                    println!("Error: {:?}", err);
                }
                ApiResponse::Success(row_data) => match row_data.result {
                    ResultType::Order(data) => {
                        println!("Order data: {:#?}", data);
                    }
                    ResultType::PositionRisk(data) => {
                        println!("Position risk data: {:?}", data);
                    }
                    ResultType::UserDataStream(data) => {
                        println!("User data stream: {:?}", data);
                    }
                    ResultType::ServerTime(data) => {
                        println!("Server time: {:?}", data);
                    }
                    ResultType::EmptyBody(data) => {
                        println!("Empty body: {:?}", data);
                    }
                },
                ApiResponse::RawData(row_data) => {
                    println!("Raw data: {:?}", row_data);
                }
            }
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
}
