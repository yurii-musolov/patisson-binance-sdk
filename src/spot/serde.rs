use serde::Deserialize;

pub fn deserialize_str<'de, T>(
    json: &'de str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>>
where
    T: Deserialize<'de>,
{
    let deserializer = &mut serde_json::Deserializer::from_str(json);

    serde_path_to_error::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::spot::*;

    use super::*;

    #[test]
    fn test_deserialize_incoming_message_combined_stream_event_trade() {
        let json = r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1751132780369,"s":"BTCUSDT","t":5052328858,"p":"107407.88000000","q":"0.00024000","T":1751132780368,"m":true,"M":true}}"#;
        let symbol = String::from("BTCUSDT");
        let event = EventTrade {
            event_time: 1751132780369,
            symbol: String::from("BTCUSDT"),
            trade_id: 5052328858,
            price: dec!(107407.88000000),
            qty: dec!(0.00024000),
            trade_time: 1751132780368,
            is_buyer: true,
        };
        let event = CombinedStreamEvent {
            stream: StreamName::Trade {
                symbol: symbol.to_lowercase(),
            },
            data: StreamEvent::Trade(event),
        };
        let expected = IncomingMessage::StreamEvent(event);

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn test_deserialize_incoming_message_combined_stream_event_kline() {
        let json = r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1751132772018,"s":"BTCUSDT","k":{"t":1751132760000,"T":1751132819999,"s":"BTCUSDT","i":"1m","f":5052328412,"L":5052328605,"o":"107413.25000000","c":"107413.18000000","h":"107413.25000000","l":"107413.18000000","v":"0.61673000","n":194,"x":false,"q":"66244.96435620","V":"0.02328000","Q":"2500.57910880","B":"0"}}}"#;
        let symbol = String::from("BTCUSDT");
        let interval = crate::spot::KlineInterval::Minute1;
        let event = EventKline {
            event_time: 1751132772018,
            symbol: symbol.clone(),
            kline: KlineMsg {
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
        let event = CombinedStreamEvent {
            stream: StreamName::Kline {
                symbol: symbol.to_lowercase(),
                interval,
            },
            data: StreamEvent::Kline(event),
        };
        let expected = IncomingMessage::StreamEvent(event);

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }
}
