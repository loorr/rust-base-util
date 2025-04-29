use std::sync::Arc;

use dashmap::DashMap;
use log::info;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use base_util::time::current_time_nanos;
use base_util::ws::WsClient;

use crate::model::ApiResponse;
use crate::req_param::{BaseReq, WsReqMethod};

// 请求跟踪
#[derive(Debug, Clone)]
struct ReqTrack {
    id: String,
    // 开始发送的时间
    start_send_time: Option<u64>,
    // 发送成功的时间
    end_send_time: Option<u64>,
    // 收到响应的时间
    resp_time: Option<u64>,
}

// 返回 发送订单 和 接受消息结果
// 是否能转换为同步调用, 通过wake机制
// 发送订单后, 监听消息里面的id, 匹配
pub async fn start_ws_fapi(base_url: &str) -> (Receiver<ApiResponse>, Sender<WsReqMethod>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);
    let client_arc = WsClient::new(base_url, None, tx.clone(), None);
    client_arc.connect_spawn();

    // id 配对
    let track_map: Arc<DashMap<String, ReqTrack>> = Arc::new(DashMap::new());

    // 响应序列化线程
    let (resp_tx, resp_rx) = tokio::sync::mpsc::channel(1024 * 8);
    let track_map_clone = track_map.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Received message: {:?}", msg);
            let response: ApiResponse = serde_json::from_str(msg.as_str())
                .map_err(|e| {
                    info!("Error parsing message: {:?}", e);
                })
                .unwrap();

            // 移除匹配id
            match &response {
                ApiResponse::Error(data) => {
                    if let Some(id) = data.id.clone() {
                        if let Some(mut track) = track_map_clone.get_mut(&id) {
                            track.resp_time = Some(current_time_nanos());
                            info!("Error response: {:?}", data);
                        } else {
                            info!("No matching id found for error response: {:?}", data);
                        }
                    }
                }
                ApiResponse::Success(data) => {
                    let id = data.id.clone();
                    if let Some(mut track) = track_map_clone.get_mut(&id) {
                        track.resp_time = Some(current_time_nanos());
                    } else {
                        info!("No matching id found for success response: {:?}", data);
                    }
                }
                ApiResponse::RawData(_) => {}
            }

            resp_tx
                .send(response)
                .await
                .map_err(|e| {
                    info!("Error sending message: {:?}", e);
                })
                .unwrap();
        }
    });

    let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel::<WsReqMethod>(1024 * 8);
    let track_map_clone = track_map.clone();
    tokio::spawn(async move {
        while let Some(ws_req) = outbound_rx.recv().await {
            let uuid = Uuid::new_v4();
            // 取前几位
            let uuid = &uuid.to_string()[..8];
            let base_req = match ws_req {
                WsReqMethod::OrderPlace(req) => BaseReq::new(
                    format!("newOrder_{uuid}").as_str(),
                    WsReqMethod::OrderPlace(req),
                ),
                WsReqMethod::CancelOrder(req) => BaseReq::new(
                    format!("cancelOrder_{uuid}").as_str(),
                    WsReqMethod::CancelOrder(req),
                ),
                WsReqMethod::PositionRisk(req) => BaseReq::new(
                    format!("positionRisk_{uuid}").as_str(),
                    WsReqMethod::PositionRisk(req),
                ),
                WsReqMethod::OrderStatus(req) => BaseReq::new(
                    format!("orderStatus_{uuid}").as_str(),
                    WsReqMethod::OrderStatus(req),
                ),
                WsReqMethod::UserDataStreamStart(req) => BaseReq::new(
                    format!("userDataStreamStart_{uuid}").as_str(),
                    WsReqMethod::UserDataStreamStart(req),
                ),
                WsReqMethod::UserDataStreamPing(req) => BaseReq::new(
                    format!("userDataStreamPing_{uuid}").as_str(),
                    WsReqMethod::UserDataStreamPing(req),
                ),
                WsReqMethod::UserDataStreamStop(req) => BaseReq::new(
                    format!("userDataStreamPtop_{uuid}").as_str(),
                    WsReqMethod::UserDataStreamStop(req),
                ),
                WsReqMethod::Time => {
                    BaseReq::new(format!("ping_{uuid}").as_str(), WsReqMethod::Time)
                }
                WsReqMethod::Ping => {
                    BaseReq::new(format!("time_{uuid}").as_str(), WsReqMethod::Ping)
                }
            };
            info!("Sending request: {:?}", base_req);
            let mut track = ReqTrack {
                id: base_req.id.clone(),
                start_send_time: Some(current_time_nanos()),
                end_send_time: None,
                resp_time: None,
            };

            loop {
                match client_arc
                    .send_message(&base_req.serialize().as_str())
                    .await
                {
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        info!("Error sending message: {:?}", err);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
            track.end_send_time = Some(current_time_nanos());
            track_map_clone.insert(base_req.id.clone(), track);
        }
    });

    let track_map_clone = track_map.clone();
    // 启动一个task, 打印id 报错信息
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let mut to_remove = vec![];
            for track in track_map_clone.iter() {
                if track.resp_time.is_some() {
                    to_remove.push(track.key().clone());
                    info!(
                        "Pending response for id: {}, send cost: {} us, receive cost: {} us",
                        track.id,
                        (track.end_send_time.unwrap() - track.start_send_time.unwrap_or(0)) as f64
                            / 1000.0,
                        (track.resp_time.unwrap() - track.start_send_time.unwrap_or(0)) as f64
                            / 1000.0
                    );
                }
            }
            for key in to_remove {
                track_map_clone.remove(&key);
            }
        }
    });
    (resp_rx, outbound_tx)
}

