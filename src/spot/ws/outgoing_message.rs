use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{serde::deserialize_json, spot::KlineInterval};

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
    /// "<symbol>@depth"
    Depth { symbol: String },
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
            Self::Depth { symbol } => format!("{symbol}@depth"),
            Self::Kline { symbol, interval } => format!("{symbol}@kline_{interval}"),
            Self::MiniTicker24 { symbol } => format!("{symbol}@24hrMiniTicker"),
            Self::ServerShutdownRaw => format!("serverShutdown"),
            Self::ServerShutdownCombined => format!("!serverShutdown"),
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
            (r#""serverShutdown""#, StreamName::ServerShutdownRaw),
            (r#""!serverShutdown""#, StreamName::ServerShutdownCombined),
        ];

        cases.into_iter().for_each(|(serialized, expected)| {
            let stream = deserialize_json(serialized).unwrap();
            assert_eq!(expected, stream);
        });
    }
}
