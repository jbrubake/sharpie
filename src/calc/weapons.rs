use crate::calc::{Measurement, Ship, Units};

use serde::{Deserialize, Serialize};

use std::f64::consts::PI;
use std::fmt;

// Torpedoes {{{1
/// A set of torpedo mounts or tubes.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Torpedoes {
    /// Units
    pub units: Units,
    /// Year torpedo was designed.
    pub year: u32,

    /// Number of mounts.
    pub mounts: u32,
    /// Type of mount.
    pub kind: TorpedoMountType,

    /// Number of torpedoes in the set
    pub num: u32,

    /// Torpedo diameter.
    pub diam: Measurement,
    /// Torpedo length.
    pub len: Measurement,
}

impl Torpedoes { // {{{2
    // wgt {{{3
    /// Weight of all torpedoes and mounts in the set.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    // wgt_weaps {{{3
    /// Weight of torpedoes in the set.
    ///
    pub fn wgt_weaps(&self) -> f64 {
        (
            PI * self.diam.imp().powf(2.0) * self.len.imp() /
            (
                (f64::max(1907.0 - self.year as f64, 0.0) + 25.0) * 937.0
            ) + (self.year as f64 - 1890.0) * 0.004
        ) * self.num as f64
    }

    // wgt_mounts {{{3
    /// Weight of mounts in the set.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.kind.wgt_factor() * self.wgt_weaps()
    }

    // hull_space {{{3
    /// Hull space taken up by the set.
    ///
    pub fn hull_space(&self) -> f64 {
        self.kind.hull_space(self.len.imp(), self.diam.imp()) * self.num as f64
    }

    // deck_space {{{3
    /// Deck space taken up by the set.
    ///
    pub fn deck_space(&self, b: f64) -> f64 {
        self.kind.deck_space(b, self.num, self.len.imp(), self.diam.imp(), self.mounts)
    }
}

// TorpedoMountType {{{1
/// Type of torpedo mount.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum TorpedoMountType {
    #[default]
    FixedTubes,
    DeckSideTubes,
    CenterTubes,
    DeckReloads,
    BowTubes,
    SternTubes,
    BowAndSternTubes,
    SubmergedSideTubes,
    SubmergedReloads,
}

choice_enum!(TorpedoMountType {
    FixedTubes         => ("deck mounted carriage/fixed tube"),
    DeckSideTubes      => ("deck mounted side rotating tube"),
    CenterTubes        => ("deck mounted centre rotating tube"),
    DeckReloads        => ("deck mounted reload"),
    BowTubes           => ("submerged bow tube"),
    SternTubes         => ("submerged stern tube"),
    BowAndSternTubes   => ("submerged bow & stern tube"),
    SubmergedSideTubes => ("submerged side tube"),
    SubmergedReloads   => ("below water reload"),
});

impl TorpedoMountType { // {{{2
    // wgt_factor {{{3
    /// Multiplier used to determine weight of torpedo mounts.
    ///
    pub fn wgt_factor(&self) -> f64 {
        match self {
            Self::FixedTubes         => 0.25,
            Self::DeckSideTubes      => 1.0,
            Self::CenterTubes        => 1.0,
            Self::DeckReloads        => 0.25,
            Self::BowTubes           => 1.0,
            Self::SternTubes         => 1.0,
            Self::BowAndSternTubes   => 1.0,
            Self::SubmergedSideTubes => 1.0,
            Self::SubmergedReloads   => 0.25,
        }
    }

    // hull_space {{{3
    /// Hull space taken up by torpedo mounts.
    ///
    pub fn hull_space(&self, len: f64, diam: f64) -> f64 {
        match self {
            Self::FixedTubes |
            Self::DeckSideTubes |
            Self::CenterTubes |
            Self::DeckReloads => 0.0,

            Self::BowTubes |
            Self::SternTubes |
            Self::BowAndSternTubes |
            Self::SubmergedSideTubes => len * 2.5 * (diam * 2.75/12.0).powf(2.0),

            Self::SubmergedReloads   => len * 1.5 * (diam * 1.5/12.0).powf(2.0),
        }
    }

