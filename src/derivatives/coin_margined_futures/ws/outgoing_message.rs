use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{derivatives::coin_margined_futures::KlineInterval, serde::deserialize_json};

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

/// Subset of stream names supported by COIN-M Futures market data streams.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamName {
    /// "<symbol>@aggTrade"
    AggTrade { symbol: String },
    /// "<symbol>@kline_<interval>"
    Kline {
        symbol: String,
        interval: KlineInterval,
    },
    /// "<pair>@indexPrice" — index price updates
    IndexPrice { pair: String },
    /// "<symbol>@markPrice"
    MarkPrice { symbol: String },
    /// "<symbol>@depth"
    Depth { symbol: String },
    /// "<symbol>@forceOrder" — liquidation order
    ForceOrder { symbol: String },
    /// "!forceOrder@arr" — all-symbol liquidation
    ForceOrderAll,
}

impl Serialize for StreamName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::AggTrade { symbol } => format!("{symbol}@aggTrade"),
            Self::Kline { symbol, interval } => format!("{symbol}@kline_{interval}"),
            Self::IndexPrice { pair } => format!("{pair}@indexPrice"),
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
        let (sym_or_pair, kind) = s
            .split_once('@')
            .ok_or_else(|| serde::de::Error::custom("invalid stream format"))?;
        match kind {
            "aggTrade" => Ok(Self::AggTrade {
                symbol: sym_or_pair.to_owned(),
            }),
            "indexPrice" => Ok(Self::IndexPrice {
                pair: sym_or_pair.to_owned(),
            }),
            "markPrice" => Ok(Self::MarkPrice {
                symbol: sym_or_pair.to_owned(),
            }),
            "depth" => Ok(Self::Depth {
                symbol: sym_or_pair.to_owned(),
            }),
            "forceOrder" => Ok(Self::ForceOrder {
                symbol: sym_or_pair.to_owned(),
            }),
            kind => {
                if let Some(("kline", params)) = kind.split_once('_').map(|(k, p)| (k, p)) {
                    let interval = format!("\"{params}\"");
                    let interval = deserialize_json(&interval)
                        .map_err(|_| serde::de::Error::custom("invalid kline interval"))?;
                    Ok(Self::Kline {
                        symbol: sym_or_pair.to_owned(),
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
