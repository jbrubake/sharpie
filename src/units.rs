use serde::{Deserialize, Serialize};
use std::fmt;

// Units {{{1
#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum Units {
    #[default]
    Imperial,
    Metric,
}

impl From<String> for Units { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for Units {
    fn from(index: &str) -> Self {
        match index {
            "1"     => Self::Metric,
            "0" | _ => Self::Imperial,
        }
    }
}

impl fmt::Display for Units { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::Imperial => "imperial",
                Self::Metric   => "metric",
            }
        )
    }
}

pub enum UnitType { // {{{1
    LengthSmall,
    LengthLong,
    Area,
    Weight,
    Power,
    WeightPerArea,
}

// Conversion constants {{{2
const INCH2MM: f64         = 25.4;
const FEET2METERS: f64     = 0.3048;
const SQFEET2SQMETERS: f64 = 0.092903;
const POUND2KG: f64        = 0.45359236;
const HP2KW: f64           = 0.746;

// Functions {{{2
//
pub fn metric(imperial: f64, unit_type: UnitType, units: Units) -> f64 { // {{{3
    if units == Units::Metric { return imperial; }

    match unit_type {
        UnitType::LengthSmall => imperial * INCH2MM,
        UnitType::LengthLong => imperial * FEET2METERS,
        UnitType::Area => imperial * SQFEET2SQMETERS,
        UnitType::Weight => imperial * POUND2KG,
        UnitType::Power => imperial * HP2KW,
        UnitType::WeightPerArea => imperial / SQFEET2SQMETERS * POUND2KG,
    }
}

// Testing {{{1
//
#[cfg(test)]
mod units {
    use super::*;
    use crate::test_support::*;

    // Test metric {{{3
    macro_rules! test_metric {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, imperial, unit_type, units) = $value;

                    assert_eq!(expected, to_place(metric(imperial, unit_type, units), 6));
                }
            )*
        }
    }

    test_metric! {
        // name:                  (metric, imperial, unit_type, units)
        metric_len_small:        (25.4, 1.0, UnitType::LengthSmall, Units::Imperial),
        metric_len_long:         (0.3048, 1.0, UnitType::LengthLong, Units::Imperial),
        metric_area:             (0.092903, 1.0, UnitType::Area, Units::Imperial),
        metric_weight:           (0.453592, 1.0, UnitType::Weight, Units::Imperial),
        metric_power:            (0.746, 1.0, UnitType::Power, Units::Imperial),
        metric_wgt_per_area:     (4.88243, 1.0, UnitType::WeightPerArea, Units::Imperial),
        metric_len_small_metric: (1.0, 1.0, UnitType::LengthSmall, Units::Metric),
        metric_len_long_metric:  (1.0, 1.0, UnitType::LengthLong, Units::Metric),
        metric_area_metric:      (1.0, 1.0, UnitType::Area, Units::Metric),
        metric_weight_metric:    (1.0, 1.0, UnitType::Weight, Units::Metric),
        metric_power_metric:     (1.0, 1.0, UnitType::Power, Units::Metric),
        metric_wgt_area_metric:  (1.0, 1.0, UnitType::WeightPerArea, Units::Metric),
    }

    // Test from {{{3
    macro_rules! test_from {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, index) = $value;

                    assert_eq!(expected, Units::from(index));
                }
            )*
        }
    }

    test_from! {
        // name:       (units, index)
        from_0:        (Units::Imperial, "0"),
        from_1:        (Units::Metric, "1"),
        from_invalid:  (Units::Imperial, "2"),
        from_garbage:  (Units::Imperial, "garbage"),
        from_empty:    (Units::Imperial, ""),
    }

    // Test display {{{3
    macro_rules! test_display {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, units) = $value;

                    assert_eq!(expected, format!("{}", units));
                }
            )*
        }
    }

    test_display! {
        // name:            (display, units)
        display_imperial:  ("imperial", Units::Imperial),
        display_metric:    ("metric", Units::Metric),
    }

    // Test default {{{3
    #[test]
    fn default_imperial() {
        assert_eq!(Units::Imperial, Units::default());
    }
}
