use crate::{serde::deserialize_json, spot::KlineInterval};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// The MessageID is used as an identifier to uniquely identify the messages going back and forth. The following formats are accepted:
///     64-bit signed integer
///     alphanumeric strings; max length 36
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

#[derive(Debug, Clone, PartialEq)]
pub enum StreamName {
    /// "<symbol>@aggTrade"
    AggTrade { symbol: String },
    /// "<symbol>@trade"
    Trade { symbol: String },
    /// Top <levels> bids and asks, pushed every second. Valid <levels> are 5, 10, or 20.
    /// Stream Names: <symbol>@depth<levels> OR <symbol>@depth<levels>@100ms
    PartialBookDepth {
        symbol: String,
        levels: PartialBookDepthLevels,
        update_speed: Option<PartialBookDepthUpdateSpeed>,
    },
    /// Order book price and quantity depth updates used to locally manage an order book.
    /// Stream Names: <symbol>@depth OR <symbol>@depth@100ms
    DiffDepth {
        symbol: String,
        update_speed: Option<DiffDepthUpdateSpeed>,
    },
    /// "<symbol>@kline_<interval>"
    Kline {
        symbol: String,
        interval: KlineInterval,
    },
    /// "<symbol>@24hrMiniTicker"
    MiniTicker24 { symbol: String },
    /// "serverShutdown"
    ServerShutdownRaw,
    /// "!serverShutdown"
    ServerShutdownCombined,
}

impl Serialize for StreamName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::AggTrade { symbol } => format!("{symbol}@aggTrade"),
            Self::Trade { symbol } => format!("{symbol}@trade"),
            Self::PartialBookDepth {
                symbol,
                levels,
                update_speed,
            } => update_speed.as_ref().map_or_else(
                || format!("{symbol}@depth{levels}"),
                |speed| format!("{symbol}@depth{levels}@{speed}"),
            ),
            Self::DiffDepth {
                symbol,
                update_speed,
            } => update_speed.as_ref().map_or_else(
                || format!("{symbol}@depth"),
                |speed| format!("{symbol}@depth@{speed}"),
            ),
            Self::Kline { symbol, interval } => format!("{symbol}@kline_{interval}"),
            Self::MiniTicker24 { symbol } => format!("{symbol}@24hrMiniTicker"),
            Self::ServerShutdownRaw => String::from("serverShutdown"),
            Self::ServerShutdownCombined => String::from("!serverShutdown"),
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
                "24hrMiniTicker" => Ok(Self::MiniTicker24 {
                    symbol: symbol.to_owned(),
                }),
                kind => {
                    if let Some(params) = kind.strip_prefix("depth") {
                        let symbol = symbol.to_string();
                        return if let Some((levels, update_speed)) = params.split_once('@') {
                            if !levels.is_empty() {
                                // PartialBookDepth: <symbol>@depth<levels>@100ms
                                let levels = format!("\"{levels}\"");
                                let levels = deserialize_json(&levels).map_err(|_| {
                                    serde::de::Error::custom("invalid stream format")
                                })?;

                                let update_speed = format!("\"{update_speed}\"");
                                let update_speed =
                                    deserialize_json(&update_speed).map_err(|_| {
                                        serde::de::Error::custom("invalid stream format")
                                    })?;
                                let stream = Self::PartialBookDepth {
                                    symbol,
                                    levels,
                                    update_speed: Some(update_speed),
                                };
                                Ok(stream)
                            } else {
                                // DiffDepth: <symbol>@depth@100ms
                                let update_speed = format!("\"{update_speed}\"");
                                let update_speed =
                                    deserialize_json(&update_speed).map_err(|_| {
                                        serde::de::Error::custom("invalid stream format")
                                    })?;
                                let stream = Self::DiffDepth {
                                    symbol,
                                    update_speed: Some(update_speed),
                                };
                                Ok(stream)
                            }
                        } else {
                            let levels = params;
                            if !levels.is_empty() {
                                // PartialBookDepth: <symbol>@depth<levels>
                                let levels = format!("\"{levels}\"");
                                let levels = deserialize_json(&levels).map_err(|_| {
                                    serde::de::Error::custom("invalid stream format")
                                })?;
                                let stream = Self::PartialBookDepth {
                                    symbol,
                                    levels,
                                    update_speed: None,
                                };
                                Ok(stream)
                            } else {
                                // DiffDepth: <symbol>@depth
                                let stream = Self::DiffDepth {
                                    symbol,
                                    update_speed: None,
                                };
                                Ok(stream)
                            }
                        };
                    }

                    if let Some((kind, params)) = kind.split_once('_') {
                        match kind {
                            "kline" => {
                                let interval = format!("\"{params}\"");
                                let interval = match deserialize_json(&interval) {
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
            match s {
                "serverShutdown" => Ok(Self::ServerShutdownRaw),
                "!serverShutdown" => Ok(Self::ServerShutdownCombined),
                _ => Err(serde::de::Error::custom("invalid stream format")),
            }
        }
    }
}

/// Top <levels> bids and asks, pushed every second. Valid <levels> are 5, 10, or 20.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PartialBookDepthLevels {
    #[serde(rename = "5")]
    Level5,
    #[serde(rename = "10")]
    Level10,
    #[serde(rename = "20")]
    Level20,
}

impl fmt::Display for PartialBookDepthLevels {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Self::Level5 => "5",
            Self::Level10 => "10",
            Self::Level20 => "20",
        };
        write!(f, "{value}")
    }
}

