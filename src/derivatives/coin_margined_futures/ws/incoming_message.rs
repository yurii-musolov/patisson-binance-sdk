use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    Timestamp,
    derivatives::coin_margined_futures::{
        KlineInterval, OrderSide,
        http::OrderLevel,
        ws::{MessageID, StreamName},
    },
    ws::ReceivedMessage,
};

#[derive(PartialEq, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum IncomingMessage {
    CombinedStream(CombinedStreamMessage<StreamMessage>),
    Stream(StreamMessage),
    Response(ResponseMessage),
}

impl ReceivedMessage for IncomingMessage {
    fn server_shutdown_event_time(&self) -> Option<u64> {
        // COIN-M Futures market data streams do not emit an app-level
        // serverShutdown event the way spot does — we rely on the shared
        // driver's heartbeat / close handling.
        None
    }
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct ResponseMessage {
    pub id: Option<MessageID>,
    pub result: Option<serde_json::Value>,
    pub status: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CombinedStreamMessage<T> {
    pub stream: StreamName,
    pub data: T,
}

#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "e")]
pub enum StreamMessage {
    #[serde(rename = "aggTrade")]
    AggTrade(AggTradeMsg),
    #[serde(rename = "kline")]
    Kline(KlineMsg),
    #[serde(rename = "indexPriceUpdate")]
    IndexPriceUpdate(IndexPriceUpdateMsg),
    #[serde(rename = "markPriceUpdate")]
    MarkPriceUpdate(MarkPriceUpdateMsg),
    #[serde(rename = "forceOrder")]
    ForceOrder(ForceOrderMsg),
    #[serde(rename = "depthUpdate")]
    DepthUpdate(DepthUpdateMsg),
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct AggTradeMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Symbol (e.g. BTCUSD_PERP)
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "a")]
    pub trade_id: i64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub qty: Decimal,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    #[serde(rename = "T")]
    pub trade_time: Timestamp,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct KlineMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: Kline,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct Kline {
    #[serde(rename = "t")]
    pub start_time: Timestamp,
    #[serde(rename = "T")]
    pub close_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: KlineInterval,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "o")]
    pub open_price: Decimal,
    #[serde(rename = "c")]
    pub close_price: Decimal,
    #[serde(rename = "h")]
    pub high_price: Decimal,
    #[serde(rename = "l")]
    pub low_price: Decimal,
    /// Base asset volume — for COIN-M this is in contracts.
    #[serde(rename = "v")]
    pub base_asset_volume: Decimal,
    #[serde(rename = "n")]
    pub trade_number: i64,
    #[serde(rename = "x")]
    pub is_closed: bool,
    /// Quote asset volume.
    #[serde(rename = "q")]
    pub quote_asset_volume: Decimal,
    #[serde(rename = "V")]
    pub taker_buy_base_asset_volume: Decimal,
    #[serde(rename = "Q")]
    pub taker_buy_quote_asset_volume: Decimal,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct IndexPriceUpdateMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Pair (e.g. BTCUSD).
    #[serde(rename = "i")]
    pub pair: String,
    #[serde(rename = "p")]
    pub index_price: Decimal,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct MarkPriceUpdateMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub mark_price: Decimal,
    /// Estimated settle price (only useful in the last hour before settlement).
    #[serde(rename = "P")]
    pub estimated_settle_price: Decimal,
    /// Funding rate — only meaningful for perpetual contracts.
    #[serde(rename = "r")]
    pub funding_rate: Option<Decimal>,
    /// Next funding time — only meaningful for perpetual contracts.
    #[serde(rename = "T")]
    pub next_funding_time: Option<Timestamp>,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct ForceOrderMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "o")]
    pub order: ForceOrderEntry,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct ForceOrderEntry {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub orig_qty: Decimal,
    #[serde(rename = "ap")]
    pub avg_price: Decimal,
    #[serde(rename = "z")]
    pub executed_qty: Decimal,
    #[serde(rename = "T")]
    pub trade_time: Timestamp,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct DepthUpdateMsg {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    /// Pair (e.g. BTCUSD).
    #[serde(rename = "ps")]
    pub pair: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    /// Final update id in the previous stream event (i.e. `u` of the prior
    /// event). Used to verify the diff-stream chain has no gaps.
    #[serde(rename = "pu")]
    pub previous_final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<OrderLevel>,
    #[serde(rename = "a")]
    pub asks: Vec<OrderLevel>,
}
