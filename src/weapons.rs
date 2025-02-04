use crate::{MineType, ASWType, TorpedoType};
use serde::{Serialize, Deserialize};
use std::f64::consts::PI;

// Battery {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Battery {
    pub num: u32,
    pub cal: f64,
    pub len: u32,
    pub year: u32,
    pub kind: u32,
    pub shells: u32,
    pub shell_wgt: f64,

    // pub mount_num: u32,
    // pub mount_kind: u32,
    // pub armor_face: u32,
    // pub armor_other: u32,
    // pub armor_barb: u32,

    // pub g1_kind: u32,
    // pub g1_layout: u32,
    // pub g1_super: u32,
    // pub g1_above: u32,
    // pub g1_on: u32,
    // pub g1_below: u32,

    // pub g2_kind: u32,
    // pub g2_layout: u32,
    // pub g2_super: u32,
    // pub g2_above: u32,
    // pub g2_on: u32,
    // pub g2_below: u32,
}

impl Battery {
    // new {{{2
    pub fn new() -> Battery {
        Default::default()
    }
}


// Torpedoes {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Torpedoes {
    pub year: u32,
    pub num: u32,
    pub mounts: u32,
    pub diam: f64,
    pub len: f64,
    pub kind: TorpedoType,
}

impl Torpedoes {
    // new {{{2
    pub fn new() -> Torpedoes {
        Default::default()
    }

    // wgt {{{2
    pub fn wgt(&self) -> f64 {
        let factor = match self.kind {
            TorpedoType::FixedTubes         => 0.25,
            TorpedoType::DeckSideTubes      => 1.0,
            TorpedoType::CenterTubes        => 1.0,
            TorpedoType::DeckReloads        => 0.25,
            TorpedoType::BowTubes           => 1.0,
            TorpedoType::SternTubes         => 1.0,
            TorpedoType::BowAndSternTubes   => 1.0,
            TorpedoType::SubmergedSideTubes => 1.0,
            TorpedoType::SubmergedReloads   => 0.25,
        };
        (
            (PI * self.diam.powf(2.0) * self.len) /
            (((1907 as f64 - self.year as f64) + 25.0) * 937.0).max(0.0)
        ) + 0.004 * (self.year as f64 - 1890 as f64) * self.num as f64 * factor
    }

    // hull_space {{{2
    pub fn hull_space(&self) -> f64 {
        match self.kind {
            TorpedoType::FixedTubes         => 0.0,
            TorpedoType::DeckSideTubes      => 0.0,
            TorpedoType::CenterTubes        => 0.0,
            TorpedoType::DeckReloads        => 0.0,
            TorpedoType::BowTubes           => self.len * 2.5 * (self.diam * 2.75/12.0).powf(2.0) * self.num as f64,
            TorpedoType::SternTubes         => self.len * 2.5 * (self.diam * 2.75/12.0).powf(2.0) * self.num as f64,
            TorpedoType::BowAndSternTubes   => self.len * 2.5 * (self.diam * 2.75/12.0).powf(2.0) * self.num as f64,
            TorpedoType::SubmergedSideTubes => self.len * 2.5 * (self.diam * 2.75/12.0).powf(2.0) * self.num as f64,
            TorpedoType::SubmergedReloads   => self.len * 1.5 * (self.diam * 1.5/12.0).powf(2.0) * self.num as f64,
        }
    }

    // deck_space {{{2
    pub fn deck_space(&self, b: f64) -> f64 {
        match self.kind {
            TorpedoType::FixedTubes         => self.len * self.diam / 12.0 * self.num as f64,
            TorpedoType::DeckSideTubes      => {
                (
                    (
                        self.len.powf(2.0) +
                        (
                            ((self.num / self.mounts) as f64 * self.diam / 12.0) +
                            (self.num / self.mounts - 1) as f64 * 0.5
                        ).powf(2.0)
                    ).sqrt()*0.5
                ).powf(2.0) * PI +
                (
                    ((self.num / self.mounts) as f64 * self.diam / 12.0) +
                    (self.num / self.mounts - 1) as f64 * 0.5
                ) * 0.5 * self.len
            },
            TorpedoType::CenterTubes        => {
                (
                    self.len.powf(2.0) +
                    (
                        ((self.num / self.mounts) as f64 * self.diam / 12.0) +
                        (self.num / self.mounts - 1) as f64 * 0.5
                    ).powf(2.0)
                ).sqrt() * b * self.mounts as f64
            },
            TorpedoType::DeckReloads        => self.len * 1.5 * (self.diam + 6.0) / 12.0 * self.num as f64,
            TorpedoType::BowTubes           => 0.0,
            TorpedoType::SternTubes         => 0.0,
            TorpedoType::BowAndSternTubes   => 0.0,
            TorpedoType::SubmergedSideTubes => 0.0,
            TorpedoType::SubmergedReloads   => 0.0,
        }

    }
}

