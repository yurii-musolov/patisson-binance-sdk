//! Trading rules of a symbol (`exchangeInfo` filters) in one place: round a
//! price or quantity to what Binance accepts and check an order before
//! sending it, instead of finding out from a `-1013 Filter failure`.
//!
//! Formulas follow the official filter documentation (`filters.md`): a value
//! of `0` disables a rule, `price % tickSize == 0`, `quantity % stepSize == 0`.
//!
//! ```
//! use binance::{Rounding, SymbolRules};
//! use rust_decimal::dec;
//!
//! let rules = SymbolRules {
//!     tick_size: Some(dec!(0.10)),
//!     step_size: Some(dec!(0.001)),
//!     min_qty: Some(dec!(0.001)),
//!     min_notional: Some(dec!(100)),
//!     ..SymbolRules::new("BTCUSDT")
//! };
//! let price = rules.round_price(dec!(65000.1234), Rounding::Down);
//! let qty = rules.round_qty(dec!(0.00157), Rounding::Down);
//! assert_eq!((price, qty), (dec!(65000.1), dec!(0.001)));
//! // 65000.1 * 0.001 = 65.0001 < 100
//! assert!(rules.validate(price, qty).is_err());
//! ```

use std::{collections::HashMap, fmt};

use rust_decimal::{Decimal, RoundingStrategy};

use crate::{
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    spot,
};

/// Direction to round a price or quantity to its tick / step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// Toward zero: never more than requested (safe for quantities).
    Down,
    /// Away from zero.
    Up,
    /// To the nearest multiple; halfway rounds away from zero.
    Nearest,
}

/// Trading rules of one symbol. `None` means the rule is absent or disabled.
///
/// Build it from `exchangeInfo` (`From<&SymbolInfo>`) or by hand with
/// `SymbolRules { tick_size: ..., ..SymbolRules::new("BTCUSDT") }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRules {
    pub symbol: String,
    /// `PRICE_FILTER`.
    pub tick_size: Option<Decimal>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    /// `LOT_SIZE` (limit orders).
    pub step_size: Option<Decimal>,
    pub min_qty: Option<Decimal>,
    pub max_qty: Option<Decimal>,
    /// `MARKET_LOT_SIZE`; when absent, market orders use `LOT_SIZE`.
    pub market_step_size: Option<Decimal>,
    pub market_min_qty: Option<Decimal>,
    pub market_max_qty: Option<Decimal>,
    /// Spot `NOTIONAL` / `MIN_NOTIONAL`, USD-M `MIN_NOTIONAL`. COIN-M has no
    /// notional filter (its quantity is in contracts).
    pub min_notional: Option<Decimal>,
    pub max_notional: Option<Decimal>,
    /// Whether `min_notional` / `max_notional` also apply to market orders.
    pub min_notional_applies_to_market: bool,
    pub max_notional_applies_to_market: bool,
}

/// Why an order would be rejected by a symbol filter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuleViolation {
    PriceBelowMin { price: Decimal, min: Decimal },
    PriceAboveMax { price: Decimal, max: Decimal },
    PriceNotOnTick { price: Decimal, tick: Decimal },
    QtyBelowMin { qty: Decimal, min: Decimal },
    QtyAboveMax { qty: Decimal, max: Decimal },
    QtyNotOnStep { qty: Decimal, step: Decimal },
    NotionalBelowMin { notional: Decimal, min: Decimal },
    NotionalAboveMax { notional: Decimal, max: Decimal },
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PriceBelowMin { price, min } => write!(f, "price {price} is below {min}"),
            Self::PriceAboveMax { price, max } => write!(f, "price {price} is above {max}"),
            Self::PriceNotOnTick { price, tick } => {
                write!(f, "price {price} is not a multiple of tick size {tick}")
            }
            Self::QtyBelowMin { qty, min } => write!(f, "quantity {qty} is below {min}"),
            Self::QtyAboveMax { qty, max } => write!(f, "quantity {qty} is above {max}"),
            Self::QtyNotOnStep { qty, step } => {
                write!(f, "quantity {qty} is not a multiple of step size {step}")
            }
            Self::NotionalBelowMin { notional, min } => {
                write!(f, "notional {notional} is below {min}")
            }
            Self::NotionalAboveMax { notional, max } => {
                write!(f, "notional {notional} is above {max}")
            }
        }
    }
}

impl std::error::Error for RuleViolation {}

