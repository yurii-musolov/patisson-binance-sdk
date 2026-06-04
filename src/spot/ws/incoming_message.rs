use rust_decimal::Decimal;
use serde::Deserialize;

use crate::spot::{KlineInterval, Timestamp, ws::StreamName};

#[derive(PartialEq, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum IncomingMessage {
    Response {
        result: Option<serde_json::Value>,
        id: String,
    },
    Error {
        code: i64,
        msg: String,
        id: Option<String>,
    },
    StreamEvent(CombinedStreamEvent<StreamEvent>),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CombinedStreamEvent<T> {
    pub stream: StreamName,
    pub data: T,
}

#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "e")]
pub enum StreamEvent {
    #[serde(rename = "aggTrade")]
    AggTrade(EventAggTrade),
    #[serde(rename = "trade")]
    Trade(EventTrade),
    #[serde(rename = "kline")]
    Kline(EventKline),
    #[serde(rename = "24hrMiniTicker")]
    MiniTicker24(EventMiniTicker24),
}

/// The Aggregate Trade Streams push trade information that is aggregated for a single taker order.
#[derive(PartialEq, Deserialize, Debug)]
pub struct EventAggTrade {
    /// Event time
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Aggregate trade ID
    #[serde(rename = "a")]
    pub trade_id: i64,
    /// Price
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// First trade ID
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// Last trade ID
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    /// Trade time
    #[serde(rename = "T")]
    pub trade_time: Timestamp,
    /// Is the buyer the market maker?
    #[serde(rename = "m")]
    pub is_buyer: bool,
}

/// The Trade Streams push raw trade information; each trade has a unique buyer and seller.
#[derive(PartialEq, Deserialize, Debug)]
pub struct EventTrade {
    /// Event time
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Aggregate trade ID
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Price
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// Trade time
    #[serde(rename = "T")]
    pub trade_time: Timestamp,
    /// Is the buyer the market maker?
    #[serde(rename = "m")]
    pub is_buyer: bool,
}

/// The Kline/Candlestick Stream push updates to the current klines/candlestick every second in UTC+0 timezone
#[derive(PartialEq, Deserialize, Debug)]
pub struct EventKline {
    /// Event time
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: KlineMsg,
}
#[derive(PartialEq, Deserialize, Debug)]
pub struct KlineMsg {
    /// Kline start time
    #[serde(rename = "t")]
    pub start_time: Timestamp,
    /// Kline close time
    #[serde(rename = "T")]
    pub close_time: Timestamp,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Interval
    #[serde(rename = "i")]
    pub interval: KlineInterval,
    /// First trade ID
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// Last trade ID
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    /// Open price
    #[serde(rename = "o")]
    pub open_price: Decimal,
    /// Close price
    #[serde(rename = "c")]
    pub close_price: Decimal,
    /// High price
    #[serde(rename = "h")]
    pub high_price: Decimal,
    /// Low price
    #[serde(rename = "l")]
    pub low_price: Decimal,
    /// Base asset volume
    #[serde(rename = "v")]
    pub base_asset_volume: Decimal,
    /// Number of trades
    #[serde(rename = "n")]
    pub trade_number: i64,
    /// Is this kline closed?
    #[serde(rename = "x")]
    pub is_closed: bool,
    /// Quote asset volume
    #[serde(rename = "q")]
    pub quote_asset_volume: Decimal,
    /// Taker buy base asset volume
    #[serde(rename = "V")]
    pub taker_buy_base_asset_volume: Decimal,
    /// Taker buy quote asset volume
    #[serde(rename = "Q")]
    pub taker_buy_quote_asset_volume: Decimal,
}