// Mines {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Mines {
    pub year: u32,
    pub num: u32,
    pub reload: u32,
    pub wgt: f64,
    pub kind: MineType,
}

impl Mines {
    // new {{{2
    pub fn new() -> Mines {
        Default::default()
    }

    // wgt {{{2
    pub fn wgt(&self) -> f64 {
        let factor = match self.kind {
            MineType::SternRails => 0.25,
            MineType::BowTubes   => 1.0,
            MineType::SternTubes => 1.0,
            MineType::SideTubes  => 1.0,
        };
        ((self.num + self.reload) as f64 * self.wgt / crate::POUND2TON) * factor
    }
}

// ASW {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ASW {
    pub year: u32,
    pub num: u32,
    pub reload: u32,
    pub wgt: f64,
    pub kind: ASWType,
}

impl ASW {
    // new {{{2
    pub fn new() -> ASW {
        Default::default()
    }

    // wgt {{{2
    pub fn wgt(&self) -> f64 {
        let factor = match self.kind {
            ASWType::SternRacks   => 0.25,
            ASWType::Throwers     => 0.5,
            ASWType::Hedgehogs    => 0.5,
            ASWType::SquidMortars => 10.0,
        };
        ((self.num + self.reload) as f64 * self.wgt / crate::POUND2TON) * factor
    }
}

#[cfg(test)] // {{{1
mod test {
    use super::*;

    // Test mines_wgt {{{2
    macro_rules! test_mines_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (kind, num, reload, wgt) = $value;

                    let factor = match kind {
                        MineType::SternRails => 0.25,
                        MineType::BowTubes   => 1.0,
                        MineType::SternTubes => 1.0,
                        MineType::SideTubes  => 1.0,
                    };
                    let expected = ((num + reload) as f64 * wgt / crate::POUND2TON) * factor;

                    let mut mines = Mines::default();
                    mines.kind = kind; mines.num = num; mines.reload = reload; mines.wgt = wgt;