impl SymbolRules {
    /// Rules with nothing set; fill the fields or build from `exchangeInfo`
    /// with `From<&SymbolInfo>`.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            tick_size: None,
            min_price: None,
            max_price: None,
            step_size: None,
            min_qty: None,
            max_qty: None,
            market_step_size: None,
            market_min_qty: None,
            market_max_qty: None,
            min_notional: None,
            max_notional: None,
            min_notional_applies_to_market: false,
            max_notional_applies_to_market: false,
        }
    }

    /// `price` rounded to a multiple of the tick size (unchanged without one).
    pub fn round_price(&self, price: Decimal, rounding: Rounding) -> Decimal {
        round_to(price, self.tick_size, rounding)
    }

    /// Limit order quantity rounded to a multiple of the `LOT_SIZE` step.
    pub fn round_qty(&self, qty: Decimal, rounding: Rounding) -> Decimal {
        round_to(qty, self.step_size, rounding)
    }

    /// Market order quantity rounded to the `MARKET_LOT_SIZE` step (or the
    /// `LOT_SIZE` step when there is none).
    pub fn round_market_qty(&self, qty: Decimal, rounding: Rounding) -> Decimal {
        round_to(qty, self.market_step_size.or(self.step_size), rounding)
    }

    /// Check a limit order: price, quantity and notional (`price * qty`).
    pub fn validate(&self, price: Decimal, qty: Decimal) -> Result<(), RuleViolation> {
        self.validate_price(price)?;
        check_qty(qty, self.step_size, self.min_qty, self.max_qty)?;
        self.check_notional(price * qty, true, true)
    }

    /// Check a market order. Notional limits that apply to market orders are
    /// checked only when `reference_price` (e.g. the average or mark price)
    /// is given.
    pub fn validate_market(
        &self,
        qty: Decimal,
        reference_price: Option<Decimal>,
    ) -> Result<(), RuleViolation> {
        check_qty(
            qty,
            self.market_step_size.or(self.step_size),
            self.market_min_qty.or(self.min_qty),
            self.market_max_qty.or(self.max_qty),
        )?;
        match reference_price {
            Some(price) => self.check_notional(
                price * qty,
                self.min_notional_applies_to_market,
                self.max_notional_applies_to_market,
            ),
            None => Ok(()),
        }
    }

    /// Check a price or stop price against `PRICE_FILTER`.
    pub fn validate_price(&self, price: Decimal) -> Result<(), RuleViolation> {
        if let Some(min) = self.min_price
            && price < min
        {
            return Err(RuleViolation::PriceBelowMin { price, min });
        }
        if let Some(max) = self.max_price
            && price > max
        {
            return Err(RuleViolation::PriceAboveMax { price, max });
        }
        if let Some(tick) = self.tick_size
            && !(price % tick).is_zero()
        {
            return Err(RuleViolation::PriceNotOnTick { price, tick });
        }
        Ok(())
    }

    fn check_notional(
        &self,
        notional: Decimal,
        check_min: bool,
        check_max: bool,
    ) -> Result<(), RuleViolation> {
        if check_min
            && let Some(min) = self.min_notional
            && notional < min
        {
            return Err(RuleViolation::NotionalBelowMin { notional, min });
        }
        if check_max
            && let Some(max) = self.max_notional
            && notional > max
        {
            return Err(RuleViolation::NotionalAboveMax { notional, max });
        }
        Ok(())
    }
}

fn check_qty(
    qty: Decimal,
    step: Option<Decimal>,
    min: Option<Decimal>,
    max: Option<Decimal>,
) -> Result<(), RuleViolation> {
    if let Some(min) = min
        && qty < min
    {
        return Err(RuleViolation::QtyBelowMin { qty, min });
    }
    if let Some(max) = max
        && qty > max
    {
        return Err(RuleViolation::QtyAboveMax { qty, max });
    }
    if let Some(step) = step
        && !(qty % step).is_zero()
    {
        return Err(RuleViolation::QtyNotOnStep { qty, step });
    }
    Ok(())
}

fn round_to(value: Decimal, step: Option<Decimal>, rounding: Rounding) -> Decimal {
    let Some(step) = step else { return value };
    let strategy = match rounding {
        Rounding::Down => RoundingStrategy::ToZero,
        Rounding::Up => RoundingStrategy::AwayFromZero,
        Rounding::Nearest => RoundingStrategy::MidpointAwayFromZero,
    };
    ((value / step).round_dp_with_strategy(0, strategy) * step).normalize()
}

/// `0` disables a filter value.
fn enabled(value: Decimal) -> Option<Decimal> {
    (!value.is_zero()).then_some(value)
}

