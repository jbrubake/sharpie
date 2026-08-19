use crate::Hull;
use crate::units::{Measurement, UnitType::{LengthSmall, LengthLong}, Units};

use serde::{Deserialize, Serialize};

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
    /// Upper belt armor.
    pub upper: Belt,
    /// Incline of belt armor.
    pub incline: f64,

    /// Torpedo bulge armor.
    ///
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
        self.main.len.imp() / (lwl * 0.65)
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

    // Test belt_coverage {{{3
    macro_rules! test_belt_coverage {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, belt_len, lwl) = $value;

                    let mut armor = Armor::default();
                    armor.main.len = Measurement::new(belt_len, LengthLong, Units::Imperial);

                    assert_eq!(expected, to_place(armor.belt_coverage(lwl), 2));
                }
            )*
        }
    }
    test_belt_coverage! {
        // name:       (belt_coverage, belt_len, lwl)
        belt_coverage: (1.0, 0.65, 1.0),
    }

    // Test max_hgt {{{3
    macro_rules! test_max_hgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, incline) = $value;

                    let t = 10.0;

                    let mut ship = test_ship();
                    ship.armor.incline = incline;

                    assert_eq!(
                        to_place(ship.armor.max_belt_hgt(t, ship.hull.freeboard_dist()), 2),
                        expected
                    );
                }
            )*
        }
    }
    test_max_hgt! {
        // name:            (max_hgt, incline)
        max_belt_hgt_0:     (20.02, 0.0),
        max_belt_hgt_45:     (28.3, 45.0),
        max_belt_hgt_neg_45: (28.3, -45.0),
    }

    // Test wgt {{{3

    // All-zero armor weighs nothing.
    #[test]
    fn wgt_zero() {
        let ship = test_ship();

        assert_eq!(to_place(ship.armor.wgt(ship.hull, 100.0, 100.0), 2), 0.0);
    }

    // The aggregate wgt() is the sum of every belt, the deck and both
    // conning towers.
    #[test]
    fn wgt_all() {
        let ship = test_ship();
        let mut armor = ship.armor;

        armor.main.thick = Measurement::new(1.0, LengthSmall, Units::Imperial);
        armor.main.len   = Measurement::new(20.0, LengthLong, Units::Imperial);
        armor.main.hgt   = Measurement::new(5.0, LengthLong, Units::Imperial);

        armor.end.thick = Measurement::new(1.0, LengthSmall, Units::Imperial);
        armor.end.len   = Measurement::new(20.0, LengthLong, Units::Imperial);
        armor.end.hgt   = Measurement::new(5.0, LengthLong, Units::Imperial);

        armor.upper.thick = Measurement::new(1.0, LengthSmall, Units::Imperial);
        armor.upper.len   = Measurement::new(20.0, LengthLong, Units::Imperial);
        armor.upper.hgt   = Measurement::new(5.0, LengthLong, Units::Imperial);

        armor.bulge.thick = Measurement::new(1.0, LengthSmall, Units::Imperial);
        armor.bulge.len   = Measurement::new(20.0, LengthLong, Units::Imperial);
        armor.bulge.hgt   = Measurement::new(5.0, LengthLong, Units::Imperial);

        armor.bulkhead.thick = Measurement::new(1.0, LengthSmall, Units::Imperial);
        armor.bulkhead.len   = Measurement::new(20.0, LengthLong, Units::Imperial);
        armor.bulkhead.hgt   = Measurement::new(5.0, LengthLong, Units::Imperial);

        armor.deck.fc = 0.5;
        armor.deck.md = 1.0;
        armor.deck.qd = 0.5;

        armor.ct_fwd.thick = 1.0;
        armor.ct_aft.thick = 2.0;

        assert_eq!(to_place(armor.wgt(ship.hull, 100.0, 100.0), 2), 110.32);
    }
}

