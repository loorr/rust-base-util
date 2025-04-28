use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error, info};
use reqwest::{Client, Method};
use tokio::time::{Duration, Instant};
use base_util::time::current_time_millis;
use crate::model::{BookTickers, EmptyBody, ExchangeInformation, PositionRiskResp, ServerTime, UserDataStreamResp};
use crate::req_param::{KeyPair};

// ms
const DEFAULT_TIMEOUT: u64 = 10_000;
const DEFAULT_RECV_WINDOW: u64 = 9999_999;
const DEFAULT_MBX_APIKEY: &str = "X-MBX-APIKEY";


struct FuturesApi {
    client: Client,
    base_url: String,
    recv_window: Option<u64>,
    timeout: Option<u64>,
    key_pair: Arc<KeyPair>,
}

impl FuturesApi {
    pub fn new(base_url: &str, key_pair: Arc<KeyPair>, recv_window: Option<u64>, timeout: Option<u64>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout.unwrap_or(DEFAULT_TIMEOUT)))
            .user_agent("rust-binance-futures-api/1.0") // Add a user agent
            .build()
            .expect("Failed to build HTTP client"); // Improve error message

        FuturesApi {
            client,
            base_url: base_url.to_string(),
            key_pair,
            recv_window,
            timeout,
        }
    }

    /// /fapi/v1/time
    pub async fn get_server_time(&self) -> Result<ServerTime, String> {
        let uri = "/fapi/v1/time";
        let s = Instant::now();
        let result = self.send_request(Method::GET, uri).await.map_err(|e| format!("Failed to get server time: {}", e));
        let elapsed = s.elapsed();
        println!("get_server_time elapsed: {:?}", elapsed.as_millis());
        result
    }

    /// /fapi/v1/listenKey
    pub async fn start(&self) -> Result<UserDataStreamResp, String> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<UserDataStreamResp>(Method::POST, uri).await
    }

    /// /fapi/v1/listenKey
    pub async fn keep_alive(&self) -> Result<UserDataStreamResp, String> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<UserDataStreamResp>(Method::PUT, &uri).await
    }

    /// /fapi/v1/listenKey
    pub async fn close(&self) -> Result<EmptyBody, String> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<EmptyBody>(Method::DELETE, &uri).await
    }

    /// /fapi/v1/ticker/bookTicker
    pub async fn book_ticker(&self, symbol: &str) -> Result<BookTickers, String> {
        let uri = format!("/fapi/v1/ticker/bookTicker?symbol={}", symbol);
        self.send_request::<BookTickers>(Method::GET, &uri).await
    }

    /// /fapi/v1/exchangeInfo
    pub async fn exchange_info(&self) -> Result<ExchangeInformation, String> {
        let uri = "/fapi/v1/exchangeInfo";
        self.send_request::<ExchangeInformation>(Method::GET, &uri).await
    }

    /// /fapi/v3/positionRisk
    pub async fn position_risk(&self, symbol: &str) -> Result<Vec<PositionRiskResp>, String> {
        let mut params_map = HashMap::new();
        params_map.insert("symbol", symbol.to_lowercase());
        params_map.insert("recvWindow", self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string());
        params_map.insert("timestamp", current_time_millis().to_string());
        let params_str = self.key_pair.signature(&params_map);
        let uri = format!("/fapi/v3/positionRisk?{}", params_str);
        self.send_request::<Vec<PositionRiskResp>>(Method::GET, &uri).await
    }

    async fn send_request<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        method: Method,
        endpoint: &str,
    ) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        info!("Sending {} request to {}", method, url);

        let start_time = Instant::now(); // Start timing the request

        let request_builder = match method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url),
            Method::DELETE => self.client.delete(&url),
            Method::PUT => self.client.put(&url),
            _ => {
                error!("Unsupported HTTP method: {}", method);
                return Err("Unsupported HTTP method".to_string());
            }
        };

        let result = match request_builder
            .header(DEFAULT_MBX_APIKEY, &self.key_pair.api_key)
            .timeout(Duration::from_millis(self.timeout.unwrap_or(DEFAULT_TIMEOUT)))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let json = response.json().await;
                    info!("request url: {}|{}| response: {:?}", method, url, json);
                    match json {
                        Ok(parsed) => Ok(parsed),
                        Err(err) => {
                            error!("Failed to parse response: {}", err);
                            Err(format!("Failed to parse response: {}", err))
                        }
                    }
                } else {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    error!("HTTP error: {}, body: {}", status, text);
                    Err(format!("HTTP error: {}, body: {}", status, text))
                }
            }
            Err(err) => {
                error!("Request failed: {}", err);
                Err(format!("Request failed: {}", err))
            }
        };

        let elapsed_time = start_time.elapsed(); // Calculate elapsed time
        info!(
            "Request to {} completed in {} ms",
            url,
            elapsed_time.as_millis()
        );

        result
    }
}


#[cfg(test)]
mod tests {
    use reqwest::Method;

    use crate::req_param::KeyPair;

    use super::*;

    fn init() -> (KeyPair, String) {
        env_logger::init();
        let key_pair = KeyPair {
            api_key: "3ec340c3bf6399c711d2b0e34f8cefff48c2aed984c7c5157c813810a027ee45".to_string(),
            secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477".to_string(),
        };
        let base_url = "https://testnet.binancefuture.com";
        (key_pair, base_url.to_string())
    }

    #[tokio::test]
    async fn test_get_server_time() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.get_server_time().await {
            Ok(time) => println!("{:?}", time),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_send_request() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.send_request::<ServerTime>(Method::GET, "/fapi/v1/time").await {
            Ok(time) => println!("{:?}", time),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_user_data_stream() {
        let (key_pair, base_url) = init();

        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.start().await {
            Ok(resp) => println!("{:?}", resp.listen_key),
            Err(e) => eprintln!("Error: {}", e),
        }
        match api.keep_alive().await {
            Ok(resp) => println!("{:?}", resp.listen_key),
            Err(e) => eprintln!("Error: {}", e),
        }
        match api.close().await {
            Ok(_) => println!("Closed successfully"),
            Err(e) => eprintln!("Error: {}", e),
        }
        match api.start().await {
            Ok(resp) => println!("{:?}", resp.listen_key),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_book_ticker() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.book_ticker("BTCUSDT").await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_exchange_info() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.exchange_info().await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_position_risk() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.position_risk("BTCUSDT").await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
