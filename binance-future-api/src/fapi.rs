use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error, info};
use reqwest::{Client, Method};
use tokio::time::{Duration, Instant};

use base_util::time::current_time_millis;

use crate::model::{
    BatchOrderResp, BookTickers, CancelAllOpenOrdersResp, EmptyBody, ErrorDetail,
    ExchangeInformation, OrderResp, PositionRiskResp, ServerTime, UserDataStreamResp,
};
use crate::req_param::{KeyPair, OrderPlace};

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
            .user_agent("rust-binance-futures-api/1.0")
            // .http2_prior_knowledge() // 跳过协议协商，强制使用 HTTP/2（需确保服务器支持）
            .pool_idle_timeout(Some(Duration::from_secs(1800)))
            .tcp_keepalive(Duration::from_secs(1800))
            .pool_max_idle_per_host(20)
            .build()
            .expect("Failed to build HTTP client");

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
        self.send_request(Method::GET, uri).await
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
    pub async fn position_risk(&self, symbol: &str) -> Result<Vec<PositionRiskResp>, ErrorDetail> {
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
    pub async fn order(
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

    /// POST /fapi/v1/order
    pub async fn new_order(&self, mut req: OrderPlace) -> Result<OrderResp, ErrorDetail> {
        let uri = format!(
            "/fapi/v1/order?{}",
            self.key_pair.signature_str(&req.params_row())
        );
        println!("uri: {}", uri);
        self.send_request::<OrderResp>(Method::POST, &uri).await
    }

    /// /fapi/v1/batchOrders
    ///
    /// 其中batchOrders应以list of JSON格式填写订单参数
    ///
    /// 例子: /fapi/v1/batchOrders?batchOrders=[{"type":"LIMIT","timeInForce":"GTC",
    /// "symbol":"BTCUSDT","side":"BUY","price":"10001","quantity":"0.001"}]
    pub async fn batch_orders(
        &self,
        orders: Vec<OrderPlace>,
    ) -> Result<Vec<BatchOrderResp>, ErrorDetail> {
        todo!("Batch orders not implemented yet");
        let mut params = HashMap::new();
        params.insert(
            "recvWindow",
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string(),
        );
        params.insert("timestamp", current_time_millis().to_string());
        let batch_orders = serde_json::to_string(&orders).map_err(|e| ErrorDetail {
            code: -1,
            msg: format!("Failed to serialize orders: {}", e),
        })?;
        println!("batch_orders: {}", batch_orders);
        params.insert("batchOrders", batch_orders);
        let uri = format!("/fapi/v1/batchOrders?{}", self.key_pair.signature(&params));
        self.send_request::<Vec<BatchOrderResp>>(Method::POST, &uri)
            .await
    }

    /// GET /fapi/v1/openOrders
    ///
    /// 请求权重
    /// + 带symbol 1
    /// + 不带 40 请小心使用不带symbol参数的调用
    pub async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<OrderResp>, ErrorDetail> {
        let mut params = HashMap::new();
        if let Some(sym) = symbol {
            params.insert("symbol", sym.to_lowercase());
        }
        params.insert(
            "recvWindow",
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string(),
        );
        params.insert("timestamp", current_time_millis().to_string());
        if let Some(sym) = symbol {
            params.insert("symbol", sym.to_lowercase());
        }
        params.insert(
            "recvWindow",
            self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW).to_string(),
        );
        let uri = format!("/fapi/v1/openOrders?{}", self.key_pair.signature(&params));
        self.send_request::<Vec<OrderResp>>(Method::GET, &uri).await
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
                let status_code = response.status();
                let text = response.text().await.map_err(|e| ErrorDetail {
                    code: -1,
                    msg: format!("Failed to read response text: {}", e),
                })?;
                if status_code.is_success() {
                    match serde_json::from_str::<T>(&text) {
                        Ok(data) => data,
                        Err(err) => {
                            error!("Failed to parse response: {}", err);
                            return Err(ErrorDetail {
                                code: -1,
                                msg: format!("Failed to parse response: {}, text: {}", err, text),
                            });
                        }
                    }
                } else {
                    match serde_json::from_str::<ErrorDetail>(&text) {
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
            }?,
        };

        let elapsed_time = start_time.elapsed(); // Calculate elapsed time
        debug!(
            "Request to {} completed in {} ms",
            url,
            elapsed_time.as_millis()
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{OrderSide, PositionSide, TimeInForce};
    use crate::req_param::KeyPair;
    use reqwest::Method;
    use rust_decimal::Decimal;
    use std::str::FromStr;

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
        match api.order("BTCUSDT1", None, Some("test_order")).await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_open_orders() {
        let (key_pair, base_url) = init();
        let api = FuturesApi::new(base_url.as_str(), Arc::new(key_pair), None, None);
        match api.open_orders(None).await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_new_order() {
        let (key_pair, base_url) = init();
        let key_pair = Arc::new(key_pair);
        let api = FuturesApi::new(base_url.as_str(), key_pair.clone(), None, None);
        let place_market = OrderPlace::place_market(
            key_pair.clone(),
            "BTCUSDT",
            &OrderSide::Buy,
            &PositionSide::Long,
            &Decimal::from_str("0.01").unwrap(),
            Some("test_order_id"),
        );

        match api.new_order(place_market).await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_batch_orders() {
        let (key_pair, base_url) = init();
        let key_pair = Arc::new(key_pair);
        let api = FuturesApi::new(base_url.as_str(), key_pair.clone(), None, None);
        let place_market = OrderPlace::place_market(
            key_pair.clone(),
            "BTCUSDT",
            &OrderSide::Buy,
            &PositionSide::Long,
            &Decimal::from_str("0.01").unwrap(),
            Some("test_order_id_1"),
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
        match api.batch_orders(vec![place_market, place_limit]).await {
            Ok(resp) => println!("{:#?}", resp),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
