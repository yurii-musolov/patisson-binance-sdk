//! User data stream event examples from the official USDⓈ-M and COIN-M
//! Futures documentation ("User Data Streams" event pages, read 2026-10-05),
//! parsed with the SDK types. Each example must deserialize into its own
//! variant (not `Unknown`) without silently ignoring any field.
//!
//! Fixes to the documentation examples, which are not valid JSON as
//! published: a full-width comma in the USD-M `ACCOUNT_UPDATE` positions
//! array, trailing commas in `TRADE_LITE` and
//! `CONDITIONAL_ORDER_TRIGGER_REJECT`, and single quotes plus a missing comma
//! in the COIN-M `listenKeyExpired` example.

use binance::derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm};
use rust_decimal::dec;
use serde::de::DeserializeOwned;

/// Parse `name` as the enum `T` and as the event struct `S` itself.
///
/// `serde_ignored` can't see through an internally tagged enum (serde
/// buffers the content), so the strict check runs on the struct, where only
/// the `e` tag may be left over.
fn parse_event<T: DeserializeOwned, S: DeserializeOwned>(dir: &str, name: &str) -> T {
    let _: S = parse_ignoring(dir, name, &["e"]);
    parse(dir, name)
}

fn parse<T: DeserializeOwned>(dir: &str, name: &str) -> T {
    parse_ignoring(dir, name, &[])
}

fn parse_ignoring<T: DeserializeOwned>(dir: &str, name: &str, allowed: &[&str]) -> T {
    let path = format!("{}/tests/fixtures/{dir}/{name}", env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut ignored = Vec::new();
    let mut track = |path: serde_ignored::Path| ignored.push(path.to_string());
    let de = &mut serde_json::Deserializer::from_str(&json);
    let value =
        serde_ignored::deserialize(de, &mut track).unwrap_or_else(|e| panic!("{name}: {e}"));
    ignored.retain(|path| !allowed.contains(&path.as_str()));
    assert!(
        ignored.is_empty(),
        "{name}: fields not modelled: {ignored:?}"
    );
    value
}

mod usd_m {
    use super::*;
    use usdm::{
        MarginType, OrderStatus, OrderType, PositionSide, ws,
        ws::{AccountUpdateReason, ExecutionType, UserDataMessage as M},
    };

    fn event<S: DeserializeOwned>(name: &str) -> M {
        parse_event::<M, S>("usdm_user_data", name)
    }

    #[test]
    fn order_trade_update() {
        let M::OrderTradeUpdate(e) = event::<ws::OrderTradeUpdateEvent>("order_trade_update.json")
        else {
            panic!()
        };
        assert_eq!(e.order.order_type, OrderType::TrailingStopMarket);
        assert_eq!(e.order.execution_type, ExecutionType::New);
        assert_eq!(e.order.order_status, OrderStatus::New);
        assert_eq!(e.order.order_id, 8886774);
        assert_eq!(e.order.activation_price, Some(dec!(7476.89)));
        assert_eq!(e.order.expiry_reason.as_deref(), Some("0"));
    }

    #[test]
    fn account_update() {
        let M::AccountUpdate(e) = event::<ws::AccountUpdateEvent>("account_update.json") else {
            panic!()
        };
        assert_eq!(e.update.reason, AccountUpdateReason::Order);
        assert_eq!(e.update.balances.len(), 2);
        assert_eq!(e.update.positions.len(), 3);
        assert_eq!(e.update.positions[2].position_side, PositionSide::Short);
        assert_eq!(e.update.positions[2].margin_type, MarginType::Isolated);
        assert_eq!(e.update.positions[2].position_amount, dec!(-10));
    }

    #[test]
    fn other_events() {
        let M::MarginCall(e) = event::<ws::MarginCallEvent>("margin_call.json") else {
            panic!()
        };
        assert_eq!(e.positions[0].margin_type, MarginType::Crossed);
        let M::TradeLite(e) = event::<ws::TradeLiteEvent>("trade_lite.json") else {
            panic!()
        };
        assert_eq!(e.last_filled_quantity, dec!(0.040));
        let M::AccountConfigUpdate(e) =
            event::<ws::AccountConfigUpdateEvent>("account_config_update_leverage.json")
        else {
            panic!()
        };
        assert_eq!(e.leverage.unwrap().leverage, 25);
        let M::AccountConfigUpdate(e) =
            event::<ws::AccountConfigUpdateEvent>("account_config_update_multi_assets.json")
        else {
            panic!()
        };
        assert!(e.multi_assets.unwrap().multi_assets_mode);
        let M::AlgoUpdate(e) = event::<ws::AlgoUpdateEvent>("algo_update.json") else {
            panic!()
        };
        assert_eq!(e.order.algo_id, 2148719);
        assert_eq!(e.order.actual_order_id, None, "empty string means not set");
        assert!(matches!(
            event::<ws::ConditionalOrderTriggerRejectEvent>(
                "conditional_order_trigger_reject.json"
            ),
            M::ConditionalOrderTriggerReject(_)
        ));
        assert!(matches!(
            event::<ws::StrategyUpdateEvent>("strategy_update.json"),
            M::StrategyUpdate(_)
        ));
        assert!(matches!(
            event::<ws::GridUpdateEvent>("grid_update.json"),
            M::GridUpdate(_)
        ));
        assert!(matches!(
            event::<ws::ListenKeyExpiredEvent>("listen_key_expired.json"),
            M::ListenKeyExpired(_)
        ));
    }
}

mod coin_m {
    use super::*;
    use coinm::{MarginType, ws, ws::UserDataMessage as M};

    fn event<S: DeserializeOwned>(name: &str) -> M {
        parse_event::<M, S>("coinm_user_data", name)
    }

    #[test]
    fn documented_events() {
        let M::OrderTradeUpdate(e) = event::<ws::OrderTradeUpdateEvent>("order_trade_update.json")
        else {
            panic!()
        };
        assert_eq!(e.account_alias, "SfsR");
        assert_eq!(e.order.margin_asset, "BTC");
        let M::AccountUpdate(e) = event::<ws::AccountUpdateEvent>("account_update.json") else {
            panic!()
        };
        assert_eq!(e.update.positions[0].margin_type, MarginType::Isolated);
        let M::MarginCall(e) = event::<ws::MarginCallEvent>("margin_call.json") else {
            panic!()
        };
        assert_eq!(e.account_alias, "SfsR");
        assert!(matches!(
            event::<ws::AccountConfigUpdateEvent>("account_config_update.json"),
            M::AccountConfigUpdate(_)
        ));
        assert!(matches!(
            event::<ws::StrategyUpdateEvent>("strategy_update.json"),
            M::StrategyUpdate(_)
        ));
        assert!(matches!(
            event::<ws::GridUpdateEvent>("grid_update.json"),
            M::GridUpdate(_)
        ));
        assert!(matches!(
            event::<ws::ListenKeyExpiredEvent>("listen_key_expired.json"),
            M::ListenKeyExpired(_)
        ));
    }
}
