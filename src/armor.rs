use crate::Hull;
use crate::units::Units;

use serde::{Serialize, Deserialize};

use std::fmt;

// Armor {{{1
/// The ship's armor, excluding gun armor.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Armor {
    /// Units
    pub units: Units,

    /// Main belt armor.
    pub main: Belt,
    /// End belt armor.
    pub end: Belt,
    /// Uppper belt armor.
    pub upper: Belt,
    /// Incline of belt armor.
    pub incline: f64,

    /// Torpedo bulge armor.
    pub bulge: Belt,
    /// Bulkhead armor.
    pub bulkhead: Belt,
    /// What it says on the tin.
    pub bh_kind: BulkheadType,
    /// Beam between outer and inner bulkheads.
    pub bh_beam: f64,

    /// Deck armor.
    pub deck: Deck,

    /// Forward conning tower armor.
    pub ct_fwd: CT,
    /// Aft conning tower armor.
    pub ct_aft: CT,
}

impl Default for Armor { // {{{2
    fn default() -> Self {
        Armor {
            units: Units::Imperial,

            main:     Belt::new(BeltType::Main),
            end:      Belt::new(BeltType::End),
            upper:    Belt::new(BeltType::Upper),
            bulge:    Belt::new(BeltType::Bulge),
            bulkhead: Belt::new(BeltType::Bulkhead),

            bh_kind: BulkheadType::Additional,
            incline: 0.0,
            bh_beam: 0.0,

            deck: Deck::default(),

            ct_fwd: CT::default(),
            ct_aft: CT::default(),
        }
    }
}

impl Armor { // {{{2
    // XXX: I do not know what this does.
    pub const INCH: f64 = 0.0185; 

    // wgt {{{3
    /// Total weight of armor.
    ///
    pub fn wgt(&self, hull: Hull, wgt_mag: f64, wgt_engine: f64) -> f64 {
        let lwl = hull.lwl();
        let cwp = hull.cwp();
        let b   = hull.b;
        let d   = hull.d();

        self.main    .wgt(lwl, cwp, b) +
        self.end     .wgt(lwl, cwp, b) +
        self.upper   .wgt(lwl, cwp, b) +
        self.bulge   .wgt(lwl, cwp, b) +
        self.bulkhead.wgt(lwl, cwp, b) +

        self.deck    .wgt(hull.clone(), wgt_mag, wgt_engine) +

        self.ct_fwd  .wgt(d) +
        self.ct_aft  .wgt(d)
    }

    // belt_coverage {{{3
    /// Percentage of the "vital areas" covered by the main belt.
    ///
    pub fn belt_coverage(&self, lwl: f64) -> f64 {
        self.main.len / (lwl * 0.65)
    }

    // max_hgt {{{3
    /// Maximum allowable belt height.
    ///
    pub fn max_belt_hgt(&self, t: f64, dist: f64) -> f64 {
        use std::f64::consts::PI;

        let radians = self.incline * PI / 180.0;

        (t + dist) * (1.0 / radians.abs().cos()) + 0.02
    }
}

// Testing Armor {{{2
#[cfg(test)]
mod armor {
    use super::*;
    use crate::test_support::*;
    use crate::Hull;

    // Test belt_coverage {{{3
    macro_rules! test_belt_coverage {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, belt_len, lwl) = $value;

                    let mut armor = Armor::default();
                    armor.main.len = belt_len;

                    assert!(expected == to_place(armor.belt_coverage(lwl), 2));
                }
            )*
        }
    }
    test_belt_coverage! {
        // name:         (belt_coverage, belt_len, lwl)
        belt_coverage_1: (1.0, 0.65, 1.0),
        belt_coverage_2: (1.54, 1.0, 1.0),
    }

    // Test max_hgt {{{3
    macro_rules! test_max_hgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, incline) = $value;

                    let t = 10.0;

                    let mut armor = Armor::default();
                    armor.incline = incline;

                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 10.0;

                    hull.ad_fwd = 10.0;
                    hull.ad_aft = 10.0;

                    hull.qd_len = 0.15;

                    assert!(expected == to_place(armor.max_belt_hgt(t, hull.freeboard_dist()), 2));
                }
            )*
        }
    }
    test_max_hgt! {
        // name:        (max_hgt, incline)
        max_belt_hgt_0: (20.02, 0.0),
        max_belt_hgt_45: (28.3, 45.0),
    }
}

// Belt {{{1
/// Belt, bulkhead and torpedo bulge armor.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Belt {
    /// Belt thickness.
    pub thick: f64,
    /// Belth length.
    pub len: f64,
    /// Belt height.
    pub hgt: f64,

    /// Type of belt.
    ///
    /// Using this private "set once" field allows Belt to represent the
    /// multiple types that differ only in how their weight is calculated.
        kind: BeltType, // kind should not be changed after creation
}

