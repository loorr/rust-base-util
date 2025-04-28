use base_util::ws::WsClient;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Utf8Bytes;

// 返回 发送订单 和 接受消息结果
// 是否能转换为同步调用, 通过wake机制
// 发送订单后, 监听消息里面的id, 匹配
pub async fn start_ws_fapi(base_url: &str) -> (Receiver<Utf8Bytes>, Arc<WsClient>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);
    let client_arc = WsClient::new(base_url, None, tx.clone(), None);
    client_arc.connect_spawn();
    (rx, client_arc)
}
