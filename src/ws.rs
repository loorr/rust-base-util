use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::IntoClientRequest;
use tungstenite::Utf8Bytes;


/// WebSocket client
/// params:
/// - url: WebSocket URL
/// - reconnect_interval: Reconnect interval in milliseconds
/// - tx: Channel to send messages to the outside world
/// - name: Name of the connection
#[derive(Debug, Clone)]
pub struct WsClient {
    name: String,
    url: String,

    // 重连的时间间隔
    reconnect_interval: Duration,

    // 对外界发送 ws 收到的消息
    tx: Sender<Utf8Bytes>,

    // 发送消息给 ws
    write_tx: broadcast::Sender<Message>,

    // 当前是否连通
    is_connected: Arc<AtomicBool>,

    // shutdown 消息发送通道
    shutdown_tx: broadcast::Sender<()>,
}

#[derive(Debug)]
pub enum SendError {
    NotConnected,
    ChannelClosed,
}

impl WsClient {
    pub fn new(
        url: &str,
        reconnect_interval: Option<u64>,
        tx: Sender<Utf8Bytes>,
        name: Option<&str>,
    ) -> Arc<Self> {
        // 设置这个连接特别的名称
        let binding = rand::random::<u64>().to_string();
        let name = name.unwrap_or(&*binding);

        // 用作发送消息的通道
        let (write_tx, _) = broadcast::channel(32);

        // 用作停止连接的通道
        let (shutdown_tx, _) = broadcast::channel(1);

        let client = WsClient {
            name: name.to_string(),
            url: url.to_string(),
            reconnect_interval: Duration::from_millis(reconnect_interval.unwrap_or(20)),
            tx,
            write_tx,
            is_connected: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
        };
        Arc::new(client)
    }

    pub fn connect_spawn(&self) {
        let self_cone = self.clone();
        tokio::task::spawn(async move {
            self_cone.connect().await;
        });
    }

