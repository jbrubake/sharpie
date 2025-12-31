use serde::{Serialize, Deserialize};
use std::fmt;

// Units {{{1
#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum Units {
    #[default]
    Imperial,
    Metric
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