#[cfg(test)]
mod tests {
    use log::info;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    use base_util::ws::WsClient;

    use crate::model::{ApiResponse, OrderSide, PositionSide, ResultType, TimeInForce};
    use crate::req_param::{
        BaseReq, CancelOrder, KeyPair, OrderPlace, OrderStatus, PositionRisk, UserDataStream,
        WsReqMethod,
    };
    use crate::ws_fapi::start_ws_fapi;

    fn setup() -> Runtime {
        let _ = env_logger::init();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("tk-thread")
            .enable_all()
            .build()
            .unwrap();
        runtime
    }

    #[test]
    fn test_2() {
        let runtime = setup();

        runtime.block_on(async {
            let key_pair = Arc::new(KeyPair {
                api_key: "3ec340c3bf6399c711d2b0e34f8cefff48c2aed984c7c5157c813810a027ee45"
                    .to_string(),
                secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477"
                    .to_string(),
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
                            info!("Error: {:?}", err);
                        }
                        ApiResponse::Success(row_data) => match row_data.result {
                            ResultType::Order(data) => {
                                info!("Order data: {:#?}", data);
                            }
                            ResultType::PositionRisk(data) => {
                                info!("Position risk data: {:?}", data);
                            }
                            ResultType::UserDataStream(data) => {
                                info!("User data stream: {:?}", data);
                            }
                            ResultType::ServerTime(data) => {
                                info!("Server time: {:?}", data);
                            }
                            ResultType::EmptyBody(data) => {
                                info!("Empty body: {:?}", data);
                            }
                        },
                        ApiResponse::RawData(row_data) => {
                            info!("Raw data: {:?}", row_data);
                        }
                    }
                }
            });

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        });
    }

    #[test]
    fn test_aync() {
        let runtime = setup();

        runtime.block_on(async {
            let key_pair = Arc::new(KeyPair {
                api_key: "3ec340c3bf6399c711d2b0e34f8cefff48c2aed984c7c5157c813810a027ee45"
                    .to_string(),
                secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477"
                    .to_string(),
            });

            let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);

            // wss://testnet.binancefuture.com/ws-fapi/v1
            let base_url = "wss://testnet.binancefuture.com/ws-fapi/v1";
            let client_arc = WsClient::new(base_url, None, tx.clone(), None);
            client_arc.connect_spawn();

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    info!("Received message: {:?} \n", msg);
                    let response: ApiResponse = serde_json::from_str(msg.as_str())
                        .map_err(|e| {
                            info!("Error parsing message: {:?}", e);
                        })
                        .unwrap();
                    info!("Received message: {:?}", response);
                }
            });
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let place_market = OrderPlace::place_market(
                key_pair.clone(),
                "BTCUSDT",
                &OrderSide::Buy,
                &PositionSide::Long,
                &Decimal::from_str("0.01").unwrap(),
                Some("test_order_id"),
            );
            let place_market_req =
                BaseReq::new("place_market", WsReqMethod::OrderPlace(place_market));

            // place limit
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
            let place_limit_req = BaseReq::new("place_limit", WsReqMethod::OrderPlace(place_limit));

            let time_req = BaseReq::new("time", WsReqMethod::Time);
            let ping_req = BaseReq::new("ping", WsReqMethod::Ping);

            let position_risk = PositionRisk::new(key_pair.clone(), None, None);
            let position_risk_req =
                BaseReq::new("position_risk", WsReqMethod::PositionRisk(position_risk));

            // 4372849252
            let cancel_order = CancelOrder::new(
                key_pair.clone(),
                "BTCUSDT",
                None,
                Some("test_order_id"),
                Some(9999_9999),
            );
            let cancel_order_req =
                BaseReq::new("cancel_order", WsReqMethod::CancelOrder(cancel_order));

            let order_status = OrderStatus::new(
                key_pair.clone(),
                "BTCUSDT",
                None,
                Some("test_order_id"),
                Some(9999_9999),
            );
            let order_status_req =
                BaseReq::new("order_status", WsReqMethod::OrderStatus(order_status));

            let user_data_stream_start_req = BaseReq::new(
                "user_data_stream_start",
                WsReqMethod::UserDataStreamStart(UserDataStream::new(key_pair.clone())),
            );
            let user_data_stream_ping_req = BaseReq::new(
                "user_data_stream_ping",
                WsReqMethod::UserDataStreamPing(UserDataStream::new(key_pair.clone())),
            );
            let user_data_stream_stop_req = BaseReq::new(
                "user_data_stream_stop",
                WsReqMethod::UserDataStreamStop(UserDataStream::new(key_pair.clone())),
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
                info!("Sending request {}: {:?}", i, req);
                match client_arc.send_message(req.serialize().as_str()).await {
                    Ok(_) => {}
                    Err(err) => {
                        info!("Error sending message: {:?}", err);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            // Keep the main thread alive to receive messages
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }
}
