use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::spot::KlineInterval;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamName {
    /// <symbol>@aggTrade
    AggTrade { symbol: String },
    /// <symbol>@trade
    Trade { symbol: String },
    /// <symbol>@depth
    Depth { symbol: String },
    /// <symbol>@kline_<interval>
    Kline {
        symbol: String,
        interval: KlineInterval,
    },
    /// <symbol>@24hrMiniTicker
    MiniTicker24 { symbol: String },
}

impl Serialize for StreamName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::AggTrade { symbol } => format!("{symbol}@aggTrade"),
            Self::Trade { symbol } => format!("{symbol}@trade"),
            Self::Depth { symbol } => format!("{symbol}@depth"),
            Self::Kline { symbol, interval } => format!("{symbol}@kline_{interval}"),
            Self::MiniTicker24 { symbol } => format!("{symbol}@24hrMiniTicker"),
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
        if let Some((symbol, kind)) = s.split_once('@') {
            match kind {
                "aggTrade" => Ok(Self::AggTrade {
                    symbol: symbol.to_owned(),
                }),
                "trade" => Ok(Self::Trade {
                    symbol: symbol.to_owned(),
                }),
                "depth" => Ok(Self::Depth {
                    symbol: symbol.to_owned(),
                }),
                "24hrMiniTicker" => Ok(Self::MiniTicker24 {
                    symbol: symbol.to_owned(),
                }),
                kind => {
                    if let Some((kind, params)) = kind.split_once('_') {
                        match kind {
                            "kline" => {
                                let interval = format!("\"{params}\"");
                                let interval = match serde_json::from_str(&interval) {
                                    Ok(interval) => interval,
                                    Err(_) => {
                                        return Err(serde::de::Error::custom(
                                            "invalid stream format",
                                        ));
                                    }
                                };
                                Ok(Self::Kline {
                                    symbol: symbol.to_owned(),
                                    interval,
                                })
                            }
                            _ => Err(serde::de::Error::custom(format!(
                                "unknown stream type: {kind}"
                            ))),
                        }
                    } else {
                        Err(serde::de::Error::custom("invalid stream format"))
                    }
                }
            }
        } else {
            Err(serde::de::Error::custom("invalid stream format"))
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "method")]
pub enum OutgoingMessage {
    Empty,
    #[serde(rename = "SUBSCRIBE")]
    Subscribe {
        id: String,
        params: Vec<StreamName>,
    },
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe {
        id: String,
        params: Vec<StreamName>,
    },
    #[serde(rename = "LIST_SUBSCRIPTIONS")]
    ListSubscriptions {
        id: String,
    },
    #[serde(rename = "SET_PROPERTY")]
    SetProperty {
        id: String,
        params: (String, bool), // ("combined", true | false)
    },
    #[serde(rename = "GET_PROPERTY")]
    GetProperty {
        id: String,
        params: String, // "combined"
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_stream_name() {
        let cases = vec![
            (
                StreamName::AggTrade {
                    symbol: String::from("btcusdt"),
                },
                r#""btcusdt@aggTrade""#,
            ),
            (
                StreamName::Trade {
                    symbol: String::from("btcusdt"),
                },
                r#""btcusdt@trade""#,
            ),
            (
                StreamName::Depth {
                    symbol: String::from("btcusdt"),
                },
                r#""btcusdt@depth""#,
            ),
            (
                StreamName::Kline {
                    symbol: String::from("btcusdt"),
                    interval: KlineInterval::Minute1,
                },
                r#""btcusdt@kline_1m""#,
            ),
            (
                StreamName::MiniTicker24 {
                    symbol: String::from("btcusdt"),
                },
                r#""btcusdt@24hrMiniTicker""#,
            ),
        ];

        cases.into_iter().for_each(|(stream, expected)| {
            let serialized = serde_json::to_string(&stream).unwrap();
            assert_eq!(expected, serialized);
        });
    }

    #[test]
    fn test_deserialize_stream_name() {
        let cases = vec![
            (
                r#""btcusdt@aggTrade""#,
                StreamName::AggTrade {
                    symbol: String::from("btcusdt"),
                },
            ),
            (
                r#""btcusdt@trade""#,
                StreamName::Trade {
                    symbol: String::from("btcusdt"),
                },
            ),
            (
                r#""btcusdt@depth""#,
                StreamName::Depth {
                    symbol: String::from("btcusdt"),
                },
            ),
            (
                r#""btcusdt@kline_1m""#,
                StreamName::Kline {
                    symbol: String::from("btcusdt"),
                    interval: KlineInterval::Minute1,
                },
            ),
            (
                r#""btcusdt@24hrMiniTicker""#,
                StreamName::MiniTicker24 {
                    symbol: String::from("btcusdt"),
                },
            ),
        ];

        cases.into_iter().for_each(|(serialized, expected)| {
            let stream = serde_json::from_str(serialized).unwrap();
            assert_eq!(expected, stream);
        });
    }
}