    // deck_space {{{3
    /// Deck space taken up by torpedo mounts.
    ///
    pub fn deck_space(&self, b: f64, num: u32, len: f64, diam: f64, mounts: u32) -> f64 {
        let num = num as f64;
        let mounts = mounts as f64;

        match self {
            Self::FixedTubes => len * diam / 12.0 * num,

            Self::DeckSideTubes => {
                f64::powf(
                    f64::sqrt(
                        f64::powf(len,2.0) + f64::powf(((num/mounts)*diam/12.0)+(num/mounts-1.0)*0.5,2.0)
                    )*0.5,2.0
                )*PI+(((num/mounts)*diam/12.0)+(num/mounts-1.0)*0.5)*0.5*len
            },

            Self::CenterTubes => {
                let x = f64::powf(len, 2.0);
                let y = f64::powf((num / mounts) * diam / 12.0 + (num / mounts-1.0) * 0.5, 2.0);

                f64::sqrt(x + y) * b * mounts
            }

            Self::DeckReloads => len * 1.5 * (diam + 6.0) / 12.0 * num,

            Self::BowTubes |
            Self::SternTubes |
            Self::BowAndSternTubes |
            Self::SubmergedSideTubes |
            Self::SubmergedReloads   => 0.0,
        }
    }

    // desc {{{3
    /// Description of torpedo mounts.
    ///
    pub fn desc(&self, tubes: u32, mounts: u32) -> String {
        let desc = match self {
            Self::FixedTubes         => "deck mounted carriage/fixed tube",
            Self::DeckSideTubes      => "deck mounted side rotating tube",
            Self::CenterTubes        => "deck mounted centre rotating tube",
            Self::DeckReloads        => "deck mounted reload",
            Self::BowTubes           => "submerged bow tube",
            Self::SternTubes         => "submerged stern tube",
            Self::BowAndSternTubes   => &format!("submerged bow {} stern tube", if tubes > 1 { "&" } else { "OR" }).to_owned(),
            Self::SubmergedSideTubes => "submerged side tube",
            Self::SubmergedReloads   => "below water reload",
        };

        let prefix = match self {
            Self::FixedTubes |
            Self::DeckSideTubes |
            Self::CenterTubes |
            Self::DeckReloads => {
                if tubes > 1 {
                    format!("In {} sets of ", mounts)
                } else {
                    "In a ".into()
                }
            }

            _ => "".into(),
        };

        prefix + desc + if tubes > 1 { "s" } else { "" }
    }
}

// Testing Torpedo MountType {{{2
#[cfg(test)]
mod torpedo_mount_type {
    use super::*;
    use crate::calc::test_support::*;