/// Update Speed: 1000ms or 100ms
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PartialBookDepthUpdateSpeed {
    #[serde(rename = "1000ms")]
    Speed1000ms,
    #[serde(rename = "100ms")]
    Speed100ms,
}

impl fmt::Display for PartialBookDepthUpdateSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Self::Speed1000ms => "1000ms",
            Self::Speed100ms => "100ms",
        };
        write!(f, "{value}")
    }
}

/// Update Speed: 1000ms or 100ms
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum DiffDepthUpdateSpeed {
    #[serde(rename = "1000ms")]
    Speed1000ms,
    #[serde(rename = "100ms")]
    Speed100ms,
}

impl fmt::Display for DiffDepthUpdateSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Self::Speed1000ms => "1000ms",
            Self::Speed100ms => "100ms",
        };
        write!(f, "{value}")
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "method")]
pub enum OutgoingMessage {
    Empty,
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
    ListSubscriptions {
        id: Option<MessageID>,
    },
    #[serde(rename = "SET_PROPERTY")]
    SetProperty {
        id: Option<MessageID>,
        params: (String, bool), // ("combined", true | false)
    },
    #[serde(rename = "GET_PROPERTY")]
    GetProperty {
        id: Option<MessageID>,
        params: String, // "combined"
    },
}

#[cfg(test)]
mod tests {
    use crate::serde::{deserialize_json, serialize_json};

    use super::*;

    #[test]
    fn test_message_id_serializes_as_bare_value() {
        assert_eq!(
            serialize_json(&MessageID::Str("req-0001".into())).unwrap(),
            r#""req-0001""#,
        );
        assert_eq!(serialize_json(&MessageID::Int(42)).unwrap(), r#"42"#);
        assert_eq!(
            deserialize_json::<MessageID>(r#""req-0001""#).unwrap(),
            MessageID::Str("req-0001".into()),
        );
        assert_eq!(
            deserialize_json::<MessageID>(r#"42"#).unwrap(),
            MessageID::Int(42),
        );
    }

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
            (
                StreamName::PartialBookDepth {
                    symbol: String::from("btcusdt"),
                    levels: PartialBookDepthLevels::Level20,
                    update_speed: None,
                },
                r#""btcusdt@depth20""#,
            ),
            (
                StreamName::PartialBookDepth {
                    symbol: String::from("btcusdt"),
                    levels: PartialBookDepthLevels::Level5,
                    update_speed: Some(PartialBookDepthUpdateSpeed::Speed100ms),
                },
                r#""btcusdt@depth5@100ms""#,
            ),
            (
                StreamName::DiffDepth {
                    symbol: String::from("btcusdt"),
                    update_speed: None,
                },
                r#""btcusdt@depth""#,
            ),
            (
                StreamName::DiffDepth {
                    symbol: String::from("btcusdt"),
                    update_speed: Some(DiffDepthUpdateSpeed::Speed100ms),
                },
                r#""btcusdt@depth@100ms""#,
            ),
            (StreamName::ServerShutdownRaw, r#""serverShutdown""#),
            (StreamName::ServerShutdownCombined, r#""!serverShutdown""#),
        ];

        cases.into_iter().for_each(|(stream, expected)| {
            let serialized = serialize_json(&stream).unwrap();
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
            (
                r#""btcusdt@depth20""#,
                StreamName::PartialBookDepth {
                    symbol: String::from("btcusdt"),
                    levels: PartialBookDepthLevels::Level20,
                    update_speed: None,
                },
            ),
            (
                r#""btcusdt@depth5@100ms""#,
                StreamName::PartialBookDepth {
                    symbol: String::from("btcusdt"),
                    levels: PartialBookDepthLevels::Level5,
                    update_speed: Some(PartialBookDepthUpdateSpeed::Speed100ms),
                },
            ),
            (
                r#""btcusdt@depth""#,
                StreamName::DiffDepth {
                    symbol: String::from("btcusdt"),
                    update_speed: None,
                },
            ),
            (
                r#""btcusdt@depth@100ms""#,
                StreamName::DiffDepth {
                    symbol: String::from("btcusdt"),
                    update_speed: Some(DiffDepthUpdateSpeed::Speed100ms),
                },
            ),
            (r#""serverShutdown""#, StreamName::ServerShutdownRaw),
            (r#""!serverShutdown""#, StreamName::ServerShutdownCombined),
        ];

        cases.into_iter().for_each(|(serialized, expected)| {
            let stream = deserialize_json(serialized).unwrap();
            assert_eq!(expected, stream);
        });
    }
}
