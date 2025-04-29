use std::collections::HashMap;
use std::sync::Arc;

use log::{error, info};
use reqwest::{Client, Method};
use tokio::time::{Duration, Instant};

use base_util::time::current_time_millis;

use crate::model::{
    BookTickers, CancelAllOpenOrdersResp, EmptyBody, ErrorDetail, ExchangeInformation, OrderResp,
    PositionRiskResp, ServerTime, UserDataStreamResp,
};
use crate::req_param::KeyPair;

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
    pub fn new(
        base_url: &str,
        key_pair: Arc<KeyPair>,
        recv_window: Option<u64>,
        timeout: Option<u64>,
    ) -> Self {
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
    pub async fn get_server_time(&self) -> Result<ServerTime, ErrorDetail> {
        let uri = "/fapi/v1/time";
        let s = Instant::now();
        let result = self
            .send_request(Method::GET, uri)
            .await?;
        let elapsed = s.elapsed();
        println!("get_server_time elapsed: {:?}", elapsed.as_millis());
        result
    }

    /// /fapi/v1/listenKey
    pub async fn start(&self) -> Result<UserDataStreamResp, ErrorDetail> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<UserDataStreamResp>(Method::POST, uri)
            .await
    }

    /// /fapi/v1/listenKey
    pub async fn keep_alive(&self) -> Result<UserDataStreamResp, ErrorDetail> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<UserDataStreamResp>(Method::PUT, &uri)
            .await
    }

    /// /fapi/v1/listenKey
    pub async fn close(&self) -> Result<EmptyBody, ErrorDetail> {
        let uri = "/fapi/v1/listenKey";
        self.send_request::<EmptyBody>(Method::DELETE, &uri).await
    }

    /// /fapi/v1/ticker/bookTicker
    pub async fn book_ticker(&self, symbol: &str) -> Result<BookTickers, ErrorDetail> {
        let uri = format!("/fapi/v1/ticker/bookTicker?symbol={}", symbol);
        self.send_request::<BookTickers>(Method::GET, &uri).await
    }

    /// /fapi/v1/exchangeInfo
    pub async fn exchange_info(&self) -> Result<ExchangeInformation, ErrorDetail> {
        let uri = "/fapi/v1/exchangeInfo";
        self.send_request::<ExchangeInformation>(Method::GET, &uri)
            .await
    }

    /// /fapi/v3/positionRisk
    pub async fn position_risk(
        &self,
        symbol: &str,
    ) -> Result<Vec<PositionRiskResp>, ErrorDetail> {
        let params = format!(
            "symbol={}&recvWindow={}&timestamp={}",
            symbol.to_lowercase(),
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
            current_time_millis()
        );
        let uri = format!(
            "/fapi/v3/positionRisk?{}",
            self.key_pair.signature_str(&params)
        );
        self.send_request::<Vec<PositionRiskResp>>(Method::GET, &uri)
            .await
    }

    /// /fapi/v1/allOpenOrders
    pub async fn cancel_all_open_orders(
        &self,
        symbol: &str,
    ) -> Result<CancelAllOpenOrdersResp, ErrorDetail> {
        let params = format!(
            "symbol={}&recvWindow={}&timestamp={}",
            symbol.to_lowercase(),
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
            current_time_millis()
        );
        let uri = format!(
            "/fapi/v1/allOpenOrders?{}",
            self.key_pair.signature_str(&params)
        );
        self.send_request::<CancelAllOpenOrdersResp>(Method::DELETE, &uri)
            .await
    }

    /// GET /fapi/v1/order
    pub async fn open_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<OrderResp, ErrorDetail> {
        if order_id.is_none() && orig_client_order_id.is_none() {
            return Err(ErrorDetail {
                code: -1,
                msg: "Either order_id or orig_client_order_id must be provided".to_string(),
            });
        }
        let mut params = HashMap::new();
        params.insert("symbol", symbol.to_lowercase());
        params.insert(
            "recvWindow",
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string(),
        );
        params.insert("timestamp", current_time_millis().to_string());
        if let Some(id) = order_id {
            params.insert("orderId", id.to_string());
        }
        if let Some(orig_id) = orig_client_order_id {
            params.insert("origClientOrderId", orig_id.to_string());
        }
        let uri = format!("/fapi/v1/order?{}", self.key_pair.signature(&params));
        self.send_request::<OrderResp>(Method::GET, &uri).await
    }

    /// DELETE /fapi/v1/order
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<OrderResp, ErrorDetail> {
        if order_id.is_none() && orig_client_order_id.is_none() {
            return Err(ErrorDetail {
                code: -1,
                msg: "Either order_id or orig_client_order_id must be provided".to_string(),
            });
        }
        let mut params = HashMap::new();
        params.insert("symbol", symbol.to_lowercase());
        params.insert(
            "recvWindow",
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string(),
        );
        params.insert("timestamp", current_time_millis().to_string());
        if let Some(id) = order_id {
            params.insert("orderId", id.to_string());
        }
        if let Some(orig_id) = orig_client_order_id {
            params.insert("origClientOrderId", orig_id.to_string());
        }
        let uri = format!("/fapi/v1/order?{}", self.key_pair.signature(&params));
        self.send_request::<OrderResp>(Method::DELETE, &uri).await
    }

    async fn send_request<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        method: Method,
        endpoint: &str,
    ) -> Result<T, ErrorDetail> {
        let url = format!("{}{}", self.base_url, endpoint);
        info!("Sending {} request to {}", method, url);

        let start_time = Instant::now(); // Start timing the request

        let request_builder = match method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url),
            Method::DELETE => self.client.delete(&url),
            Method::PUT => self.client.put(&url),
            _ => {
                return Err(ErrorDetail {
                    code: -1,
                    msg: format!("Unsupported HTTP method: {}", method),
                });
            }
        };

        let result = match request_builder
            .header(DEFAULT_MBX_APIKEY, &self.key_pair.api_key)
            .timeout(Duration::from_millis(
                self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            ))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<T>().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("Failed to parse response: {}", err);
                            return Err(ErrorDetail {
                                code: -1,
                                msg: format!("Failed to parse response: {}", err),
                            });
                        }
                    }
                } else {
                    match response.json::<ErrorDetail>().await {
                        Ok(data) => Err(data),
                        Err(err) => Err(ErrorDetail {
                            code: -1,
                            msg: format!("Failed to parse error response: {}", err),
                        }),
                    }?
                }
            }
            Err(err) => {
                Err(ErrorDetail {
                    code: -1,
                    msg: format!("Request failed: {}", err),
                })
            }?
        };

        let elapsed_time = start_time.elapsed(); // Calculate elapsed time
        info!(
            "Request to {} completed in {} ms",
            url,
            elapsed_time.as_millis()
        );

        Ok(result)
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
            secret_key: "559052c703589a3f5259395c240c524a3bc7d98cac89f9c3d0e54f1ba5a48477"
                .to_string(),
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
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_request() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api
            .send_request::<ServerTime>(Method::GET, "/fapi/v1/time")
            .await
        {
            Ok(time) => println!("{:?}", time),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_user_data_stream() {
        let (key_pair, base_url) = init();

        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.start().await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
        match api.keep_alive().await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
        match api.close().await {
            Ok(_) => println!("Closed successfully"),
            Err(e) => eprintln!("Error: {:?}", e),
        }
        match api.start().await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_book_ticker() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.book_ticker("BTCUSDT").await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_exchange_info() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.exchange_info().await {
            Ok(resp) => println!("{:?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_position_risk() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.position_risk("BTCUSDT").await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_cancel_all_open_orders() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.cancel_all_open_orders("BTCUSDT").await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api
            .cancel_order("BTCUSDT", Some(1234567890), Some("test_order"))
            .await
        {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_open_order() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api
            .open_order("BTCUSDT1", None, Some("test_order"))
            .await
        {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