    // Test wgt_factor {{{3
    macro_rules! test_wgt_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, torp) = $value;

                    assert_eq!(expected, torp.wgt_factor());
                }
            )*
        }
    }

    test_wgt_factor! {
        // name:               (factor, torp)
        wgt_factor_fixed:      (0.25, TorpedoMountType::FixedTubes),
        wgt_factor_deck:       (1.0, TorpedoMountType::DeckSideTubes),
        wgt_factor_center:     (1.0, TorpedoMountType::CenterTubes),
        wgt_factor_reload:     (0.25, TorpedoMountType::DeckReloads),
        wgt_factor_bow:        (1.0, TorpedoMountType::BowTubes),
        wgt_factor_stern:      (1.0, TorpedoMountType::SternTubes),
        wgt_factor_bow_stern:  (1.0, TorpedoMountType::BowAndSternTubes),
        wgt_factor_sub_side:   (1.0, TorpedoMountType::SubmergedSideTubes),
        wgt_factor_sub_reload: (0.25, TorpedoMountType::SubmergedReloads),
    }

    // Test hull_space {{{3
    macro_rules! test_hull_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, torp) = $value;

                    let len = 18.0; let diam = 21.0;
                    assert_eq!(expected, to_place(torp.hull_space(len, diam), 2));
                }
            )*
        }
    }

    test_hull_space! {
        // name:               (factor, torp)
        hull_space_fixed:      (0.0, TorpedoMountType::FixedTubes),
        hull_space_deck:       (0.0, TorpedoMountType::DeckSideTubes),
        hull_space_center:     (0.0, TorpedoMountType::CenterTubes),
        hull_space_reload:     (0.0, TorpedoMountType::DeckReloads),
        hull_space_bow:        (1042.21, TorpedoMountType::BowTubes),
        hull_space_stern:      (1042.21, TorpedoMountType::SternTubes),
        hull_space_bow_stern:  (1042.21, TorpedoMountType::BowAndSternTubes),
        hull_space_sub_side:   (1042.21, TorpedoMountType::SubmergedSideTubes),
        hull_space_sub_reload: (186.05, TorpedoMountType::SubmergedReloads),
    }

    // Test deck_space {{{3
    macro_rules! test_deck_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, torp) = $value;

                    let len = 18.0; let diam = 21.0; let num = 2; let mounts = 2;
                    let b = 50.0;
                    assert_eq!(expected, to_place(torp.deck_space(b, num, len, diam, mounts), 2));
                }
            )*
        }
    }

    test_deck_space! {
        // name:               (factor, torp)
        deck_space_fixed:      (63.0, TorpedoMountType::FixedTubes),
        deck_space_deck:       (272.62, TorpedoMountType::DeckSideTubes),
        deck_space_center:     (1808.49, TorpedoMountType::CenterTubes),
        deck_space_reload:     (121.5, TorpedoMountType::DeckReloads),
        deck_space_bow:        (0.0, TorpedoMountType::BowTubes),
        deck_space_stern:      (0.0, TorpedoMountType::SternTubes),
        deck_space_bow_stern:  (0.0, TorpedoMountType::BowAndSternTubes),
        deck_space_sub_side:   (0.0, TorpedoMountType::SubmergedSideTubes),
        deck_space_sub_reload: (0.0, TorpedoMountType::SubmergedReloads),
    }

    // Test from/index round-trip {{{3
    #[test]
    fn from_matches_sship_codes() {
        assert_eq!(TorpedoMountType::from("0"), TorpedoMountType::FixedTubes);
        assert_eq!(TorpedoMountType::from("4"), TorpedoMountType::BowTubes);
        assert_eq!(TorpedoMountType::from("8"), TorpedoMountType::SubmergedReloads);
    }

    #[test]
    fn index_roundtrip() {
        for v in TorpedoMountType::ALL {
            assert_eq!(TorpedoMountType::from_index(v.index()), *v);
            assert_eq!(TorpedoMountType::from(v.index().to_string()), *v);
        }
    }

    #[test]
    fn from_unknown_falls_back_to_default() {
        assert_eq!(TorpedoMountType::from("99"), TorpedoMountType::default());
        assert_eq!(TorpedoMountType::from("abc"), TorpedoMountType::default());
        assert_eq!(TorpedoMountType::from(""), TorpedoMountType::default());
    }

    #[test]
    fn labels_match_dropdown_order() {
        let labels: Vec<&str> = TorpedoMountType::ALL.iter().map(|v| v.label()).collect();
        assert_eq!(
            labels,
            ["deck mounted carriage/fixed tube", "deck mounted side rotating tube",
             "deck mounted centre rotating tube", "deck mounted reload",
             "submerged bow tube", "submerged stern tube",
             "submerged bow & stern tube", "submerged side tube",
             "below water reload"]
        );
    }
}

// Mines {{{1
/// Mines and deployment gear.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Mines {
    /// Units
    pub units: Units,

    /// Year mines were designed.
    pub year: u32,

    /// Number of mines.
    pub num: u32,
    /// Number of mine reloads.
    pub reload: u32,

    /// Weight of a single mine.
    pub wgt: Measurement,

    /// Type of mine deployment system.
    pub kind: MineType,
}

impl Mines { // {{{2
    // wgt {{{3
    /// Weight of mines, reloads and deployment gear.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    // wgt_weaps {{{3
    /// Weight of mines and reloads.
    ///
    pub fn wgt_weaps(&self) -> f64 {
        (self.num + self.reload) as f64 * self.wgt.imp() / Ship::POUND2TON
    }

