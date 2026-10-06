// Wire models: names mirror Binance's documentation field by field; they are
// documented where the meaning isn't obvious. Full coverage comes later.
#![allow(missing_docs)]

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    derivatives::usds_margined_futures::{KlineInterval, Path},
    serde::deserialize_json,
};

/// Identifier echoed back by the server. Accepts a 64-bit signed integer or an
/// alphanumeric string up to 36 characters.
#[derive(PartialEq, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum MessageID {
    Str(String),
    Int(i64),
}

impl From<String> for MessageID {
    fn from(s: String) -> Self {
        MessageID::Str(s)
    }
}

impl From<&str> for MessageID {
    fn from(s: &str) -> Self {
        MessageID::Str(s.to_string())
    }
}

impl From<i64> for MessageID {
    fn from(n: i64) -> Self {
        MessageID::Int(n)
    }
}

/// Subset of stream names supported by USDⓈ-M Futures market data streams.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamName {
    /// `<symbol>@aggTrade`
    AggTrade { symbol: String },
    /// `<symbol>@kline_<interval>`
    Kline {
        symbol: String,
        interval: KlineInterval,
    },
    /// `<symbol>@markPrice` — 1s update
    MarkPrice { symbol: String },
    /// `<symbol>@depth`
    Depth { symbol: String },
    /// `<symbol>@forceOrder` — liquidation order
    ForceOrder { symbol: String },
    /// "!forceOrder@arr" — all-symbol liquidation
    ForceOrderAll,
}

impl StreamName {
    /// URL segment that serves this stream: [`Path::Public`] for order book
    /// streams (`depth`, `bookTicker`), [`Path::Market`] for the rest.
    ///
    /// Since 2026-03 `wss://fstream.binance.com` routes market streams only
    /// under `/market`: subscribing to `aggTrade`, `kline`, `markPrice`, ...
    /// on the root `/ws` or `/stream` path opens the connection but never
    /// delivers data. A combined stream connection therefore only carries
    /// streams of one segment, e.g. `wss://fstream.binance.com/market/stream`.
    pub fn path(&self) -> Path {
        match self {
            Self::Depth { .. } => Path::Public,
            Self::AggTrade { .. }
            | Self::Kline { .. }
            | Self::MarkPrice { .. }
            | Self::ForceOrder { .. }
            | Self::ForceOrderAll => Path::Market,
        }
    }
}

impl Serialize for StreamName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::AggTrade { symbol } => format!("{symbol}@aggTrade"),
            Self::Kline { symbol, interval } => format!("{symbol}@kline_{interval}"),
            Self::MarkPrice { symbol } => format!("{symbol}@markPrice"),
            Self::Depth { symbol } => format!("{symbol}@depth"),
            Self::ForceOrder { symbol } => format!("{symbol}@forceOrder"),
            Self::ForceOrderAll => String::from("!forceOrder@arr"),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for StreamName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s == "!forceOrder@arr" {
            return Ok(Self::ForceOrderAll);
        }
        let (symbol, kind) = s
            .split_once('@')
            .ok_or_else(|| serde::de::Error::custom("invalid stream format"))?;
        match kind {
            "aggTrade" => Ok(Self::AggTrade {
                symbol: symbol.to_owned(),
            }),
            "markPrice" => Ok(Self::MarkPrice {
                symbol: symbol.to_owned(),
            }),
            "depth" => Ok(Self::Depth {
                symbol: symbol.to_owned(),
            }),
            "forceOrder" => Ok(Self::ForceOrder {
                symbol: symbol.to_owned(),
            }),
            kind => {
                if let Some(("kline", params)) = kind.split_once('_') {
                    let interval = format!("\"{params}\"");
                    let interval = deserialize_json(&interval)
                        .map_err(|_| serde::de::Error::custom("invalid kline interval"))?;
                    Ok(Self::Kline {
                        symbol: symbol.to_owned(),
                        interval,
                    })
                } else {
                    Err(serde::de::Error::custom(format!(
                        "unknown stream type: {kind}"
                    )))
                }
            }
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "method")]
pub enum OutgoingMessage {
    #[serde(rename = "SUBSCRIBE")]
    Subscribe {
        id: Option<MessageID>,
        params: Vec<StreamName>,
    },
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe {
        id: Option<MessageID>,
        params: Vec<StreamName>,
    },
    #[serde(rename = "LIST_SUBSCRIPTIONS")]
    ListSubscriptions { id: Option<MessageID> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_are_routed_to_public_or_market() {
        use crate::derivatives::usds_margined_futures::KlineInterval;
        let symbol = || "btcusdt".to_string();
        assert_eq!(StreamName::Depth { symbol: symbol() }.path(), Path::Public);
        for stream in [
            StreamName::AggTrade { symbol: symbol() },
            StreamName::Kline {
                symbol: symbol(),
                interval: KlineInterval::Minute1,
            },
            StreamName::MarkPrice { symbol: symbol() },
            StreamName::ForceOrder { symbol: symbol() },
            StreamName::ForceOrderAll,
        ] {
            assert_eq!(stream.path(), Path::Market, "{stream:?}");
        }
    }
}