impl From<&spot::http::SymbolInfo> for SymbolRules {
    fn from(info: &spot::http::SymbolInfo) -> Self {
        use spot::http::Filter;
        let mut rules = Self::new(&info.symbol);
        // NOTIONAL supersedes MIN_NOTIONAL when both are present.
        let mut has_notional = false;
        for filter in &info.filters {
            match filter {
                Filter::PriceFilter {
                    min_price,
                    max_price,
                    tick_size,
                } => {
                    rules.tick_size = enabled(*tick_size);
                    rules.min_price = enabled(*min_price);
                    rules.max_price = enabled(*max_price);
                }
                Filter::LotSize {
                    min_qty,
                    max_qty,
                    step_size,
                } => {
                    rules.step_size = enabled(*step_size);
                    rules.min_qty = enabled(*min_qty);
                    rules.max_qty = enabled(*max_qty);
                }
                Filter::MarketLotSize {
                    min_qty,
                    max_qty,
                    step_size,
                } => {
                    rules.market_step_size = enabled(*step_size);
                    rules.market_min_qty = enabled(*min_qty);
                    rules.market_max_qty = enabled(*max_qty);
                }
                Filter::Notional {
                    min_notional,
                    apply_min_to_market,
                    max_notional,
                    apply_max_to_market,
                    ..
                } => {
                    has_notional = true;
                    rules.min_notional = enabled(*min_notional);
                    rules.max_notional = enabled(*max_notional);
                    rules.min_notional_applies_to_market = *apply_min_to_market;
                    rules.max_notional_applies_to_market = *apply_max_to_market;
                }
                Filter::MinNotional {
                    min_notional,
                    apply_to_market,
                    ..
                } if !has_notional => {
                    rules.min_notional = enabled(*min_notional);
                    rules.min_notional_applies_to_market = *apply_to_market;
                }
                _ => {}
            }
        }
        rules
    }
}

impl From<&usdm::http::SymbolInfo> for SymbolRules {
    fn from(info: &usdm::http::SymbolInfo) -> Self {
        use usdm::http::SymbolFilter;
        let mut rules = Self::new(&info.symbol);
        for filter in &info.filters {
            match filter {
                SymbolFilter::PriceFilter(f) => {
                    rules.tick_size = enabled(f.tick_size);
                    rules.min_price = enabled(f.min_price);
                    rules.max_price = enabled(f.max_price);
                }
                SymbolFilter::LotSize(f) => {
                    rules.step_size = enabled(f.step_size);
                    rules.min_qty = enabled(f.min_qty);
                    rules.max_qty = enabled(f.max_qty);
                }
                SymbolFilter::MarketLotSize(f) => {
                    rules.market_step_size = enabled(f.step_size);
                    rules.market_min_qty = enabled(f.min_qty);
                    rules.market_max_qty = enabled(f.max_qty);
                }
                SymbolFilter::MinNotional(f) => {
                    // Futures check the notional of market orders too
                    // (against the mark price).
                    rules.min_notional = enabled(f.notional);
                    rules.min_notional_applies_to_market = true;
                }
                _ => {}
            }
        }
        rules
    }
}

impl From<&coinm::http::SymbolInfo> for SymbolRules {
    fn from(info: &coinm::http::SymbolInfo) -> Self {
        use coinm::http::SymbolFilter;
        let mut rules = Self::new(&info.symbol);
        for filter in &info.filters {
            match filter {
                SymbolFilter::PriceFilter(f) => {
                    rules.tick_size = enabled(f.tick_size);
                    rules.min_price = enabled(f.min_price);
                    rules.max_price = enabled(f.max_price);
                }
                SymbolFilter::LotSize(f) => {
                    rules.step_size = enabled(f.step_size);
                    rules.min_qty = enabled(f.min_qty);
                    rules.max_qty = enabled(f.max_qty);
                }
                SymbolFilter::MarketLotSize(f) => {
                    rules.market_step_size = enabled(f.step_size);
                    rules.market_min_qty = enabled(f.min_qty);
                    rules.market_max_qty = enabled(f.max_qty);
                }
                _ => {}
            }
        }
        rules
    }
}

/// Rules of every symbol of an `exchangeInfo` response, by symbol name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExchangeRules {
    symbols: HashMap<String, SymbolRules>,
}