                    assert!(expected == mines.wgt());
                }
            )*
        }
    }
    test_mines_wgt! {
        // name:                    (kind, num, reload, wgt)
        test_wgt_mines_stern_rails: (MineType::SternRails, 100, 100, 10.0),
        test_wgt_mines_bow_tubes:   (MineType::BowTubes, 100, 100, 10.0),
        test_wgt_mines_stern_tubes: (MineType::SternTubes, 100, 100, 10.0),
        test_wgt_mines_side_tubes:  (MineType::SideTubes, 100, 100, 10.0),
    }

    // Test asw_wgt {{{2
    macro_rules! test_asw_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (kind, num, reload, wgt) = $value;

                    let factor = match kind {
                        ASWType::SternRacks   => 0.25,
                        ASWType::Throwers     => 0.5,
                        ASWType::Hedgehogs    => 0.5,
                        ASWType::SquidMortars => 10.0,
                    };
                    let expected = ((num + reload) as f64 * wgt / crate::POUND2TON) * factor;

                    let mut asw = ASW::default();
                    asw.kind = kind; asw.num = num; asw.reload = reload; asw.wgt = wgt;

                    assert!(expected == asw.wgt());
                }
            )*
        }
    }
    test_asw_wgt! {
        // name:                    (kind, num, reload, wgt)
        test_wgt_asw_stern_racks:   (ASWType::SternRacks, 100, 100, 10.0),
        test_wgt_asw_throwers:      (ASWType::Throwers, 100, 100, 10.0),
        test_wgt_asw_hedgehogs:     (ASWType::Hedgehogs, 100, 100, 10.0),
        test_wgt_asw_squid_mortars: (ASWType::SquidMortars, 100, 100, 10.0),
    }

    // Test torpedo_wgt {{{2
    macro_rules! test_torpedo_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (kind, diam, len, num, year) = $value;

                    let factor = match kind {
                        TorpedoType::FixedTubes         => 0.25,
                        TorpedoType::DeckSideTubes      => 1.0,
                        TorpedoType::CenterTubes        => 1.0,
                        TorpedoType::DeckReloads        => 0.25,
                        TorpedoType::BowTubes           => 1.0,
                        TorpedoType::SternTubes         => 1.0,
                        TorpedoType::BowAndSternTubes   => 1.0,
                        TorpedoType::SubmergedSideTubes => 1.0,
                        TorpedoType::SubmergedReloads   => 0.25,
                    };
                    let expected = (
                        (PI * diam.powf(2.0) * len) /
                        (((1907 as f64 - year as f64) + 25.0) * 937.0).max(0.0)
                    ) + 0.004 * (year - 1890) as f64 * num as f64 * factor;

                    let mut torp = Torpedoes::default();
                    torp.kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.year = year;

                    assert!(expected == torp.wgt());
                }
            )*
        }
    }
    test_torpedo_wgt! {
        // name:                  (kind, diam, len, num, year)
        test_wgt_fixed_tubes:         (TorpedoType::FixedTubes,         18.0 as f64, 21.0, 4, 1940),
        test_wgt_deck_side_tubes:     (TorpedoType::DeckSideTubes,      18.0 as f64, 21.0, 4, 1940),
        test_wgt_center_tubes:        (TorpedoType::CenterTubes,        18.0 as f64, 21.0, 4, 1940),
        test_wgt_deck_reloads:        (TorpedoType::DeckReloads,        18.0 as f64, 21.0, 4, 1940),
        test_wgt_bow_tubes:           (TorpedoType::BowTubes,           18.0 as f64, 21.0, 4, 1940),
        test_wgt_stern_tubes:         (TorpedoType::SternTubes,         18.0 as f64, 21.0, 4, 1940),
        test_wgt_bow_and_stern_tubes: (TorpedoType::BowAndSternTubes,   18.0 as f64, 21.0, 4, 1940),
        test_wgt_submerged_tubes:     (TorpedoType::SubmergedSideTubes, 18.0 as f64, 21.0, 4, 1940),
        test_wgt_submerged_reloads:   (TorpedoType::SubmergedReloads,   18.0 as f64, 21.0, 4, 1940),
    }

    // Test torpedo_hull_space {{{2
    macro_rules! test_torpedo_hull_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (kind, diam, len, num) = $value;

                    let expected = match kind {
                        TorpedoType::FixedTubes         => 0.0,
                        TorpedoType::DeckSideTubes      => 0.0,
                        TorpedoType::CenterTubes        => 0.0,
                        TorpedoType::DeckReloads        => 0.0,
                        TorpedoType::BowTubes           => len * 2.5 * (diam * 2.75/12.0).powf(2.0) * num as f64,
                        TorpedoType::SternTubes         => len * 2.5 * (diam * 2.75/12.0).powf(2.0) * num as f64,
                        TorpedoType::BowAndSternTubes   => len * 2.5 * (diam * 2.75/12.0).powf(2.0) * num as f64,
                        TorpedoType::SubmergedSideTubes => len * 2.5 * (diam * 2.75/12.0).powf(2.0) * num as f64,
                        TorpedoType::SubmergedReloads   => len * 1.5 * (diam * 1.5/12.0).powf(2.0) * num as f64,
                    };
                    let mut torp = Torpedoes::default();
                    torp.kind = kind; torp.diam = diam; torp.len = len; torp.num = num;

                    assert!(expected == torp.hull_space());
                }
            )*
        }
    }
    test_torpedo_hull_space! {
        // name:                             (kind, diam, len, num, year)
        test_hull_space_fixed_tubes:         (TorpedoType::FixedTubes,         18.0 as f64, 21.0, 4),
        test_hull_space_deck_side_tubes:     (TorpedoType::DeckSideTubes,      18.0 as f64, 21.0, 4),
        test_hull_space_center_tubes:        (TorpedoType::CenterTubes,        18.0 as f64, 21.0, 4),
        test_hull_space_deck_reloads:        (TorpedoType::DeckReloads,        18.0 as f64, 21.0, 4),
        test_hull_space_bow_tubes:           (TorpedoType::BowTubes,           18.0 as f64, 21.0, 4),
        test_hull_space_stern_tubes:         (TorpedoType::SternTubes,         18.0 as f64, 21.0, 4),
        test_hull_space_bow_and_stern_tubes: (TorpedoType::BowAndSternTubes,   18.0 as f64, 21.0, 4),
        test_hull_space_submerged_tubes:     (TorpedoType::SubmergedSideTubes, 18.0 as f64, 21.0, 4),
        test_hull_space_submerged_reloads:   (TorpedoType::SubmergedReloads,   18.0 as f64, 21.0, 4),
    }

    // Test torpedo_deck_space {{{2
    macro_rules! test_torpedo_deck_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (kind, diam, len, num, mounts) = $value;
                    let b = 10.0;

                    let expected = match kind {
                            TorpedoType::FixedTubes         => len * diam / 12.0 * num as f64,
                            TorpedoType::DeckSideTubes      => {
                                (
                                    (
                                        len.powf(2.0) +
                                        (
                                            ((num / mounts) as f64 * diam / 12.0) +
                                            (num / mounts - 1) as f64 * 0.5
                                        ).powf(2.0)
                                    ).sqrt()*0.5
                                ).powf(2.0) * PI +
                                    (
                                        ((num / mounts) as f64 * diam / 12.0) +
                                        (num / mounts - 1) as f64 * 0.5
                                    ) * 0.5 * len
                            },
                            TorpedoType::CenterTubes        => {
                                (
                                    len.powf(2.0) +
                                    (
                                        ((num / mounts) as f64 * diam / 12.0) +
                                        (num / mounts - 1) as f64 * 0.5
                                    ).powf(2.0)
                                ).sqrt() * b * mounts as f64
                            },
                            TorpedoType::DeckReloads        => len * 1.5 * (diam + 6.0) / 12.0 * num as f64,
                            TorpedoType::BowTubes           => 0.0,
                            TorpedoType::SternTubes         => 0.0,
                            TorpedoType::BowAndSternTubes   => 0.0,
                            TorpedoType::SubmergedSideTubes => 0.0,
                            TorpedoType::SubmergedReloads   => 0.0,
                        };

                    let mut torp = Torpedoes::default();
                    torp.kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.mounts = mounts;

                    assert!(expected == torp.deck_space(b));
                }
            )*
        }
    }
    test_torpedo_deck_space! {
        // name:                             (kind, diam, len, num, mounts)
        test_deck_space_fixed_tubes:         (TorpedoType::FixedTubes,         18.0 as f64, 21.0, 4, 2),
        test_deck_space_deck_side_tubes:     (TorpedoType::DeckSideTubes,      18.0 as f64, 21.0, 4, 2),
        test_deck_space_center_tubes:        (TorpedoType::CenterTubes,        18.0 as f64, 21.0, 4, 2),
        test_deck_space_deck_reloads:        (TorpedoType::DeckReloads,        18.0 as f64, 21.0, 4, 2),
        test_deck_space_bow_tubes:           (TorpedoType::BowTubes,           18.0 as f64, 21.0, 4, 2),
        test_deck_space_stern_tubes:         (TorpedoType::SternTubes,         18.0 as f64, 21.0, 4, 2),
        test_deck_space_bow_and_stern_tubes: (TorpedoType::BowAndSternTubes,   18.0 as f64, 21.0, 4, 2),
        test_deck_space_submerged_tubes:     (TorpedoType::SubmergedSideTubes, 18.0 as f64, 21.0, 4, 2),
        test_deck_space_submerged_reloads:   (TorpedoType::SubmergedReloads,   18.0 as f64, 21.0, 4, 2),
    }
}