    // wgt_mounts {{{3
    /// Weight of deployment gear.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.wgt_weaps() * self.kind.wgt_factor()
    }
}

// MineType {{{1
/// Types of mine deployment gear.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum MineType {
    #[default]
    SternRails,
    BowTubes,
    SternTubes,
    SideTubes,
}

choice_enum!(MineType {
    SternRails => ("Above water - Stern racks/rails"),
    BowTubes   => ("Below water - bow tubes"),
    SternTubes => ("Below water - stern tubes"),
    SideTubes  => ("Below water - side tubes"),
});

impl MineType { // {{{2
    // wgt_factor {{{3
    /// Multiplier to determine weight of mine deployment gear.
    ///
    pub fn wgt_factor(&self) -> f64 {
        match self {
            Self::SternRails => 0.25,
            Self::BowTubes   => 1.0,
            Self::SternTubes => 1.0,
            Self::SideTubes  => 1.0,
        }
    }

    // desc {{{3
    /// Description of mine deployment gear type.
    ///
    pub fn desc(&self) -> String {
        match self {
            Self::SternRails => "in Above water - Stern racks/rails",
            Self::BowTubes   => "in Below water - bow tubes",
            Self::SternTubes => "",
            Self::SideTubes  => "",
        }.into()
    }
}

// Testing MineType {{{2
#[cfg(test)]
mod mine_type {
    use super::*;

    // Test wgt_factor {{{3
    macro_rules! test_wgt_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mines) = $value;

                    assert_eq!(expected, mines.wgt_factor());
                }
            )*
        }
    }

    test_wgt_factor! {
        // name: (factor, mines)
        rails:   (0.25, MineType::SternRails),
        bow:     (1.0, MineType::BowTubes),
        stern:   (1.0, MineType::SternTubes),
        side:    (1.0, MineType::SideTubes),
    }

    // Test from/index round-trip {{{3
    #[test]
    fn from_matches_sship_codes() {
        assert_eq!(MineType::from("0"), MineType::SternRails);
        assert_eq!(MineType::from("1"), MineType::BowTubes);
        assert_eq!(MineType::from("2"), MineType::SternTubes);
        assert_eq!(MineType::from("3"), MineType::SideTubes);
    }

    #[test]
    fn index_roundtrip() {
        for v in MineType::ALL {
            assert_eq!(MineType::from_index(v.index()), *v);
            assert_eq!(MineType::from(v.index().to_string()), *v);
        }
    }

    #[test]
    fn from_unknown_falls_back_to_default() {
        assert_eq!(MineType::from("99"), MineType::default());
        assert_eq!(MineType::from("abc"), MineType::default());
        assert_eq!(MineType::from(""), MineType::default());
    }

    #[test]
    fn labels_match_dropdown_order() {
        let labels: Vec<&str> = MineType::ALL.iter().map(|v| v.label()).collect();
        assert_eq!(
            labels,
            ["Above water - Stern racks/rails", "Below water - bow tubes",
             "Below water - stern tubes", "Below water - side tubes"]
        );
    }
}

// ASW {{{1
/// ASW weapons and deployment gear.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ASW {
    /// Units.
    pub units: Units,

    /// Year ASW system was designed.
    pub year: u32,

    /// Number of weapons.
    pub num: u32,
    /// Number of reloads.
    pub reload: u32,

    /// Weight of a single weapon.
    pub wgt: Measurement,

    /// Type of weapon.
    pub kind: ASWType,
}

impl ASW { // {{{2
    // wgt {{{3
    /// Weight of weapons, reloads and mounts.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    // wgt_weaps {{{3
    /// Weight of weapons and reloads.
    ///
    pub fn wgt_weaps(&self) -> f64 {
        (self.num + self.reload) as f64 * self.wgt.imp() / Ship::POUND2TON
    }

    // wgt_mounts {{{3
    /// Weight of mounts.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.wgt_weaps() * self.kind.mount_wgt_factor()
    }
}