impl ExchangeRules {
    pub fn get(&self, symbol: &str) -> Option<&SymbolRules> {
        self.symbols.get(symbol)
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SymbolRules> {
        self.symbols.values()
    }
}

impl FromIterator<SymbolRules> for ExchangeRules {
    fn from_iter<I: IntoIterator<Item = SymbolRules>>(iter: I) -> Self {
        Self {
            symbols: iter.into_iter().map(|r| (r.symbol.clone(), r)).collect(),
        }
    }
}

impl From<&spot::http::ExchangeInfo> for ExchangeRules {
    fn from(info: &spot::http::ExchangeInfo) -> Self {
        info.symbols.iter().map(SymbolRules::from).collect()
    }
}

impl From<&usdm::http::ExchangeInfo> for ExchangeRules {
    fn from(info: &usdm::http::ExchangeInfo) -> Self {
        info.symbols.iter().map(SymbolRules::from).collect()
    }
}

impl From<&coinm::http::ExchangeInfo> for ExchangeRules {
    fn from(info: &coinm::http::ExchangeInfo) -> Self {
        info.symbols.iter().map(SymbolRules::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;

    fn btcusdt() -> SymbolRules {
        SymbolRules {
            tick_size: Some(dec!(0.10)),
            min_price: Some(dec!(556.80)),
            max_price: Some(dec!(4529764)),
            step_size: Some(dec!(0.001)),
            min_qty: Some(dec!(0.001)),
            max_qty: Some(dec!(1000)),
            market_step_size: Some(dec!(0.01)),
            market_min_qty: Some(dec!(0.01)),
            market_max_qty: Some(dec!(120)),
            min_notional: Some(dec!(100)),
            min_notional_applies_to_market: true,
            ..SymbolRules::new("BTCUSDT")
        }
    }

    #[test]
    fn rounding_modes() {
        let r = btcusdt();
        assert_eq!(r.round_price(dec!(65000.15), Rounding::Down), dec!(65000.1));
        assert_eq!(r.round_price(dec!(65000.15), Rounding::Up), dec!(65000.2));
        assert_eq!(
            r.round_price(dec!(65000.15), Rounding::Nearest),
            dec!(65000.2)
        );
        assert_eq!(
            r.round_price(dec!(65000.14), Rounding::Nearest),
            dec!(65000.1)
        );
        assert_eq!(r.round_price(dec!(65000.1), Rounding::Up), dec!(65000.1));
        assert_eq!(r.round_qty(dec!(0.0019), Rounding::Down), dec!(0.001));
        assert_eq!(r.round_market_qty(dec!(0.019), Rounding::Down), dec!(0.01));
        // Without a step the value is unchanged.
        let none = SymbolRules::new("X");
        assert_eq!(none.round_qty(dec!(1.23456), Rounding::Down), dec!(1.23456));
    }

    #[test]
    fn rounded_values_pass_validation() {
        let r = btcusdt();
        let price = r.round_price(dec!(65432.987), Rounding::Down);
        let qty = r.round_qty(dec!(0.0123456), Rounding::Down);
        assert_eq!(r.validate(price, qty), Ok(()));
    }

    #[test]
    fn violations() {
        let r = btcusdt();
        use RuleViolation::*;
        assert!(matches!(
            r.validate(dec!(500), dec!(1)),
            Err(PriceBelowMin { .. })
        ));
        assert!(matches!(
            r.validate(dec!(5000000), dec!(1)),
            Err(PriceAboveMax { .. })
        ));
        assert!(matches!(
            r.validate(dec!(65000.05), dec!(1)),
            Err(PriceNotOnTick { .. })
        ));
        assert!(matches!(
            r.validate(dec!(65000), dec!(0.0005)),
            Err(QtyBelowMin { .. })
        ));
        assert!(matches!(
            r.validate(dec!(65000), dec!(1001)),
            Err(QtyAboveMax { .. })
        ));
        assert!(matches!(
            r.validate(dec!(65000), dec!(0.0015)),
            Err(QtyNotOnStep { .. })
        ));
        assert_eq!(
            r.validate(dec!(65000), dec!(0.001)),
            Err(NotionalBelowMin {
                notional: dec!(65.000),
                min: dec!(100)
            })
        );
        // Market orders use MARKET_LOT_SIZE; notional only with a price.
        assert!(matches!(
            r.validate_market(dec!(0.015), None),
            Err(QtyNotOnStep { .. })
        ));
        assert_eq!(r.validate_market(dec!(0.01), None), Ok(()));
        assert!(matches!(
            r.validate_market(dec!(0.01), Some(dec!(6500))),
            Err(NotionalBelowMin { .. })
        ));
    }
}
