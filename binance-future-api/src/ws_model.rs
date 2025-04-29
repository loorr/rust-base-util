use serde::{Deserialize, Serialize};
use crate::marco::string_or_u64;
use crate::marco::string_or_float;
use crate::model::{ExecutionType, OrderSide, OrderStatus, OrderType, PositionSide, PriceMatch, SelfTradePreventionMode, TimeInForce, WorkingType};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "e")]
pub enum WebsocketEvent {
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate(Box<AccountUpdate>),
    OrderTradeUpdate(Box<OrderTradeUpdate>),
    #[serde(rename = "TRADE_LITE")]
    TradeLite(Box<TradeLite>),

    #[serde(rename = "bookTicker")]
    BookTicker(Box<BookTicker>),
    #[serde(rename = "trade")]
    Trade(Box<Trade>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    #[serde(rename = "E", with = "string_or_u64")]
    pub event_time: u64,
    #[serde(rename = "T", with = "string_or_u64")]
    pub transaction_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t", with = "string_or_u64")]
    pub trade_id: u64,
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    #[serde(rename = "q", with = "string_or_float")]
    pub quantity: f64,
    #[serde(rename = "X")]
    pub order_type: String, // Renamed from execution_type to be more descriptive
    #[serde(rename = "m")]
    pub is_maker_side: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeLite {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "T", with = "string_or_u64")]
    pub transaction_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q", with = "string_or_float")]
    pub original_quantity: f64,
    #[serde(rename = "p", with = "string_or_float")]
    pub original_price: f64,
    #[serde(rename = "m")]
    pub is_maker_side: bool,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "L", with = "string_or_float")]
    pub last_filled_price: f64,
    #[serde(rename = "l", with = "string_or_float")]
    pub order_last_filled_quantity: f64,
    #[serde(rename = "t", with = "string_or_u64")]
    pub trade_id: u64,
    #[serde(rename = "i", with = "string_or_u64")]
    pub order_id: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderTradeUpdate {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "o")]
    pub order: Order,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Order {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q", with = "string_or_float")]
    pub quantity: f64,
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    #[serde(rename = "ap", with = "string_or_float")]
    pub average_price: f64,
    #[serde(rename = "sp", with = "string_or_float")]
    pub stop_price: f64,
    #[serde(rename = "x")]
    pub execution_type: ExecutionType,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l", with = "string_or_float")]
    pub order_last_filled_quantity: f64,
    #[serde(rename = "z", with = "string_or_float")]
    pub order_filled_accumulated_quantity: f64,
    #[serde(rename = "L", with = "string_or_float")]
    pub last_filled_price: f64,
    #[serde(default, rename = "n", with = "string_or_float_opt")]
    pub commission: Option<f64>,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub order_trade_time: u64,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "b", with = "string_or_float")]
    pub bid_notional: f64,
    #[serde(rename = "a", with = "string_or_float")]
    pub ask_notional: f64,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "R")]
    pub is_reduce: bool,
    #[serde(rename = "wt")]
    pub working_type: WorkingType,
    #[serde(rename = "ot")]
    pub original_order_type: OrderType,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    #[serde(rename = "cp")]
    pub close_position: bool,
    #[serde(default, rename = "AP", with = "string_or_float_opt")]
    pub activation_price: Option<f64>,
    #[serde(default, rename = "cr", with = "string_or_float_opt")]
    pub callback_rate: Option<f64>,
    #[serde(rename = "pP")]
    pub price_protect: bool,
    #[serde(rename = "rp", with = "string_or_float")]
    pub realized_profit: f64,
    #[serde(rename = "V")]
    pub stp_mode: SelfTradePreventionMode,
    #[serde(rename = "pm")]
    pub price_match: PriceMatch,
    #[serde(rename = "gtd")]
    pub good_till_date: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountUpdate {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "a")]
    pub account: Account,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Account {
    #[serde(rename = "m")]
    pub reason_type: ReasonType,
    #[serde(rename = "B")]
    pub balances: Vec<Balance>,
    #[serde(rename = "P")]
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonType {
    Deposit,
    Withdraw,
    Order,
    FundingFee,
    WithdrawReject,
    Adjustment,
    InsuranceClear,
    AdminDeposit,
    AdminWithdraw,
    MarginTransfer,
    MarginTypeChange,
    AssetTransfer,
    OptionsPremiumFee,
    OptionsSettleProfit,
    AutoExchange,
    CoinSwapDeposit,
    CoinSwapWithdraw,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Balance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb", with = "string_or_float")]
    pub wallet_balance: f64,
    #[serde(rename = "cw", with = "string_or_float")]
    pub cross_wallet_balance: f64,
    #[serde(rename = "bc", with = "string_or_float")]
    pub balance_change: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa", with = "string_or_float")]
    pub position_amount: f64,
    #[serde(rename = "ep", with = "string_or_float")]
    pub entry_price: f64,
    #[serde(rename = "bep", with = "string_or_float")]
    pub breakeven_price: f64,
    #[serde(rename = "cr", with = "string_or_float")]
    pub accumulated_realized: f64,
    #[serde(rename = "up", with = "string_or_float")]
    pub unrealized_profit: f64,
    #[serde(rename = "mt")]
    pub margin_type: MarginType,
    #[serde(rename = "iw", with = "string_or_float")]
    pub isolated_wallet: f64,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MarginType {
    Isolated,
    Cross,
}




/// Book ticker event [Reference](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Individual-Symbol-Book-Ticker-Streams)
///
/// example:
/// ```json
/// {
///   "e":"bookTicker",			// event type
///   "u":400900217,     		// order book updateId
///   "E": 1568014460893,  		// event time
///   "T": 1568014460891,  		// transaction time
///   "s":"BNBUSDT",     		// symbol
///   "b":"25.35190000", 		// best bid price
///   "B":"31.21000000", 		// best bid qty
///   "a":"25.36520000", 		// best ask price
///   "A":"40.66000000"  		// best ask qty
/// }
/// ```
///
///
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
    #[serde(rename = "u", with = "string_or_u64")]
    pub update_id: u64,
    #[serde(rename = "E", with = "string_or_u64")]
    pub event_time: u64,
    #[serde(rename = "T", with = "string_or_u64")]
    pub transaction_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b", with = "string_or_float")]
    pub best_bid_price: f64,
    #[serde(rename = "B", with = "string_or_float")]
    pub best_bid_qty: f64,
    #[serde(rename = "a", with = "string_or_float")]
    pub best_ask_price: f64,
    #[serde(rename = "A", with = "string_or_float")]
    pub best_ask_qty: f64,
}