// Belt {{{1
/// Belt, bulkhead and torpedo bulge armor.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Belt {
    /// Belt thickness.
    pub thick: Measurement,
    /// Belt length.
    pub len: Measurement,
    /// Belt height.
    pub hgt: Measurement,

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
        let len   = self.len.imp();
        let hgt   = self.hgt.imp();
        let thick = self.thick.imp();

        // Calculate the area of one bulkhead across the beam
        let beam_bulkhead = match self.kind {
            BeltType::Main | BeltType::Upper =>
                (1.0 - len / lwl).powf(1.0 - cwp) * b,
            _ => 0.0
        };

        // Calculate the weight of one belt and one bulkhead across the beam
        let wgt = (len + beam_bulkhead) * hgt * thick * Armor::INCH;

        // Double the weight to account for two belts and two beam bulkheads
        wgt * 2.0
    }

    // new {{{3
    /// Create a Belt of type "kind".
    ///
    pub fn new(kind: BeltType) -> Belt {
        Belt {
            thick: Measurement::new(0.0, LengthSmall, Units::Imperial),
            len:   Measurement::new(0.0, LengthLong, Units::Imperial),
            hgt:   Measurement::new(0.0, LengthLong, Units::Imperial),
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
                    belt.thick = Measurement::new(thick, LengthSmall, Units::Imperial);
                    belt.len   = Measurement::new(len, LengthLong, Units::Imperial);
                    belt.hgt   = Measurement::new(hgt, LengthLong, Units::Imperial);

                    assert_eq!(expected, to_place(belt.wgt(lwl, cwp, b), 2));
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
    /// Simpler and thinner.
    Strengthened,
    /// Modern, multilayered and thicker.
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
        10.0 * (d / 10_000.0).powf(2.0 / 3.0) * self.thick
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

                    assert_eq!(expected, to_place(ct.wgt(d), 2));
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

    // Test wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, fc, md, qd) = $value;

                    let mut ship = test_ship();

                    ship.armor.deck.kind = kind;
                    ship.armor.deck.fc = fc;
                    ship.armor.deck.md = md;
                    ship.armor.deck.qd = qd;

                    let wgt_mag    = 100.0;
                    let wgt_engine = ship.wgt_engine();

                    assert_eq!(
                        to_place(ship.armor.deck.wgt(ship.hull, wgt_mag, wgt_engine), 2),
                        expected
                    );
                }
            )*
        }
    }

    test_wgt! {
        //  name:             (wgt, deck, fc, md, qd)
        wgt_mult_arm_fc:      (6.67,   DeckType::MultipleArmored,    1.0, 0.0, 0.0),
        wgt_mult_arm_md:      (60.58,  DeckType::MultipleArmored,    0.0, 1.0, 0.0),
        wgt_mult_arm_qd:      (7.49,   DeckType::MultipleArmored,    0.0, 0.0, 1.0),
        wgt_mult_arm:         (74.74,  DeckType::MultipleArmored,    1.0, 1.0, 1.0),

        wgt_one_arm_fc:       (6.67,   DeckType::SingleArmored,      1.0, 0.0, 0.0),
        wgt_mult_prot_fc:     (6.67,   DeckType::MultipleProtected,  1.0, 0.0, 0.0),
        wgt_one_prot_fc:      (6.67,   DeckType::SingleProtected,    1.0, 0.0, 0.0),
        wgt_box_machinery_md: (194.5,  DeckType::BoxOverMachinery,   0.0, 1.0, 0.0),
        wgt_box_magazine_md:  (23.24,  DeckType::BoxOverMagazine,    0.0, 1.0, 0.0),
        // XXX: springsheet gives 202.77
        wgt_box_both_md:      (202.95, DeckType::BoxOverBoth,        0.0, 1.0, 0.0),
    }
    // Test Display {{{3
    macro_rules! test_display {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, layout) = $value;

                    assert_eq!(expected, format!("{}", layout));
                }
            )*
        }
    }

    test_display! {
        display_multiple_armored: ("Armoured deck - multiple decks", DeckType::MultipleArmored),
        display_single_armored: ("Armoured deck - single deck", DeckType::SingleArmored),
        display_multiple_protected: ("Protected deck - multiple decks", DeckType::MultipleProtected),
        display_single_protected: ("Protected deck - single deck", DeckType::SingleProtected),
        display_box_machinery: ("Box over machinery", DeckType::BoxOverMachinery),
        display_box_magazine: ("Box over magazines", DeckType::BoxOverMagazine),
        display_box_both: ("Box over machiner & magazines", DeckType::BoxOverBoth),
    }

    // Test From<&str> {{{3
    macro_rules! test_from_str {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, index) = $value;

                    assert_eq!(
                        std::mem::discriminant(&expected),
                        std::mem::discriminant(&index.into())
                    );
                }
            )*
        }
    }

    test_from_str! {
        // name: (type, index)
        from_str_default: (DeckType::MultipleArmored, "default"),
        from_str_zero:    (DeckType::MultipleArmored, "0"),
        from_str_one:     (DeckType::SingleArmored, "1"),
        from_str_two:     (DeckType::MultipleProtected, "2"),
        from_str_three:   (DeckType::SingleProtected, "3"),
        from_str_four:    (DeckType::BoxOverMachinery, "4"),
        from_str_five:    (DeckType::BoxOverMagazine, "5"),
        from_str_six:     (DeckType::BoxOverBoth, "6"),
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