    async fn connect(&self) {
        let write_tx = self.write_tx.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            warn!("[{}] Try to connect to {}", &self.name, &self.url);
            let request = self.url.clone().into_client_request().unwrap();

            let connect_result = tokio::select! {
                result = connect_async(request) => result,
                _ = shutdown_rx.recv() => {
                    warn!("[{}] Shutdown signal received. Exiting connect loop.", &self.name);
                    return;
                }
            };
            let name = self.name.clone();
            match connect_result {
                Ok((ws_stream, _)) => {
                    warn!("Successfully connected to {}", self.url);
                    self.is_connected.store(true, Ordering::Release);

                    let (mut write, mut read) = ws_stream.split();
                    let tx_clone = self.tx.clone();
                    let write_tx_clone = self.write_tx.clone();
                    let mut write_rx = write_tx.subscribe();

                    let mut shutdown_rx_send = self.shutdown_tx.subscribe(); // For send task
                    let mut shutdown_rx_read = self.shutdown_tx.subscribe(); // For read task

                    // send
                    let name_send = name.clone();
                    let is_connected_clone_send = self.is_connected.clone();
                    let _send_task = tokio::spawn(async move {
                        let mut ping_interval = interval(Duration::from_secs(30));
                        loop {
                            tokio::select! {
                                message_to_send = write_rx.recv() => {
                                    match message_to_send {
                                        Ok(message) => {
                                            if let Err(e) = write.send(message).await {
                                                error!("Error sending message: {}", e);
                                                break;
                                            }
                                        }
                                        Err(broadcast::error::RecvError::Closed) => {
                                            warn!("Write channel closed. Exiting send loop.");
                                            break;
                                        }
                                        Err(broadcast::error::RecvError::Lagged(_)) => {
                                            warn!("Send loop lagged. Some messages may have been missed.");
                                        }
                                    }
                                },
                                _ = ping_interval.tick() => {
                                    if let Err(e) = write.send(Message::Ping(Vec::new().into())).await {
                                        error!("Error sending ping: {}", e);
                                        break;
                                    }
                                },
                                 _ = shutdown_rx_send.recv() => { // Check for shutdown signal
                                    info!("[{}] Send task: Shutdown signal received.", name_send);
                                    break;
                                }

                            }
                        }
                        info!("[{}] Send loop exited.", &name_send);
                        is_connected_clone_send.store(false, Ordering::Release);
                    });

                    // read
                    let name_read = name.clone();
                    let is_connected_clone_read = self.is_connected.clone();
                    let _read_task = tokio::spawn(async move {
                        loop {
                            tokio::select! {
                                message_result = read.next() => {
                                      match message_result {
                                            Some(Ok(message)) => match message {
                                                Message::Text(text) => {
                                                    if let Err(e) = tx_clone.send(text).await {
                                                        error!("Error sending message to channel: {}", e);
                                                        break;
                                                    }
                                                }
                                                Message::Ping(data) => {
                                                    if let Err(err) = write_tx_clone.send(Message::Pong(data)) {
                                                        error!("send pong message error:{}", err);
                                                    };
                                                }
                                                Message::Close(_) => {
                                                    warn!("Received close message from server.");
                                                    break;
                                                }
                                                _ => {}
                                            },
                                            Some(Err(e)) => {
                                                error!("Error reading from WebSocket: {}", e);
                                                break;
                                            }
                                            None => {  // Stream closed
                                              warn!("[{}] Read stream closed.", &name_read);
                                              break;
                                           }
                                        }

                                },
                                _ = shutdown_rx_read.recv() => {
                                   info!("[{}] Read task: Shutdown signal received.", &name_read);
                                   break;
                                }

                            }
                        }
                        is_connected_clone_read.store(false, Ordering::Release);
                        warn!("[{}] Read loop exited.", name_read);
                    });

                    // Keep the task alive, and listen for shutdown signal
                    let mut main_shutdown_rx = self.shutdown_tx.subscribe(); // For the main loop
                    tokio::select! {
                        _ = async {
                                while self.is_connected() {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                            } => {
                                warn!("Connection lost, attempting to reconnect...");
                            }
                        _ = main_shutdown_rx.recv() => {
                            info!("[{}] Main loop: Shutdown signal received.", &self.name);
                              // No need to join the tasks here; they will be aborted when the runtime shuts down.
                             break;  // Exit the main loop
                        }
                    }
                }
                Err(e) => {
                    error!("[{}] Failed to connect: {}", &self.name, e);
                    self.is_connected.store(false, Ordering::Release);
                }
            }
            sleep(self.reconnect_interval).await;
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<(), SendError> {
        if !&self.is_connected() {
            return Err(SendError::NotConnected);
        }
        self.write_tx
            .send(Message::Text(message.into()))
            .map_err(|_| SendError::ChannelClosed)?;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }
}

impl Drop for WsClient {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        info!("[{}] WsClient dropped.", &self.name);
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;
    use super::*;
    use log::info;
    use tokio::runtime::Runtime;
    use tungstenite::Utf8Bytes;

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

    async fn print(rx: &mut tokio::sync::mpsc::Receiver<Utf8Bytes>, timeout: Duration) {
        let start_time = Instant::now();
        let mut buffer: Vec<Utf8Bytes> = Vec::with_capacity(2);
        let limit = 10;
        loop {
            let size = rx.recv_many(&mut buffer, limit).await;
            if size == 0 {
                break;
            }
            for message in &buffer[..size] {
                info!("{}", message);
            }
            buffer.clear();

            if start_time.elapsed().as_millis() > timeout.as_millis() {
                break;
            }
        }
    }

    #[test]
    fn test_ws() {
        // 设置线程名称
        let runtime = setup();

        runtime.block_on(async {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);

            let base_url = "wss://fstream.binance.com/ws/btcusdt@trade";
            let client_arc = WsClient::new(base_url, None, tx.clone(), Some("btcusdt"));
            client_arc.connect_spawn();

            // let fstream_url = "wss://fstream.binance.com/ws/ethusdt@bookTicker";
            // let client_2_arc = BinanceWsClient::new(fstream_url, None, tx, Some("ethusdt"));
            // client_2_arc.connect_spawn();

            print(&mut rx, Duration::from_secs(5)).await;
            info!("WebSocket client finished.");
        });

        info!("done")
    }

    #[test]
    fn test_bybit() {
        let runtime = setup();

        runtime.block_on(async {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1024 * 8);

            let base_url = "wss://stream.bybit.com/v5/public/spot";
            let client_arc = WsClient::new(base_url, None, tx.clone(), Some("btcusdt"));
            client_arc.connect_spawn();

            let msg = r#"{
                    "req_id": "test",
                    "op": "subscribe",
                    "args": [
                        "orderbook.1.BTCUSDT"
                    ]
                }"#;
            info!("{}", msg);
            loop {
                if !client_arc.is_connected() {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                if let Err(e) = client_arc.send_message(msg).await {
                    error!("Failed to send subscription message: {:?}", e);
                }
                break;
            }

            print(&mut rx, Duration::from_secs(5)).await;
            info!("WebSocket client finished.");
        });

        info!("done")
    }
}
