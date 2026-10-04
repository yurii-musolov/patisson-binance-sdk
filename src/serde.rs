use serde::{Deserialize, Serialize};

pub fn deserialize_json<'de, T>(
    json: &'de str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>>
where
    T: Deserialize<'de>,
{
    let deserializer = &mut serde_json::Deserializer::from_str(json);

    serde_path_to_error::deserialize(deserializer).inspect_err(|err| {
        #[cfg(debug_assertions)]
        tracing::debug!(?err, ?json);
    })
}

#[inline]
pub fn serialize_json<T>(msg: &T) -> serde_json::Result<String>
where
    T: ?Sized + Serialize,
{
    serde_json::to_string(msg)
}

#[inline]
pub fn serialize_query<T>(msg: &T) -> Result<String, serde_urlencoded::ser::Error>
where
    T: ?Sized + Serialize,
{
    serde_urlencoded::to_string(msg)
}

/// Deserializer for use with `#[serde(deserialize_with = ...)]` on
/// [`Decimal`](rust_decimal::Decimal) fields Binance sends as JSON numbers
/// (e.g. `"makerCommission": 15`) rather than strings. A fractional number is
/// converted through its shortest round-trip decimal form, so `0.001` becomes
/// exactly `0.001`. Strings are accepted too.
pub fn decimal_from_number<'de, D>(deserializer: D) -> Result<rust_decimal::Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use rust_decimal::Decimal;
    use serde::de::{Error, Visitor};
    use std::{fmt, str::FromStr};

    struct DecimalVisitor;

    impl Visitor<'_> for DecimalVisitor {
        type Value = Decimal;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a decimal number or numeric string")
        }

        fn visit_i64<E: Error>(self, v: i64) -> Result<Decimal, E> {
            Ok(Decimal::from(v))
        }

        fn visit_u64<E: Error>(self, v: u64) -> Result<Decimal, E> {
            Ok(Decimal::from(v))
        }

        fn visit_f64<E: Error>(self, v: f64) -> Result<Decimal, E> {
            Decimal::from_str(&v.to_string()).map_err(E::custom)
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Decimal, E> {
            Decimal::from_str(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_any(DecimalVisitor)
}

/// Serializer for use with `#[serde(serialize_with = ...)]` on fields that
/// Binance expects as a JSON-array literal in a URL query — e.g.
/// `symbols=["BTC","ETH"]` rather than the repeated `symbols=BTC&symbols=ETH`
/// that `serde_urlencoded` would emit for `Vec<T>`.
pub fn serialize_option_as_json<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    match value {
        Some(v) => {
            let json = serde_json::to_string(v).map_err(serde::ser::Error::custom)?;
            serializer.serialize_some(&json)
        }
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::spot::ws::*;

    use super::*;

    #[test]
    fn test_deserialize_filter_variants() {
        use crate::spot::http::Filter;
        let json = r#"[
            {"filterType":"PRICE_FILTER","minPrice":"0.01","maxPrice":"1000000.00","tickSize":"0.01"},
            {"filterType":"LOT_SIZE","minQty":"0.00001","maxQty":"9000.0","stepSize":"0.00001"},
            {"filterType":"MAX_NUM_ORDERS","maxNumOrders":200},
            {"filterType":"TOTALLY_NEW_FILTER","foo":"bar"}
        ]"#;
        let filters: Vec<Filter> = deserialize_json(json).unwrap();
        assert!(matches!(filters[0], Filter::PriceFilter { .. }));
        assert!(matches!(filters[1], Filter::LotSize { .. }));
        assert!(matches!(filters[2], Filter::MaxNumOrders { .. }));
        assert!(matches!(filters[3], Filter::Unknown));
    }

    #[test]
    fn test_deserialize_exchange_filter_variants() {
        use crate::spot::ExchangeFilter;
        let json = r#"[
            {"filterType":"EXCHANGE_MAX_NUM_ORDERS","maxNumOrders":1000},
            {"filterType":"EXCHANGE_MAX_NUM_ALGO_ORDERS","maxNumAlgoOrders":200},
            {"filterType":"EXCHANGE_MAX_NUM_ICEBERG_ORDERS","maxNumIcebergOrders":10000},
            {"filterType":"EXCHANGE_SOMETHING_NEW","foo":"bar"}
        ]"#;
        let filters: Vec<ExchangeFilter> = deserialize_json(json).unwrap();
        assert_eq!(
            filters,
            vec![
                ExchangeFilter::ExchangeMaxNumOrders {
                    max_num_orders: 1000
                },
                ExchangeFilter::ExchangeMaxNumAlgoOrders {
                    max_num_algo_orders: 200
                },
                ExchangeFilter::ExchangeMaxNumIcebergOrders {
                    max_num_iceberg_orders: 10000
                },
                ExchangeFilter::Unknown,
            ]
        );
    }

    #[test]
    fn test_decimal_from_number() {
        #[derive(Deserialize)]
        struct D {
            #[serde(deserialize_with = "decimal_from_number")]
            v: rust_decimal::Decimal,
        }
        let parse = |json: &str| deserialize_json::<D>(json).unwrap().v;
        assert_eq!(parse(r#"{"v":15}"#), dec!(15));
        assert_eq!(parse(r#"{"v":-3}"#), dec!(-3));
        assert_eq!(parse(r#"{"v":0.001}"#), dec!(0.001));
        assert_eq!(parse(r#"{"v":"7.25"}"#), dec!(7.25));
        assert!(deserialize_json::<D>(r#"{"v":true}"#).is_err());
    }

    #[test]
    fn test_unknown_enum_values_do_not_break_deserialization() {
        use crate::spot::{OrderStatus, STPMode, SymbolStatus};
        let statuses: Vec<OrderStatus> =
            deserialize_json(r#"["FILLED","SOME_FUTURE_STATUS"]"#).unwrap();
        assert_eq!(statuses, vec![OrderStatus::Filled, OrderStatus::Unknown]);

        let modes: Vec<STPMode> = deserialize_json(r#"["EXPIRE_MAKER","TRANSFER"]"#).unwrap();
        assert_eq!(modes, vec![STPMode::ExpireMaker, STPMode::Unknown]);

        let status: SymbolStatus = deserialize_json(r#""PRE_TRADING""#).unwrap();
        assert_eq!(status, SymbolStatus::Unknown);

        let order_type: crate::derivatives::usds_margined_futures::OrderType =
            deserialize_json(r#""NEW_ORDER_KIND""#).unwrap();
        assert_eq!(
            order_type,
            crate::derivatives::usds_margined_futures::OrderType::Unknown
        );
    }

    #[test]
    fn test_unknown_enum_value_is_never_serialized() {
        #[derive(Serialize)]
        struct Q {
            side: crate::spot::OrderSide,
        }
        let q = Q {
            side: crate::spot::OrderSide::Unknown,
        };
        assert!(serialize_query(&q).is_err());
        let q = Q {
            side: crate::spot::OrderSide::BUY,
        };
        assert_eq!(serialize_query(&q).unwrap(), "side=BUY");
    }

    #[test]
    fn test_serialize_option_as_json_for_query_arrays() {
        #[derive(Serialize)]
        struct Q {
            #[serde(serialize_with = "serialize_option_as_json")]
            symbols: Option<Vec<String>>,
        }
        let q = Q {
            symbols: Some(vec!["BTCUSDT".into(), "BNBBTC".into()]),
        };
        let encoded = serialize_query(&q).unwrap();
        // URL-encoded form of: symbols=["BTCUSDT","BNBBTC"]
        assert_eq!(encoded, "symbols=%5B%22BTCUSDT%22%2C%22BNBBTC%22%5D");

        let empty = Q { symbols: None };
        assert_eq!(serialize_query(&empty).unwrap(), "");
    }

    #[test]
    fn test_deserialize_incoming_message_combined_stream_event_trade() {
        let json = r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1751132780369,"s":"BTCUSDT","t":5052328858,"p":"107407.88000000","q":"0.00024000","T":1751132780368,"m":true,"M":true}}"#;
        let symbol = String::from("BTCUSDT");
        let event = TradeMsg {
            event_time: 1751132780369,
            symbol: String::from("BTCUSDT"),
            trade_id: 5052328858,
            price: dec!(107407.88000000),
            qty: dec!(0.00024000),
            trade_time: 1751132780368,
            is_buyer_maker: true,
        };
        let event = CombinedStreamMessage {
            stream: StreamName::Trade {
                symbol: symbol.to_lowercase(),
            },
            data: StreamMessage::Trade(event),
        };
        let expected = IncomingMessage::CombinedStream(event);

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_incoming_message_combined_stream_event_kline() {
        let json = r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1751132772018,"s":"BTCUSDT","k":{"t":1751132760000,"T":1751132819999,"s":"BTCUSDT","i":"1m","f":5052328412,"L":5052328605,"o":"107413.25000000","c":"107413.18000000","h":"107413.25000000","l":"107413.18000000","v":"0.61673000","n":194,"x":false,"q":"66244.96435620","V":"0.02328000","Q":"2500.57910880","B":"0"}}}"#;
        let symbol = String::from("BTCUSDT");
        let interval = crate::spot::KlineInterval::Minute1;
        let event = KlineMsg {
            event_time: 1751132772018,
            symbol: symbol.clone(),
            kline: Kline {
                start_time: 1751132760000,
                close_time: 1751132819999,
                symbol: symbol.clone(),
                interval: interval.clone(),
                first_trade_id: 5052328412,
                last_trade_id: 5052328605,
                open_price: dec!(107413.25000000),
                close_price: dec!(107413.18000000),
                high_price: dec!(107413.25000000),
                low_price: dec!(107413.18000000),
                base_asset_volume: dec!(0.61673000),
                trade_number: 194,
                is_closed: false,
                quote_asset_volume: dec!(66244.96435620),
                taker_buy_base_asset_volume: dec!(0.02328000),
                taker_buy_quote_asset_volume: dec!(2500.57910880),
            },
        };
        let event = CombinedStreamMessage {
            stream: StreamName::Kline {
                symbol: symbol.to_lowercase(),
                interval,
            },
            data: StreamMessage::Kline(event),
        };
        let expected = IncomingMessage::CombinedStream(event);

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }
}
