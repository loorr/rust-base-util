use std::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};

use crate::marco::string_or_float;
use crate::marco::string_or_float_opt;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u32,
    pub limit: i32,
    pub count: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: i64,
    pub msg: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub id: Option<String>,
    pub status: u32,
    pub rate_limits: Option<Vec<RateLimit>>,
    pub error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultType {
    Order(OrderResp),
    PositionRisk(Vec<PositionRiskResp>),
    UserDataStream(UserDataStreamResp),
    ServerTime(ServerTime),
    EmptyBody(EmptyBody),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub id: String,
    pub status: u16,
    pub rate_limits: Vec<RateLimit>,
    pub result: ResultType,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Error(ErrorResponse),
    Success(SuccessResponse),
    RawData(serde_json::Value), // 兜底类型
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    #[default]
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    #[default]
    Both,
    Long,
    Short,
}

impl fmt::Display for PositionSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionSide::Both => write!(f, "BOTH"),
            PositionSide::Long => write!(f, "LONG"),
            PositionSide::Short => write!(f, "SHORT"),
        }
    }
}

/// How long will an order stay alive
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum TimeInForce {
    /// Good Till Canceled
    GTC,
    /// Immediate Or Cancel
    IOC,
    /// Fill or Kill
    FOK,
    /// Good till expired
    GTX,
    #[serde(other)]
    Other,
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TimeInForce::GTC => write!(f, "GTC"),
            TimeInForce::IOC => write!(f, "IOC"),
            TimeInForce::FOK => write!(f, "FOK"),
            TimeInForce::GTX => write!(f, "GTX"),
            TimeInForce::Other => write!(f, "OTHER"),
        }
    }
}