impl Belt { // {{{2
    // wgt {{{3
    /// Belt weight.
    ///
    pub fn wgt(&self, lwl: f64, cwp: f64, b: f64) -> f64 {
        let extra = match self.kind {
            BeltType::Main | BeltType::Upper =>
                (1.0 - self.len / lwl).powf(1.0 - cwp) * b,
            _ => 0.0
        };

        (self.len + extra) * self.hgt * self.thick * Armor::INCH * 2.0
    }

    // new {{{3
    /// Create a Belt of type "kind".
    ///
    pub fn new(kind: BeltType) -> Belt {
        Belt {
            thick: 0.0,
            len: 0.0,
            hgt: 0.0,
            kind,
        }
    }
}

// Testing Belt {{{2
#[cfg(test)]
mod belt {
    use super::*;
    use crate::test_support::*;

    // Test wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let lwl = 500.0;
                    let cwp = 0.5;
                    let b = 10.0;

                    let (expected, thick, len, hgt, kind) = $value;
                    let mut belt = Belt::new(kind);
                    belt.thick = thick; belt.len = len; belt.hgt = hgt;

                    assert!(expected == to_place(belt.wgt(lwl, cwp, b), 2));
                }
            )*
        }
    }
    test_wgt! {
        // name:      (wgt, thick, len, hgt, kind)
        wgt_zero:     (0.0, 0.0, 0.0, 0.0, BeltType::Main),
        wgt_main:     (40.31, 1.0, 100.0, 10.0, BeltType::Main),
        wgt_end:      (37.0, 1.0, 100.0, 10.0, BeltType::End),
        wgt_upper:    (40.31, 1.0, 100.0, 10.0, BeltType::Upper),
        wgt_bulge:    (37.0, 1.0, 100.0, 10.0, BeltType::Bulge),
        wgt_bulkhead: (37.0, 1.0, 100.0, 10.0, BeltType::Bulkhead),
    }
}

// BulkheadType {{{1
/// Values for Armor::bh_kind
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BulkheadType {
    Strengthened,
    Additional,
}

// BeltType {{{1
/// Values for Belt::kind
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BeltType {
    /// Main belt.
    Main,
    /// End belt.
    End,
    /// Upper belt.
    Upper,
    /// Torpedo bulges.
    Bulge,
    /// Bulkhead.
    Bulkhead,
}

// CT {{{1
/// Conning tower armor.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CT {
    /// Armor thickness.
    pub thick: f64,
}

impl CT { // {{{2
    // wgt {{{3
    /// Weight of armor.
    ///
    pub fn wgt(&self, d: f64) -> f64 {
        10.0 * (d / 10_000.0).powf(2.0/3.0) * self.thick
    }
}

// Testing CT {{{2
#[cfg(test)]
mod ct {
    use super::*;
    use crate::test_support::*;

    // Test wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 1000.0;
                    let (expected, thick) = $value;
                    let mut ct = CT::default();
                    ct.thick = thick;

                    assert!(expected == to_place(ct.wgt(d), 2));
                }
            )*
        }
    }
    test_wgt! {
        //  name: (wgt, thick)
        wgt_zero: (0.0, 0.0),
        wgt_test: (2.15, 1.0),
    }
}

// Deck {{{1
/// Deck armor.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Deck {
    /// Forecastle deck thickness.
    pub fc: f64,
    /// Main deck thickness.
    pub md: f64,
    /// Quarterdeck deck thickness.
    pub qd: f64,

    /// Deck armor configuration.
    pub kind: DeckType,
}

impl Deck { // {{{2
    // wgt {{{3
    /// Weight of deck armor.
    ///
    pub fn wgt(&self, hull: Hull, wgt_mag: f64, wgt_engine: f64) -> f64 {
        let d      = hull.d();
        let lwl    = hull.lwl();
        let b      = hull.b;
        let fc_len = hull.fc_len;
        let qd_len = hull.qd_len;
        let cwp    = hull.cwp();
        let wp     = hull.wp();

        let main_deck = self.kind.wgt_factor(
            d, lwl, b, fc_len, qd_len, wp, cwp, wgt_engine, wgt_mag
        );

        let fc_deck = (fc_len * 2.0).powf(1.0 - cwp.powf(2.0)) *
            b * lwl * fc_len * 0.5;

        let qd_deck = qd_len.powf(1.0 - cwp) * b * lwl * qd_len / 4.0 *
            (2.0 + 2.0_f64.powf(1.0 - cwp));

        (main_deck * self.md + fc_deck * self.fc + qd_deck * self.qd) * Armor::INCH
    }
}

// Testing Deck {{{2
#[cfg(test)]
mod deck {
    use super::*;
    use crate::test_support::*;
    use crate::Hull;
    use crate::hull::SternType;

    // Test wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, fc, md, qd) = $value;

                    let wgt_mag = 100.0;
                    let wgt_engine = 100.0;

                    let mut deck = Deck::default();
                    deck.kind = kind;
                    deck.fc = fc;
                    deck.md = md;
                    deck.qd = qd;