/// 24hr rolling window mini-ticker statistics. These are NOT the statistics of the UTC day, but a 24hr rolling window for the previous 24hrs.
#[derive(PartialEq, Deserialize, Debug)]
pub struct EventMiniTicker24 {
    /// Event time
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Open price
    #[serde(rename = "o")]
    pub open_price: Decimal,
    /// Close price
    #[serde(rename = "c")]
    pub close_price: Decimal,
    /// High price
    #[serde(rename = "h")]
    pub high_price: Decimal,
    /// Low price
    #[serde(rename = "l")]
    pub low_price: Decimal,
    /// Total traded base asset volume
    #[serde(rename = "v")]
    pub total_base_asset_volume: Decimal,
    /// Total traded quote asset volume
    #[serde(rename = "q")]
    pub total_quote_asset_volume: Decimal,
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::spot::serde::deserialize_json;

    use super::*;

    #[test]
    fn test_deserialize_combined_stream_event() {
        let json = r#"{
            "stream": "bnbbtc@trade",
            "data": "DATA"
        }"#;
        let expected = CombinedStreamEvent {
            stream: StreamName::Trade {
                symbol: String::from("BNBBTC").to_lowercase(),
            },
            data: String::from("DATA"),
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_stream_event_agg_trade() {
        let json = r#"{
            "e": "aggTrade",
            "E": 1672515782136,
            "s": "BNBBTC",
            "a": 12345,
            "p": "0.001",
            "q": "100",
            "f": 100,
            "l": 105,
            "T": 1672515782136,
            "m": true,
            "M": true
        }"#;
        let expected = EventAggTrade {
            event_time: 1672515782136,
            symbol: String::from("BNBBTC"),
            trade_id: 12345,
            price: dec!(0.001),
            qty: dec!(100),
            first_trade_id: 100,
            last_trade_id: 105,
            trade_time: 1672515782136,
            is_buyer: true,
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_stream_event_trade() {
        let json = r#"{
            "e": "trade",
            "E": 1672515782136,
            "s": "BNBBTC",
            "t": 12345,
            "p": "0.001",
            "q": "100",
            "T": 1672515782136,
            "m": true,
            "M": true
        }"#;
        let expected = EventTrade {
            event_time: 1672515782136,
            symbol: String::from("BNBBTC"),
            trade_id: 12345,
            price: dec!(0.001),
            qty: dec!(100),
            trade_time: 1672515782136,
            is_buyer: true,
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_stream_event_kline() {
        let json = r#"{
            "e": "kline",
            "E": 1672515782136,
            "s": "BNBBTC",
            "k": {
                "t": 1672515780000,
                "T": 1672515839999,
                "s": "BNBBTC",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "0.0010",
                "c": "0.0020",
                "h": "0.0025",
                "l": "0.0015",
                "v": "1000",
                "n": 100,
                "x": false,
                "q": "1.0000",
                "V": "500",
                "Q": "0.500",
                "B": "123456"
            }
        }"#;
        let symbol = String::from("BNBBTC");
        let expected = EventKline {
            event_time: 1672515782136,
            symbol: symbol.clone(),
            kline: KlineMsg {
                start_time: 1672515780000,
                close_time: 1672515839999,
                symbol,
                interval: KlineInterval::Minute1,
                first_trade_id: 100,
                last_trade_id: 200,
                open_price: dec!(0.0010),
                close_price: dec!(0.0020),
                high_price: dec!(0.0025),
                low_price: dec!(0.0015),
                base_asset_volume: dec!(1000),
                trade_number: 100,
                is_closed: false,
                quote_asset_volume: dec!(1.0000),
                taker_buy_base_asset_volume: dec!(500),
                taker_buy_quote_asset_volume: dec!(0.500),
            },
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_stream_event_mini_ticker24() {
        let json = r#"{
            "e": "24hrMiniTicker",
            "E": 1672515782136,
            "s": "BNBBTC",
            "c": "0.0025",
            "o": "0.0010",
            "h": "0.0025",
            "l": "0.0010",
            "v": "10000",
            "q": "18"
        }"#;
        let expected = EventMiniTicker24 {
            event_time: 1672515782136,
            symbol: String::from("BNBBTC"),
            open_price: dec!(0.0010),
            close_price: dec!(0.0025),
            high_price: dec!(0.0025),
            low_price: dec!(0.0010),
            total_base_asset_volume: dec!(10000),
            total_quote_asset_volume: dec!(18),
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }
}