/// Status of an order, this can typically change over time
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// The order has been accepted by the engine.
    New,
    /// A part of the order has been filled.
    PartiallyFilled,
    /// The order has been completely filled.
    Filled,
    /// The order has been canceled by the user.
    Canceled,
    /// Currently unused
    PendingCancel,
    /// The order was not accepted by the engine and not processed.
    Rejected,
    /// The order was canceled according to the order type's rules (e.g. LIMIT FOK orders with no fill, LIMIT IOC or MARKET orders that partially fill) or by the exchange, (e.g. orders canceled during liquidation, orders canceled during maintenance)
    Expired,
    /// The order was canceled by the exchange due to STP trigger. (e.g. an order with EXPIRE_TAKER will match with existing orders on the book with the same account or same tradeGroupId)
    ExpiredInMatch,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkingType {
    MarkPrice,
    ContractPrice,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderResp {
    pub order_id: u64,
    pub symbol: String,
    pub status: OrderStatus,
    pub client_order_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub avg_price: f64,
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    #[serde(with = "string_or_float")]
    pub cum_qty: f64,
    #[serde(with = "string_or_float")]
    pub cum_quote: f64,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub type_name: OrderType,
    pub reduce_only: bool,
    pub close_position: bool,
    pub side: OrderSide,
    pub position_side: PositionSide,
    #[serde(with = "string_or_float")]
    pub stop_price: f64,
    pub working_type: WorkingType,
    pub price_protect: bool,
    pub orig_type: OrderType,
    pub price_match: String,                // todo
    pub self_trade_prevention_mode: String, // todo
    pub good_till_date: u64,
    pub update_time: u64,

    #[serde(default)]
    #[serde(with = "string_or_float_opt")]
    pub activate_price: Option<f64>,
    #[serde(default)]
    #[serde(with = "string_or_float_opt")]
    pub price_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDataStreamResp {
    pub listen_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmptyBody {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PositionRiskResp {
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_amt: String,
    pub entry_price: String,
    /// 未实现盈亏恰好为零的价格。这通常包括了交易手续费。它比 entryPrice 略高，就是因为手续费
    pub break_even_price: String,
    pub mark_price: String,
    pub un_realized_profit: String,
    pub liquidation_price: String,
    pub isolated_margin: String,
    pub notional: String,
    pub margin_asset: String,
    pub isolated_wallet: String,
    /// 开仓时最初需要的保证金金额
    pub initial_margin: String,
    /// 维持保证金要求。这是你必须维持以保持仓位开放的最低保证金金额
    pub maint_margin: String,
    pub position_initial_margin: String,
    pub open_order_initial_margin: String,
    //  integer, not a float
    pub adl: i32,
    //  though 0, could be a float in other cases
    pub bid_notional: String,
    //  though 0, could be a float in other cases
    pub ask_notional: String,
    pub update_time: u64,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookTickers {
    pub symbol: String,
    #[serde(with = "string_or_float")]
    pub bid_price: f64,
    #[serde(with = "string_or_float")]
    pub bid_qty: f64,
    #[serde(with = "string_or_float")]
    pub ask_price: f64,
    #[serde(with = "string_or_float")]
    pub ask_qty: f64,
}


/// Rate Limit Interval, used by RateLimitType
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterval {
    Second,
    Minute,
    Day,
}

/// API Rate Limit
/// Example
/// {
///   "rateLimitType": "REQUEST_WEIGHT",
///   "interval": "MINUTE",
///   "intervalNum": 1,
///   "limit": 1200
/// }
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RequestWeight,
    Orders,
    RawRequests,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RestRateLimit {
    pub interval: RateLimitInterval,
    pub rate_limit_type: RateLimitType,
    pub interval_num: i32,
    pub limit: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "filterType")]
pub enum Filters {
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter {
        // #[serde(with = "string_or_float")]
        min_price: String,
        // #[serde(with = "string_or_float")]
        max_price: String,
        // #[serde(with = "string_or_float")]
        tick_size: String,
    },
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize {
        // #[serde(with = "string_or_float")]
        min_qty: String,
        // #[serde(with = "string_or_float")]
        max_qty: String,
        // #[serde(with = "string_or_float")]
        step_size: String,
    },
    #[serde(rename = "MARKET_LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    MarketLotSize {
        min_qty: String,
        max_qty: String,
        step_size: String,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumOrders { limit: u16 },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumAlgoOrders { limit: u16 },
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional {
        // #[serde(with = "string_or_float")]
        notional: String,
    },
    #[serde(rename = "PERCENT_PRICE")]
    #[serde(rename_all = "camelCase")]
    PercentPrice {
        // #[serde(with = "string_or_float")]
        multiplier_up: String,
        // #[serde(with = "string_or_float")]
        multiplier_down: String,
        // #[serde(with = "string_or_float")]
        multiplier_decimal: String,
    },
    #[serde(other)]
    Others,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetail {
    pub asset: String,
    pub margin_available: bool,
    #[serde(with = "string_or_float")]
    pub auto_asset_exchange: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Perpetual,
    CurrentMonth,
    NextMonth,
    CurrentQuarter,
    NextQuarter,
    #[serde(rename = "CURRENT_QUARTER DELIVERING")]
    CurrentQuarterDelivery,
    PerpetualDelivering,
    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub pair: String,
    pub contract_type: ContractType,
    pub delivery_date: u64,
    pub onboard_date: u64,
    pub status: SymbolStatus,
    #[serde(with = "string_or_float")]
    pub maint_margin_percent: f64,
    #[serde(with = "string_or_float")]
    pub required_margin_percent: f64,
    pub base_asset: String,
    pub quote_asset: String,
    pub price_precision: u16,
    pub quantity_precision: u16,
    pub base_asset_precision: u64,
    pub quote_precision: u64,
    pub underlying_type: String,
    pub underlying_sub_type: Vec<String>,
    pub settle_plan: Option<u64>,
    #[serde(with = "string_or_float")]
    pub trigger_protect: f64,
    pub filters: Vec<Filters>,
    pub order_types: Vec<OrderType>,
    pub time_in_force: Vec<TimeInForce>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    PreTrading,
    Trading,
    PostTrading,
    EndOfDay,
    Halt,
    AuctionMatch,
    Break,
    PendingTrading,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    pub timezone: String,
    pub server_time: u64,
    pub futures_type: String,
    pub rate_limits: Vec<RestRateLimit>,
    pub exchange_filters: Vec<Filters>,
    pub assets: Vec<AssetDetail>,
    pub symbols: Vec<Symbol>,
}

// test mod
#[cfg(test)]
mod test {
    use crate::model::{ErrorResponse, OrderResp, SuccessResponse};

    #[test]
    fn test() {
        let json_str = r#"
            {
            "orderId": 4375249344,
            "symbol": "BTCUSDT",
            "status": "FILLED",
            "clientOrderId": "test_order_id",
            "price": "0.00",
            "avgPrice": "94546.70000",
            "origQty": "0.010",
            "executedQty": "0.010",
            "cumQty": "0.010",
            "cumQuote": "945.46700",
            "timeInForce": "GTC",
            "type": "MARKET",
            "reduceOnly": false,
            "closePosition": false,
            "side": "BUY",
            "positionSide": "LONG",
            "stopPrice": "0.00",
            "workingType": "CONTRACT_PRICE",
            "priceProtect": false,
            "origType": "MARKET",
            "priceMatch": "NONE",
            "selfTradePreventionMode": "EXPIRE_MAKER",
            "goodTillDate": 0,
            "updateTime": 1745824127143
        }
        "#;
        let _response: OrderResp = serde_json::from_str(json_str).unwrap();
    }

    #[test]
    fn test_base() {
        let json_str = r#"
        {
    "id": "place_market",
    "status": 200,
    "result": {
        "orderId": 4375869951,
        "symbol": "BTCUSDT",
        "status": "FILLED",
        "clientOrderId": "test_order_id",
        "price": "0.00",
        "avgPrice": "95200.10000",
        "origQty": "0.010",
        "executedQty": "0.010",
        "cumQty": "0.010",
        "cumQuote": "952.00100",
        "timeInForce": "GTC",
        "type": "MARKET",
        "reduceOnly": false,
        "closePosition": false,
        "side": "BUY",
        "positionSide": "LONG",
        "stopPrice": "0.00",
        "workingType": "CONTRACT_PRICE",
        "priceProtect": false,
        "origType": "MARKET",
        "priceMatch": "NONE",
        "selfTradePreventionMode": "EXPIRE_MAKER",
        "goodTillDate": 0,
        "updateTime": 1745842667589
    },
        "rateLimits": [
            {
                "rateLimitType": "REQUEST_WEIGHT",
                "interval": "MINUTE",
                "intervalNum": 1,
                "limit": -1,
                "count": -1
            },
            {
                "rateLimitType": "ORDERS",
                "interval": "SECOND",
                "intervalNum": 10,
                "limit": 300,
                "count": 1
            },
            {
                "rateLimitType": "ORDERS",
                "interval": "MINUTE",
                "intervalNum": 1,
                "limit": 1200,
                "count": 1
            }
        ]
    }
        "#;

        let _response: SuccessResponse = serde_json::from_str(json_str).unwrap();
    }

    #[test]
    fn test_position_risk() {
        let json_str = r#"
        {
        "id": "position_risk",
        "status": 200,
        "result": [
                {
                    "symbol": "BTCUSDT",
                    "positionSide": "LONG",
                    "positionAmt": "0.290",
                    "entryPrice": "94361.84137932",
                    "breakEvenPrice": "94395.81164222",
                    "markPrice": "95183.00000000",
                    "unRealizedProfit": "238.13599999",
                    "liquidationPrice": "38677.01237264",
                    "isolatedMargin": "0",
                    "notional": "27603.07000000",
                    "marginAsset": "USDT",
                    "isolatedWallet": "0",
                    "initialMargin": "2840.30700000",
                    "maintMargin": "110.41228000",
                    "positionInitialMargin": "2760.30700000",
                    "openOrderInitialMargin": "80",
                    "adl": 1,
                    "bidNotional": "800",
                    "askNotional": "0",
                    "updateTime": 1745843130288
                }
            ],
            "rateLimits": [
                {
                    "rateLimitType": "REQUEST_WEIGHT",
                    "interval": "MINUTE",
                    "intervalNum": 1,
                    "limit": 6000,
                    "count": 12
                }
            ]
        }

        "#;

        let _response: SuccessResponse = serde_json::from_str(json_str).unwrap();
    }

    #[test]
    fn test_error() {
        let json_str = r#"{
            "id": null,
            "status": 400,
            "error": {
                "code": -1000,
                "msg": "Invalid 'id' in JSON request; expected an integer, a string matching '^[a-zA-Z0-9-_]{1,36}$', or null."
            }
        }"#;
        let _response: ErrorResponse = serde_json::from_str(json_str).unwrap();
    }
}