                    let mut hull = Hull::default();
                    hull.set_lwl(100.0);
                    hull.set_d(1000.0);
                    hull.set_shafts(2); // hull.boxy == false
                    hull.b = 50.0;
                    hull.bb = hull.b;
                    hull.t = 10.0;
                    hull.stern_type = SternType::Cruiser;

                    hull.fc_len = 0.2;
                    hull.fc_fwd = 10.0;
                    hull.fc_aft = 10.0;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = hull.fc_fwd;
                    hull.fd_aft = hull.fc_fwd;

                    hull.ad_fwd = hull.fc_fwd;
                    hull.ad_aft = hull.fc_fwd;

                    hull.qd_len = 0.15;
                    hull.qd_fwd = hull.fc_fwd;
                    hull.qd_aft = hull.fc_fwd;

                    assert!(expected == to_place(deck.wgt(hull, wgt_mag, wgt_engine), 2));
                }
            )*
        }
    }
    test_wgt! {
        //  name:             (wgt, deck, fc, md, qd)
        wgt_mult_arm_fc:      (6.67, DeckType::MultipleArmored, 1.0, 0.0, 0.0),
        wgt_mult_arm_md:      (60.58, DeckType::MultipleArmored, 0.0, 1.0, 0.0),
        wgt_mult_arm_qd:      (7.49, DeckType::MultipleArmored, 0.0, 0.0, 1.0),
        wgt_mult_arm:         (74.74, DeckType::MultipleArmored, 1.0, 1.0, 1.0),

        wgt_one_arm_fc:       (6.67, DeckType::SingleArmored, 1.0, 0.0, 0.0),
        wgt_mult_prot_fc:     (6.67, DeckType::MultipleProtected, 1.0, 0.0, 0.0),
        wgt_one_prot_fc:      (6.67, DeckType::SingleProtected, 1.0, 0.0, 0.0),
        wgt_box_machinery_md: (40.13, DeckType::BoxOverMachinery, 0.0, 1.0, 0.0),
        wgt_box_magazine_md:  (23.24, DeckType::BoxOverMagazine, 0.0, 1.0, 0.0),
        wgt_box_both_md:      (48.57, DeckType::BoxOverBoth, 0.0, 1.0, 0.0),
    }
}

// DeckType {{{1
/// Deck armor configuration types.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum DeckType {
    #[default]
    MultipleArmored,
    SingleArmored,
    MultipleProtected,
    SingleProtected,
    BoxOverMachinery,
    BoxOverMagazine,
    BoxOverBoth,
}

impl DeckType { // {{{2
    // wgt_factor {{{3
    /// Main deck weight factor for each deck type.
    ///
    pub fn wgt_factor(&self,
        d: f64, lwl: f64, b: f64, 
        fc_len: f64, qd_len:f64,
        wp: f64, cwp: f64,
        wgt_engine: f64, wgt_mag: f64) -> f64 {

        match self {
            Self::MultipleArmored |
            Self::SingleArmored |
            Self::MultipleProtected |
            Self::SingleProtected => {
                (
                    wp - (fc_len * 2.0).powf(1.0 - cwp.powf(2.0)) * b * lwl * fc_len / 2.0 -
                    (
                        qd_len.powf(1.0 - cwp) * b * lwl * qd_len * 0.25 +
                        (
                            qd_len.powf(1.0 - cwp) +
                            (qd_len * 2.0).powf(1.0 - cwp)
                        ) * b * lwl * qd_len * 0.25
                    )
                ) * 1.01
            },

            Self::BoxOverMachinery =>
                (wgt_engine * 3.0 / (d * 0.94) * 0.65 * lwl + 16.0) * (b + 16.0) - 256.0,

            Self::BoxOverMagazine =>
                (wgt_mag / (d * 0.94) * 0.65 * lwl + 16.0) * (b + 16.0) - 256.0,

            Self::BoxOverBoth =>
                ((wgt_engine * 3.0 + wgt_mag) / (d * 0.94) * 0.65 * lwl + 16.0) * (b + 16.0) - 256.0,

        }
    }
}

impl fmt::Display for DeckType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::MultipleArmored   => "Armoured deck - multiple decks",
                Self::SingleArmored     => "Armoured deck - single deck",
                Self::MultipleProtected => "Protected deck - multiple decks",
                Self::SingleProtected   => "Protected deck - single deck",
                Self::BoxOverMachinery  => "Box over machinery",
                Self::BoxOverMagazine   => "Box over magazines",
                Self::BoxOverBoth       => "Box over machiner & magazines",
            }
        )
    }
}

impl From<String> for DeckType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for DeckType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::SingleArmored,
            "2" => Self::MultipleProtected,
            "3" => Self::SingleProtected,
            "4" => Self::BoxOverMachinery,
            "5" => Self::BoxOverMagazine,
            "6" => Self::BoxOverBoth,
            "0" | _ => Self::MultipleArmored,
        }
    }
}