// Testing Torpedoes, Mines and ASW {{{2
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::*;
    use crate::units::UnitType;

    // Formula for Torpedoes::wgt_weaps().
    fn torp_weaps_wgt(diam: f64, len: f64, num: u32, year: u32) -> f64 {
        (PI * diam.powf(2.0) * len / ((f64::max(1907.0 - year as f64, 0.0) + 25.0) * 937.0)
            + (year as f64 - 1890.0) * 0.004)
            * num as f64
    }

    // Test mines_wgt_weaps {{{3
    macro_rules! test_mines_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(to_place(expected, 3), to_place(mines.wgt_weaps(), 3));
                }
            )*
        }
    }
    test_mines_wgt_weaps! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_weaps_mines_stern_rails: (200.0 * 10.0 / Ship::POUND2TON, MineType::SternRails, 100, 100, 10.0),
        wgt_weaps_mines_bow_tubes:   (200.0 * 10.0 / Ship::POUND2TON, MineType::BowTubes, 100, 100, 10.0),
        wgt_weaps_mines_stern_tubes: (200.0 * 10.0 / Ship::POUND2TON, MineType::SternTubes, 100, 100, 10.0),
        wgt_weaps_mines_side_tubes:  (200.0 * 10.0 / Ship::POUND2TON, MineType::SideTubes, 100, 100, 10.0),
    }

    // Test mines_wgt_mounts {{{3
    macro_rules! test_mines_wgt_mounts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(to_place(expected, 3), to_place(mines.wgt_mounts(), 3));
                }
            )*
        }
    }
    test_mines_wgt_mounts! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_mounts_mines_stern_rails: (200.0 * 10.0 / Ship::POUND2TON * MineType::SternRails.wgt_factor(), MineType::SternRails, 100, 100, 10.0),
        wgt_mounts_mines_bow_tubes:   (200.0 * 10.0 / Ship::POUND2TON * MineType::BowTubes.wgt_factor(), MineType::BowTubes, 100, 100, 10.0),
        wgt_mounts_mines_stern_tubes: (200.0 * 10.0 / Ship::POUND2TON * MineType::SternTubes.wgt_factor(), MineType::SternTubes, 100, 100, 10.0),
        wgt_mounts_mines_side_tubes:  (200.0 * 10.0 / Ship::POUND2TON * MineType::SideTubes.wgt_factor(), MineType::SideTubes, 100, 100, 10.0),
    }

    // Test mines_wgt {{{3
    macro_rules! test_mines_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(to_place(expected, 3), to_place(mines.wgt(), 3));
                }
            )*
        }
    }
    test_mines_wgt! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_mines_stern_rails: (200.0 * 10.0 / Ship::POUND2TON * (1.0 + MineType::SternRails.wgt_factor()), MineType::SternRails, 100, 100, 10.0),
        wgt_mines_bow_tubes:   (200.0 * 10.0 / Ship::POUND2TON * (1.0 + MineType::BowTubes.wgt_factor()), MineType::BowTubes, 100, 100, 10.0),
        wgt_mines_stern_tubes: (200.0 * 10.0 / Ship::POUND2TON * (1.0 + MineType::SternTubes.wgt_factor()), MineType::SternTubes, 100, 100, 10.0),
        wgt_mines_side_tubes:  (200.0 * 10.0 / Ship::POUND2TON * (1.0 + MineType::SideTubes.wgt_factor()), MineType::SideTubes, 100, 100, 10.0),
    }

    // Test asw_wgt_weaps {{{3
    macro_rules! test_asw_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut asw = ASW::default();
                    asw.kind = kind; asw.num = num; asw.reload = reload;
                    asw.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(expected, to_place(asw.wgt_weaps(), 3));
                }
            )*
        }
    }
    test_asw_wgt_weaps! {
        // name:                     (wgt, kind, num, reload, wgt)
        wgt_weaps_asw_stern_racks:   (0.893, ASWType::SternRacks, 100, 100, 10.0),
        wgt_weaps_asw_throwers:      (0.893, ASWType::Throwers, 100, 100, 10.0),
        wgt_weaps_asw_hedgehogs:     (0.893, ASWType::Hedgehogs, 100, 100, 10.0),
        wgt_weaps_asw_squid_mortars: (0.893, ASWType::SquidMortars, 100, 100, 10.0),
    }

    // Test asw_wgt_mounts {{{3
    macro_rules! test_asw_wgt_mounts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut asw = ASW::default();
                    asw.kind = kind; asw.num = num; asw.reload = reload;
                    asw.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(expected, to_place(asw.wgt_mounts(), 3));
                }
            )*
        }
    }
    test_asw_wgt_mounts! {
        // name:                      (expected, kind, num, reload, wgt)
        wgt_mounts_asw_stern_racks:   (0.223, ASWType::SternRacks, 100, 100, 10.0),
        wgt_mounts_asw_throwers:      (0.446, ASWType::Throwers, 100, 100, 10.0),
        wgt_mounts_asw_hedgehogs:     (0.446, ASWType::Hedgehogs, 100, 100, 10.0),
        wgt_mounts_asw_squid_mortars: (8.929, ASWType::SquidMortars, 100, 100, 10.0),
    }

    // Test asw_wgt {{{3
    macro_rules! test_asw_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut asw = ASW::default();
                    asw.kind = kind; asw.num = num; asw.reload = reload;
                    asw.wgt = Measurement::new(wgt, UnitType::Weight, Units::Imperial);

                    assert_eq!(to_place(expected, 3), to_place(asw.wgt(), 3));
                }
            )*
        }
    }
    test_asw_wgt! {
        // name:                      (expected, kind, num, reload, wgt)
        wgt_asw_stern_racks:   (200.0 * 10.0 / Ship::POUND2TON * (1.0 + ASWType::SternRacks.mount_wgt_factor()), ASWType::SternRacks, 100, 100, 10.0),
        wgt_asw_throwers:      (200.0 * 10.0 / Ship::POUND2TON * (1.0 + ASWType::Throwers.mount_wgt_factor()), ASWType::Throwers, 100, 100, 10.0),
        wgt_asw_hedgehogs:     (200.0 * 10.0 / Ship::POUND2TON * (1.0 + ASWType::Hedgehogs.mount_wgt_factor()), ASWType::Hedgehogs, 100, 100, 10.0),
        wgt_asw_squid_mortars: (200.0 * 10.0 / Ship::POUND2TON * (1.0 + ASWType::SquidMortars.mount_wgt_factor()), ASWType::SquidMortars, 100, 100, 10.0),
    }

    // Test torpedo_wgt_weaps {{{3
    macro_rules! test_torpedo_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.kind = kind;
                    torp.diam = Measurement::new(diam, UnitType::LengthSmall, Units::Imperial);
                    torp.len  = Measurement::new(len,  UnitType::LengthLong,  Units::Imperial);
                    torp.num = num; torp.year = year;

                    assert_eq!(to_place(expected, 3), to_place(torp.wgt_weaps(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt_weaps! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_weaps_torps_fixed_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_weaps_torps_deck_side_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_weaps_torps_center_tubes:        (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_weaps_torps_deck_reloads:        (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_weaps_torps_bow_tubes:           (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_weaps_torps_stern_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_weaps_torps_bow_and_stern_tubes: (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_weaps_torps_submerged_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_weaps_torps_submerged_reloads:   (torp_weaps_wgt(18.0, 21.0, 4, 1940), TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_wgt_mounts {{{3
    macro_rules! test_torpedo_wgt_mounts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.kind = kind;
                    torp.diam = Measurement::new(diam, UnitType::LengthSmall, Units::Imperial);
                    torp.len  = Measurement::new(len,  UnitType::LengthLong,  Units::Imperial);
                    torp.num = num; torp.year = year;

                    assert_eq!(to_place(expected, 3), to_place(torp.wgt_mounts(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt_mounts! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_mounts_torps_fixed_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::FixedTubes.wgt_factor(),         TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_mounts_torps_deck_side_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::DeckSideTubes.wgt_factor(),       TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_mounts_torps_center_tubes:        (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::CenterTubes.wgt_factor(),         TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_mounts_torps_deck_reloads:        (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::DeckReloads.wgt_factor(),         TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_mounts_torps_bow_tubes:           (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::BowTubes.wgt_factor(),            TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_mounts_torps_stern_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::SternTubes.wgt_factor(),          TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_mounts_torps_bow_and_stern_tubes: (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::BowAndSternTubes.wgt_factor(),    TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_mounts_torps_submerged_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::SubmergedSideTubes.wgt_factor(),  TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_mounts_torps_submerged_reloads:   (torp_weaps_wgt(18.0, 21.0, 4, 1940) * TorpedoMountType::SubmergedReloads.wgt_factor(),    TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_wgt {{{3
    macro_rules! test_torpedo_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.kind = kind;
                    torp.diam = Measurement::new(diam, UnitType::LengthSmall, Units::Imperial);
                    torp.len  = Measurement::new(len,  UnitType::LengthLong,  Units::Imperial);
                    torp.num = num; torp.year = year;

                    assert_eq!(to_place(expected, 3), to_place(torp.wgt(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_torps_fixed_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::FixedTubes.wgt_factor()),         TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_torps_deck_side_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::DeckSideTubes.wgt_factor()),       TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_torps_center_tubes:        (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::CenterTubes.wgt_factor()),         TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_torps_deck_reloads:        (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::DeckReloads.wgt_factor()),         TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_torps_bow_tubes:           (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::BowTubes.wgt_factor()),            TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_torps_stern_tubes:         (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::SternTubes.wgt_factor()),          TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_torps_bow_and_stern_tubes: (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::BowAndSternTubes.wgt_factor()),    TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_torps_submerged_tubes:     (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::SubmergedSideTubes.wgt_factor()),  TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_torps_submerged_reloads:   (torp_weaps_wgt(18.0, 21.0, 4, 1940) * (1.0 + TorpedoMountType::SubmergedReloads.wgt_factor()),    TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_hull_space {{{3
    macro_rules! test_torpedo_hull_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num) = $value;
                    let mut torp = Torpedoes::default();
                    torp.kind = kind;
                    torp.diam = Measurement::new(diam, UnitType::LengthSmall, Units::Imperial);
                    torp.len  = Measurement::new(len,  UnitType::LengthLong,  Units::Imperial);
                    torp.num = num;

                    assert_eq!(expected, to_place(torp.hull_space(), 3));
                }
            )*
        }
    }
    test_torpedo_hull_space! {
        // name:                             (space, kind, diam, len, num)
        test_hull_space_fixed_tubes:         (0.0, TorpedoMountType::FixedTubes,         18.0, 21.0, 4),
        test_hull_space_deck_side_tubes:     (0.0, TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4),
        test_hull_space_center_tubes:        (0.0, TorpedoMountType::CenterTubes,        18.0, 21.0, 4),
        test_hull_space_deck_reloads:        (0.0, TorpedoMountType::DeckReloads,        18.0, 21.0, 4),
        test_hull_space_bow_tubes:           (3573.281, TorpedoMountType::BowTubes,           18.0, 21.0, 4),
        test_hull_space_stern_tubes:         (3573.281, TorpedoMountType::SternTubes,         18.0, 21.0, 4),
        test_hull_space_bow_and_stern_tubes: (3573.281, TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4),
        test_hull_space_submerged_tubes:     (3573.281, TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4),
        test_hull_space_submerged_reloads:   (637.875, TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4),
    }

    // Test torpedo_deck_space {{{3
    macro_rules! test_torpedo_deck_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected,kind, diam, len, num, mounts) = $value;

                    let mut torp = Torpedoes::default();
                    torp.kind = kind;
                    torp.diam = Measurement::new(diam, UnitType::LengthSmall, Units::Imperial);
                    torp.len  = Measurement::new(len,  UnitType::LengthLong,  Units::Imperial);
                    torp.num = num; torp.mounts = mounts;

                    let b = 10.0;
                    assert_eq!(expected, to_place(torp.deck_space(b), 3));
                }
            )*
        }
    }
    test_torpedo_deck_space! {
        // name:                             (space, kind, diam, len, num, mounts)
        test_deck_space_fixed_tubes:         (126.0, TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 2),
        test_deck_space_deck_side_tubes:     (392.732, TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 2),
        test_deck_space_center_tubes:        (425.793, TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 2),
        test_deck_space_deck_reloads:        (252.0, TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 2),
        test_deck_space_bow_tubes:           (0.0, TorpedoMountType::BowTubes,           18.0, 21.0, 4, 2),
        test_deck_space_stern_tubes:         (0.0, TorpedoMountType::SternTubes,         18.0, 21.0, 4, 2),
        test_deck_space_bow_and_stern_tubes: (0.0, TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 2),
        test_deck_space_submerged_tubes:     (0.0, TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 2),
        test_deck_space_submerged_reloads:   (0.0, TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 2),
    }
}

// ASWType {{{1
/// Type of ASW deployment gear.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum ASWType {
    #[default]
    SternRacks,
    Throwers,
    Hedgehogs,
    SquidMortars,
}

choice_enum!(ASWType {
    SternRacks   => ("Stern depth charge racks"),
    Throwers     => ("Depth charge throwers"),
    Hedgehogs    => ("Hedgehog style A/S mortars"),
    SquidMortars => ("Squid style A/S mortars"),
});

impl ASWType { // {{{2
    // mount_wgt_factor {{{3
    /// Multiplier used to calculate total mount weight.
    ///
    pub fn mount_wgt_factor(&self) -> f64 {
        match self {
            Self::SternRacks   => 0.25,
            Self::Throwers     => 0.5,
            Self::Hedgehogs    => 0.5,
            Self::SquidMortars => 10.0,
        }
    }

    // desc {{{3
    /// Description of deployment gear.
    ///
    pub fn desc(&self) -> String {
        match self {
            Self::SternRacks   => "Depth Charges",
            Self::Throwers     => "Depth Charges",
            Self::Hedgehogs    => "ahead throwing AS Mortars",
            Self::SquidMortars => "trainable AS Mortars",
        }.into()
    }

    // dc_desc {{{3
    /// Description used to differentiate DC types.
    ///
    pub fn dc_desc(&self) -> String {
        match self {
            Self::SternRacks   => "in Stern depth charge racks",
            Self::Throwers     => "in Depth depth throwers",
            Self::Hedgehogs    => "",
            Self::SquidMortars => "",
        }.into()
    }
}

// Testing ASWType {{{2
#[cfg(test)]
mod asw_type {
    use super::*;

    // Test mount_wgt_factor {{{3
    macro_rules! test_mount_wgt_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, asw) = $value;

                    assert_eq!(expected, asw.mount_wgt_factor());
                }
            )*
        }
    }

    test_mount_wgt_factor! {
        // name: (factor, asw)
        racks:   (0.25, ASWType::SternRacks),
        throw:   (0.5, ASWType::Throwers),
        hedge:   (0.5, ASWType::Hedgehogs),
        squid:   (10.0, ASWType::SquidMortars),
    }

    // Test from/index round-trip {{{3
    #[test]
    fn from_matches_sship_codes() {
        assert_eq!(ASWType::from("0"), ASWType::SternRacks);
        assert_eq!(ASWType::from("1"), ASWType::Throwers);
        assert_eq!(ASWType::from("2"), ASWType::Hedgehogs);
        assert_eq!(ASWType::from("3"), ASWType::SquidMortars);
    }

    #[test]
    fn index_roundtrip() {
        for v in ASWType::ALL {
            assert_eq!(ASWType::from_index(v.index()), *v);
            assert_eq!(ASWType::from(v.index().to_string()), *v);
        }
    }

    #[test]
    fn from_unknown_falls_back_to_default() {
        assert_eq!(ASWType::from("99"), ASWType::default());
        assert_eq!(ASWType::from("abc"), ASWType::default());
        assert_eq!(ASWType::from(""), ASWType::default());
    }

    #[test]
    fn labels_match_dropdown_order() {
        let labels: Vec<&str> = ASWType::ALL.iter().map(|v| v.label()).collect();
        assert_eq!(
            labels,
            ["Stern depth charge racks", "Depth charge throwers",
             "Hedgehog style A/S mortars", "Squid style A/S mortars"]
        );
    }
}
