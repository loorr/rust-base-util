use log::{debug, error};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use base_util::signature::signature;
use base_util::time::current_time_millis;

use crate::model::{OrderSide, PositionSide, TimeInForce};

const DEFAULT_RECV_WINDOW: u64 = 99999999;

pub struct KeyPair {
    pub api_key: String,
    pub secret_key: String,
}

pub trait Signature {
    fn sign(&mut self, key_pair: &KeyPair) -> String;
    fn set_api_key(&mut self, key_pair: &KeyPair);
    fn set_timestamp(&mut self, timestamp: Option<u64>);
    fn set_signature(&mut self, signature: Option<String>);

    fn serialize(&self) -> String
    where
        Self: Serialize,
    {
        serde_json::to_string(self).unwrap_or_else(|e| {
            error!("Failed to serialize to JSON: {}", e);
            "{}".to_string()
        })
    }
}

pub enum WsReqMethod {
    OrderPlace(OrderPlace),
    CancelOrder(CancelOrder),
    PositionRisk(PositionRisk),
    OrderStatus(OrderStatus),
    UserDataStreamStart(UserDataStream),
    UserDataStreamPing(UserDataStream),
    UserDataStreamStop(UserDataStream),
    Time,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseReq {
    pub id: String,
    method: String,
    params: Option<String>,
}

impl BaseReq {
    pub fn new(id: &str, method: WsReqMethod) -> Self {
        let mut base_req = BaseReq {
            id: id.to_string(),
            method: "".to_string(),
            params: None,
        };

        match method {
            WsReqMethod::OrderPlace(req) => {
                base_req.method = "order.place".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::CancelOrder(req) => {
                base_req.method = "order.cancel".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::PositionRisk(req) => {
                base_req.method = "v2/account.position".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::OrderStatus(req) => {
                base_req.method = "order.status".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::UserDataStreamStart(req) => {
                base_req.method = "userDataStream.start".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::UserDataStreamPing(req) => {
                base_req.method = "userDataStream.ping".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::UserDataStreamStop(req) => {
                base_req.method = "userDataStream.stop".to_string();
                base_req.params = Some(Signature::serialize(&req));
            }
            WsReqMethod::Time => {
                base_req.method = "v1/time".to_string();
            }
            WsReqMethod::Ping => {
                base_req.method = "v1/ping".to_string();
            }
            _ => {}
        }

        base_req
    }

    pub fn serialize(&self) -> String {
        match &self.params {
            None => format!("{{\"id\":\"{}\",\"method\":\"{}\"}}", self.id, self.method),
            Some(params) => format!(
                "{{\"id\":\"{}\",\"method\":\"{}\",\"params\":{}}}",
                self.id, self.method, params
            ),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderPlace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    symbol: String,
    side: OrderSide,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_side: Option<PositionSide>, // 持仓方向,单向持仓模式下非必填,默认且仅可填BOTH;在双向持仓模式下必填,且仅可选择 LONG 或 SHORT
    #[serde(rename = "type")]
    order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduce_only: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<Decimal>, // 以合约数量计量的下单数量，使用closePosition不支持此参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<String>, // 价格
    #[serde(skip_serializing_if = "Option::is_none")]
    new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    close_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activation_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_rate: Option<String>, // [0.1, 4]
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    working_type: Option<String>, // MARK_PRICE or CONTRACT_PRICE
    #[serde(skip_serializing_if = "Option::is_none")]
    price_protect: Option<String>, // "TRUE" or "FALSE"
    #[serde(skip_serializing_if = "Option::is_none")]
    new_order_resp_type: Option<String>, // "ACK", "RESULT", "FULL"
    #[serde(skip_serializing_if = "Option::is_none")]
    recv_window: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    price_match: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    self_trade_prevention_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl OrderPlace {
    pub fn place_market(
        key_pair: &KeyPair,
        symbol: &str,
        side: &OrderSide,
        position_side: &PositionSide,
        quantity: &Decimal,
        new_client_order_id: Option<&str>,
    ) -> Self {
        let mut req = OrderPlace {
            api_key: Some(key_pair.api_key.clone()),
            symbol: symbol.to_string(),
            side: side.clone(),
            position_side: Some(position_side.clone()),
            order_type: "MARKET".to_string(),
            quantity: Some(quantity.clone()),
            new_client_order_id: new_client_order_id.map(|s| s.to_string()),
            new_order_resp_type: Some("RESULT".to_string()),
            recv_window: Some(9999_9999),
            timestamp: Some(current_time_millis()),
            ..Default::default()
        };
        let sign = req.sign(key_pair);
        req.set_signature(Some(sign));
        req
    }

    pub fn place_limit(
        key_pair: &KeyPair,
        symbol: &str,
        side: &OrderSide,
        position_side: &PositionSide,
        quantity: &Decimal,
        price: &Decimal,
        new_client_order_id: Option<&str>,
        time_in_force: TimeInForce,
        recv_window: Option<u64>,
    ) -> Self {
        let mut req = OrderPlace {
            api_key: Some(key_pair.api_key.clone()),
            symbol: symbol.to_string(),
            side: side.clone(),
            position_side: Some(position_side.clone()),
            order_type: "LIMIT".to_string(),
            quantity: Some(quantity.clone()),
            price: Some(price.to_string()),
            new_client_order_id: new_client_order_id.map(|s| s.to_string()),
            new_order_resp_type: Some("ACK".to_string()),
            recv_window,
            timestamp: Some(current_time_millis()),
            time_in_force: Some(time_in_force),
            ..Default::default()
        };
        let sign = req.sign(key_pair);
        req.set_signature(Some(sign));
        req
    }
}

impl Signature for OrderPlace {
    fn sign(&mut self, key_pair: &KeyPair) -> String {
        let mut sign_string = vec![];
        if let Some(api_key) = &self.api_key {
            sign_string.push(format!("apiKey={}", api_key))
        }
        if let Some(price) = &self.price {
            sign_string.push(format!("price={}", price));
        }
        if let Some(quantity) = &self.quantity {
            sign_string.push(format!("quantity={}", quantity));
        }
        sign_string.push(format!("side={}", self.side));
        sign_string.push(format!("symbol={}", self.symbol));
        if let Some(reduce_only) = &self.reduce_only {
            sign_string.push(format!("reduceOnly={}", reduce_only));
        }
        if let Some(position_side) = &self.position_side {
            sign_string.push(format!("positionSide={}", position_side));
        }

        if let Some(new_client_order_id) = &self.new_client_order_id {
            sign_string.push(format!("newClientOrderId={}", new_client_order_id));
        }
        if let Some(stop_price) = &self.stop_price {
            sign_string.push(format!("stopPrice={}", stop_price));
        }
        if let Some(close_position) = &self.close_position {
            sign_string.push(format!("closePosition={}", close_position));
        }
        if let Some(activation_price) = &self.activation_price {
            sign_string.push(format!("activationPrice={}", activation_price));
        }
        if let Some(callback_rate) = &self.callback_rate {
            sign_string.push(format!("callbackRate={}", callback_rate));
        }
        if let Some(time_in_force) = &self.time_in_force {
            sign_string.push(format!("timeInForce={}", time_in_force));
        }
        if let Some(working_type) = &self.working_type {
            sign_string.push(format!("workingType={}", working_type));
        }
        if let Some(price_protect) = &self.price_protect {
            sign_string.push(format!("priceProtect={}", price_protect));
        }
        if let Some(new_order_resp_type) = &self.new_order_resp_type {
            sign_string.push(format!("newOrderRespType={}", new_order_resp_type));
        }
        if let Some(recv_window) = &self.recv_window {
            sign_string.push(format!("recvWindow={}", recv_window));
        }
        if let Some(price_match) = &self.price_match {
            sign_string.push(format!("priceMatch={}", price_match));
        }
        if let Some(self_trade_prevention_mode) = &self.self_trade_prevention_mode {
            sign_string.push(format!(
                "selfTradePreventionMode={}",
                self_trade_prevention_mode
            ));
        }
        sign_string.push(format!("type={}", self.order_type));
        match self.timestamp {
            None => {
                sign_string.push(format!("timestamp={}", current_time_millis()));
            }
            Some(time) => {
                sign_string.push(format!("timestamp={}", time));
            }
        }
        sign_string.sort_by(|a, b| a.split('=').next().cmp(&b.split('=').next()));
        let sign_row = sign_string.join("&");
        debug!("sign row: {}", sign_row);
        signature(key_pair.secret_key.as_bytes(), sign_row)
    }
    fn set_api_key(&mut self, key_pair: &KeyPair) {
        self.api_key = Some(key_pair.api_key.clone());
    }

    fn set_timestamp(&mut self, timestamp: Option<u64>) {
        self.timestamp = timestamp;
    }

    fn set_signature(&mut self, signature: Option<String>) {
        self.signature = signature;
    }
}

/// symbol	STRING	YES	交易对 <br>
/// orderId	LONG	NO	系统订单号 <br>
/// origClientOrderId	STRING	NO	用户自定义的订单号 <br>
/// recvWindow	LONG	NO <br>
/// timestamp	LONG	YES <br>
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrder {
    pub api_key: Option<String>,
    pub signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl CancelOrder {
    pub fn new(
        key_pair: &KeyPair,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        recv_window: Option<u64>,
    ) -> Self {
        let mut req = CancelOrder {
            api_key: Some(key_pair.api_key.clone()),
            order_id,
            symbol: symbol.to_string(),
            orig_client_order_id: orig_client_order_id.map(|s| s.to_string()),
            recv_window,
            timestamp: Some(current_time_millis()),
            ..Default::default()
        };
        let sign = req.sign(key_pair);
        req.set_signature(Some(sign));
        req
    }
}

impl Signature for CancelOrder {
    fn sign(&mut self, key_pair: &KeyPair) -> String {
        let mut sign_string = vec![];
        if let Some(api_key) = &self.api_key {
            sign_string.push(format!("apiKey={}", api_key))
        }
        if let Some(order_id) = &self.order_id {
            sign_string.push(format!("orderId={}", order_id));
        }
        if let Some(orig_client_order_id) = &self.orig_client_order_id {
            sign_string.push(format!("origClientOrderId={}", orig_client_order_id));
        }
        let window = match self.recv_window {
            None => DEFAULT_RECV_WINDOW,
            Some(window) => window,
        };
        sign_string.push(format!("recvWindow={}", window));
        sign_string.push(format!("symbol={}", self.symbol));
        sign_string.push(format!("timestamp={}", self.timestamp.unwrap()));

        sign_string.sort_by(|a, b| a.split('=').next().cmp(&b.split('=').next()));
        let sign_row = sign_string.join("&");
        debug!("sign row: {}", sign_row);
        signature(key_pair.secret_key.as_bytes(), sign_row)
    }

    fn set_api_key(&mut self, key: &KeyPair) {
        self.api_key = Some(key.api_key.clone());
    }

    fn set_timestamp(&mut self, timestamp: Option<u64>) {
        self.timestamp = timestamp;
    }

    fn set_signature(&mut self, signature: Option<String>) {
        self.signature = signature;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionRisk {
    pub api_key: Option<String>,
    pub signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl PositionRisk {
    pub fn new(key_pair: &KeyPair, symbol: Option<String>, recv_window: Option<u64>) -> Self {
        let mut req = PositionRisk {
            api_key: Some(key_pair.api_key.clone()),
            symbol,
            recv_window,
            ..Default::default()
        };
        let sign = req.sign(key_pair);
        req.set_signature(Some(sign));
        req
    }
}

impl Signature for PositionRisk {
    fn sign(&mut self, key_pair: &KeyPair) -> String {
        let mut sign_string = vec![];
        if let Some(api_key) = &self.api_key {
            sign_string.push(format!("apiKey={}", api_key))
        }
        let window = self.recv_window.unwrap_or_else(|| DEFAULT_RECV_WINDOW);
        self.recv_window = Some(window);
        sign_string.push(format!("recvWindow={}", window));
        if let Some(symbol) = &self.symbol {
            sign_string.push(format!("symbol={}", symbol));
        }
        let timestamp = self.timestamp.unwrap_or_else(|| current_time_millis());
        self.timestamp = Some(timestamp);
        sign_string.push(format!("timestamp={}", timestamp));
        sign_string.sort_by(|a, b| a.split('=').next().cmp(&b.split('=').next()));
        let sign_row = sign_string.join("&");
        debug!("sign row: {}", sign_row);
        signature(key_pair.secret_key.as_bytes(), sign_row)
    }

    fn set_api_key(&mut self, key: &KeyPair) {
        self.api_key = Some(key.api_key.clone());
    }

    fn set_timestamp(&mut self, timestamp: Option<u64>) {
        self.timestamp = timestamp;
    }

    fn set_signature(&mut self, signature: Option<String>) {
        self.signature = signature;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatus {
    pub api_key: Option<String>,
    pub signature: Option<String>,
    pub symbol: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl OrderStatus {
    pub fn new(
        key_pair: &KeyPair,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        recv_window: Option<u64>,
    ) -> Self {
        let mut req = OrderStatus {
            api_key: Some(key_pair.api_key.clone()),
            symbol: symbol.to_string(),
            order_id,
            orig_client_order_id: orig_client_order_id.map(|s| s.to_string()),
            recv_window,
            timestamp: Some(current_time_millis()),
            ..Default::default()
        };
        let sign = req.sign(key_pair);
        req.set_signature(Some(sign));
        req
    }
}

impl Signature for OrderStatus {
    fn sign(&mut self, key_pair: &KeyPair) -> String {
        let mut sign_string = vec![];
        if let Some(api_key) = &self.api_key {
            sign_string.push(format!("apiKey={}", api_key))
        }
        if let Some(order_id) = &self.order_id {
            sign_string.push(format!("orderId={}", order_id));
        }
        if let Some(orig_client_order_id) = &self.orig_client_order_id {
            sign_string.push(format!("origClientOrderId={}", orig_client_order_id));
        }
        let window = self.recv_window.unwrap_or_else(|| DEFAULT_RECV_WINDOW);
        self.recv_window = Some(window);
        sign_string.push(format!("recvWindow={}", window));
        sign_string.push(format!("symbol={}", self.symbol));
        let timestamp = self.timestamp.unwrap_or_else(|| current_time_millis());
        self.timestamp = Some(timestamp);
        sign_string.push(format!("timestamp={}", timestamp));
        sign_string.sort_by(|a, b| a.split('=').next().cmp(&b.split('=').next()));
        let sign_row = sign_string.join("&");
        debug!("sign row: {}", sign_row);
        signature(key_pair.secret_key.as_bytes(), sign_row)
    }

    fn set_api_key(&mut self, key_pair: &KeyPair) {
        self.api_key = Some(key_pair.api_key.clone());
    }

    fn set_timestamp(&mut self, timestamp: Option<u64>) {
        self.timestamp = timestamp;
    }

    fn set_signature(&mut self, signature: Option<String>) {
        self.signature = signature;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataStream {
    pub api_key: String,
}

impl UserDataStream {
    pub fn new(key_pair: &KeyPair) -> Self {
        UserDataStream {
            api_key: key_pair.api_key.clone(),
        }
    }
}
impl Signature for UserDataStream {
    fn sign(&mut self, _key_pair: &KeyPair) -> String {
        "".to_string()
    }

    fn set_api_key(&mut self, key_pair: &KeyPair) {
        self.api_key = key_pair.api_key.clone();
    }

    fn set_timestamp(&mut self, _timestamp: Option<u64>) {}

    fn set_signature(&mut self, _signature: Option<String>) {}
}
