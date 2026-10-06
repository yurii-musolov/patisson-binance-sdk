//! User data stream examples from the official Spot API documentation
//! (`binance/binance-spot-api-docs`: `user-data-stream.md` and
//! `web-socket-api.md`, commit 828ca74), parsed with the SDK types. Each
//! example must deserialize without error and without silently ignoring any
//! field.

use binance::{
    spot::{
        ExecutionType, OrderStatus,
        ws_api::{
            BalanceUpdate, EventStreamTerminated, ExecutionReport, ExternalLockUpdate,
            IncomingMessage, ListStatus, OutboundAccountPosition, UserDataEvent,
        },
    },
    ws::ReceivedMessage,
};

fn parse(name: &str) -> IncomingMessage {
    let path = format!(
        "{}/tests/fixtures/spot_user_data/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut ignored = Vec::new();
    let mut track = |path: serde_ignored::Path| ignored.push(path.to_string());
    let de = &mut serde_json::Deserializer::from_str(&json);
    let value =
        serde_ignored::deserialize(de, &mut track).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert!(
        ignored.is_empty(),
        "{name}: fields not modelled: {ignored:?}"
    );
    value
}

/// `serde_ignored` can't see through the `e`-tagged `UserDataEvent` (serde
/// buffers its content), so the strict check also parses the event struct
/// `S` on its own; only the `e` tag may be left over.
fn strict_event<S: serde::de::DeserializeOwned>(name: &str) -> UserDataEvent {
    let path = format!(
        "{}/tests/fixtures/spot_user_data/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let inner = json["event"].to_string();
    let mut ignored = Vec::new();
    let mut track = |path: serde_ignored::Path| ignored.push(path.to_string());
    let de = &mut serde_json::Deserializer::from_str(&inner);
    let _: S = serde_ignored::deserialize(de, &mut track).unwrap_or_else(|e| panic!("{name}: {e}"));
    ignored.retain(|p| p != "e");
    assert!(
        ignored.is_empty(),
        "{name}: fields not modelled: {ignored:?}"
    );
    event(name)
}

fn event(name: &str) -> UserDataEvent {
    match parse(name) {
        IncomingMessage::Event(message) => {
            assert_eq!(message.subscription_id, Some(0), "{name}");
            message.event
        }
        other => panic!("{name}: not an event: {other:?}"),
    }
}

#[test]
fn every_documented_event_is_modelled() {
    assert!(matches!(
        strict_event::<OutboundAccountPosition>("event_outboundAccountPosition.json"),
        UserDataEvent::OutboundAccountPosition(_)
    ));
    assert!(matches!(
        strict_event::<BalanceUpdate>("event_balanceUpdate.json"),
        UserDataEvent::BalanceUpdate(_)
    ));
    assert!(matches!(
        strict_event::<ListStatus>("event_listStatus.json"),
        UserDataEvent::ListStatus(_)
    ));
    assert!(matches!(
        strict_event::<EventStreamTerminated>("event_eventStreamTerminated.json"),
        UserDataEvent::EventStreamTerminated(_)
    ));
    assert!(matches!(
        strict_event::<ExternalLockUpdate>("event_externalLockUpdate.json"),
        UserDataEvent::ExternalLockUpdate(_)
    ));
}

#[test]
fn execution_report_fields() {
    match strict_event::<ExecutionReport>("event_executionReport.json") {
        UserDataEvent::ExecutionReport(report) => {
            assert_eq!(report.execution_type, ExecutionType::New);
            assert_eq!(report.order_status, OrderStatus::New);
            assert_eq!(report.order_list_id, -1);
            assert_eq!(report.commission_asset, None);
            assert_eq!(report.prevented_match_id, Some(3));
            assert_eq!(report.working_time, Some(1499405658657));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn subscription_response_and_errors() {
    match parse("subscribe_signature_response.json") {
        IncomingMessage::Response(response) => {
            assert_eq!(response.status, 200);
            assert_eq!(response.subscription_id(), Some(0));
        }
        other => panic!("unexpected {other:?}"),
    }
    match parse("error_response.json") {
        IncomingMessage::Response(response) => {
            assert_eq!(response.status, 400);
            assert_eq!(response.error.unwrap().code.raw(), -2010);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn server_shutdown_triggers_a_reconnect() {
    let message = parse("server_shutdown.json");
    assert_eq!(message.server_shutdown_event_time(), Some(1770123456789));
}

#[test]
fn unknown_events_do_not_break_parsing() {
    let message: IncomingMessage =
        serde_json::from_str(r#"{"subscriptionId":1,"event":{"e":"somethingNew","E":1}}"#).unwrap();
    assert!(matches!(
        message,
        IncomingMessage::Event(e) if e.event == UserDataEvent::Unknown
    ));
}
