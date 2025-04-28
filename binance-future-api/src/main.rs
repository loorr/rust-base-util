use base_util::ws::{SendError, WsClient};
use binance_future_api::model::{ApiResponse, OrderSide, PositionSide, TimeInForce};
use binance_future_api::req_param::{
    BaseReq, CancelOrder, KeyPair, OrderPlace, OrderStatus, PositionRisk, UserDataStream,
    WsReqMethod,
};
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let key_pair = KeyPair {
        api_key: "3ec340c3bf6399c711d2b0e34f8cefff48c2aed984c7c5157c813810a027ee45".to_string(),
        secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477".to_string(),
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);

    // wss://testnet.binancefuture.com/ws-fapi/v1
    let base_url = "wss://testnet.binancefuture.com/ws-fapi/v1";
    let client_arc = WsClient::new(base_url, None, tx.clone(), None);
    client_arc.connect_spawn();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Received message: {:?} \n", msg);
            let response: ApiResponse = serde_json::from_str(msg.as_str())
                .map_err(|e| {
                    println!("Error parsing message: {:?}", e);
                })
                .unwrap();
            println!("Received message: {:?}", response);
        }
    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let place_market = OrderPlace::place_market(
        &key_pair,
        "BTCUSDT",
        &OrderSide::Buy,
        &PositionSide::Long,
        &Decimal::from_str("0.01").unwrap(),
        Some("test_order_id"),
    );
    let place_market_req = BaseReq::new("place_market", WsReqMethod::OrderPlace(place_market));

    // place limit
    let place_limit = OrderPlace::place_limit(
        &key_pair,
        "BTCUSDT",
        &OrderSide::Buy,
        &PositionSide::Long,
        &Decimal::from_str("0.01").unwrap(),
        &Decimal::from_str("80000").unwrap(),
        Some("test_order_id"),
        TimeInForce::GTC,
        Some(9999_9999),
    );
    let place_limit_req = BaseReq::new("place_limit", WsReqMethod::OrderPlace(place_limit));

    let time_req = BaseReq::new("time", WsReqMethod::Time);
    let ping_req = BaseReq::new("ping", WsReqMethod::Ping);

    let position_risk = PositionRisk::new(&key_pair, None, None);
    let position_risk_req = BaseReq::new("position_risk", WsReqMethod::PositionRisk(position_risk));

    // 4372849252
    let cancel_order = CancelOrder::new(
        &key_pair,
        "BTCUSDT",
        None,
        Some("test_order_id"),
        Some(9999_9999),
    );
    let cancel_order_req = BaseReq::new("cancel_order", WsReqMethod::CancelOrder(cancel_order));

    let order_status = OrderStatus::new(
        &key_pair,
        "BTCUSDT",
        None,
        Some("test_order_id"),
        Some(9999_9999),
    );
    let order_status_req = BaseReq::new("order_status", WsReqMethod::OrderStatus(order_status));

    let user_data_stream_start_req = BaseReq::new(
        "user_data_stream_start",
        WsReqMethod::UserDataStreamStart(UserDataStream::new(&key_pair)),
    );
    let user_data_stream_ping_req = BaseReq::new(
        "user_data_stream_ping",
        WsReqMethod::UserDataStreamPing(UserDataStream::new(&key_pair)),
    );
    let user_data_stream_stop_req = BaseReq::new(
        "user_data_stream_stop",
        WsReqMethod::UserDataStreamStop(UserDataStream::new(&key_pair)),
    );
    let reqs = vec![
        place_market_req,
        place_limit_req,
        time_req,
        ping_req,
        position_risk_req,
        cancel_order_req,
        order_status_req,
        user_data_stream_start_req,
        user_data_stream_ping_req,
        user_data_stream_stop_req,
    ];

    for (i, req) in reqs.iter().enumerate() {
        println!("Sending request {}: {:?}", i, req);

        match client_arc.send_message(req.serialize().as_str()).await {
            Ok(_) => {}
            Err(err) => {
                println!("Error sending message: {:?}", err);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Keep the main thread alive to receive messages
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
