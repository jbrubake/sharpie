use crate::{Ship, GunType, MountType, GunDistributionType, GunLayoutType, MineType, ASWType, TorpedoType};
use serde::{Serialize, Deserialize};
use std::f64::consts::PI;

// SubBattery {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SubBattery {
    pub layout: GunLayoutType,
    pub distribution: GunDistributionType,
    pub super_: u32,
    pub above: u32,
    pub on: u32,
    pub below: u32,
    pub lower: u32,
}

impl SubBattery {
    // super_ {{{2
    pub fn super_(&self) -> u32 {
        (2 * self.super_ + self.above - 2 * self.lower - self.below) * self.layout.guns_per()
    }

    // num_mounts {{{2
    pub fn num_mounts(&self) -> u32 {
        self.super_ + self.above + self.on + self.below + self.lower
    }

    // diameter_calc {{{2
    pub fn diameter_calc(&self, cal: f64) -> f64 {
        let mut calc = self.layout.num1() * cal * (1.0 + (1.0/cal).powf(self.layout.num2()));

        if cal < 12.0                        { calc += 12.0 / cal; }
        if cal > 1.0 && self.layout.wgt_adj() < 1.0 { calc *= 0.9; }

        calc
    }

    // wgt_adj {{{2
    pub fn wgt_adj(&self) -> f64 {
        self.layout.wgt_adj() * self.num_mounts() as f64
    }

    // free {{{2
    pub fn free(&self, fwd_len: f64, fd: f64, fd_fwd: f64, fd_aft: f64, ad: f64, ad_fwd: f64, ad_aft: f64) -> f64 {
        let mounts_fwd = match self.distribution {
            GunDistributionType::CenterlineFDFwd  => self.num_mounts(),
            GunDistributionType::CenterlineFD     => self.num_mounts(),
            GunDistributionType::CenterlineFDAft  => self.num_mounts(),
            GunDistributionType::CenterlineADFwd  => self.num_mounts(),
            GunDistributionType::SidesFDFwd       => self.num_mounts(),
            GunDistributionType::SidesFD          => self.num_mounts(),
            GunDistributionType::SidesFDAft       => self.num_mounts(),
            GunDistributionType::CenterlineAD     => 0,
            GunDistributionType::CenterlineADAft  => 0,
            GunDistributionType::SidesADFwd       => 0,
            GunDistributionType::SidesAD          => 0,
            GunDistributionType::SidesADAft       => 0,
            GunDistributionType::CenterlineEndsFD |
            GunDistributionType::SidesEndsFD =>
                if self.num_mounts() == 1 { self.num_mounts() } else { self.num_mounts() - self.num_mounts()/2 },
            GunDistributionType::CenterlineEndsAD |
            GunDistributionType::SidesEndsAD =>
                if self.num_mounts() == 1 { 0 } else { self.num_mounts() - self.num_mounts()/2 },
            GunDistributionType::CenterlineEven |
            GunDistributionType::SidesEven =>
                if self.num_mounts() == 1 && fwd_len >= 0.5 {
                    self.num_mounts()
                } else if fwd_len >= 0.5 {
                    self.num_mounts()/2
                } else if self.num_mounts() == 1 && fwd_len < 0.5 {
                    0
                } else {
                    self.num_mounts() - self.num_mounts()/2
                },
        };

        let fwd = mounts_fwd as f64;
        let tot = self.num_mounts() as f64;

        let free = match self.distribution {
            GunDistributionType::CenterlineEven |
            GunDistributionType::SidesEven => (fwd * fd + (tot - fwd) * ad) / tot,

            GunDistributionType::CenterlineEndsFD |
            GunDistributionType::CenterlineEndsAD |
            GunDistributionType::SidesEndsFD |
            GunDistributionType::SidesEndsAD => {
                let free = fwd * ((fd_fwd - fd) / fwd * 0.5 + (fd_fwd + fd) * 0.5);
                let free = free + if tot - fwd > 0.0 { (tot - fwd) * ((ad_aft - ad) / (tot - fwd) * 0.5 + (ad_aft + ad) * 0.5) } else { 0.0 };
                free / tot
            },

            GunDistributionType::CenterlineFDFwd |
            GunDistributionType::SidesFDFwd =>
                (fd_fwd - fd) / fwd * 0.5 + (fd_fwd + fd) * 0.5,

            GunDistributionType::CenterlineFD |
            GunDistributionType::SidesFD => fd,

            GunDistributionType::CenterlineFDAft |
            GunDistributionType::SidesFDAft =>
                (fd_aft - fd) / fwd * 0.5 + (fd_aft + fd) * 0.5,

            GunDistributionType::CenterlineADFwd |
            GunDistributionType::SidesADFwd =>
                (ad_fwd - ad) / (tot - fwd) * 0.5 + (ad_fwd + ad) * 0.5,

            GunDistributionType::CenterlineAD |
            GunDistributionType::SidesAD => ad,

            GunDistributionType::CenterlineADAft |
            GunDistributionType::SidesADAft =>
                (ad_aft - ad) / (tot - fwd) * 0.5 + (ad_aft + ad) * 0.5,
        };
        free * tot as f64
    }
}

// Battery {{{1
#[derive(Serialize, Deserialize, Debug)]
pub struct Battery {
    pub num: u32,
    pub cal: f64,
    pub len: u32,
    pub year: u32,
    pub shells: u32,
        shell_wgt: Option<f64>,
    pub kind: GunType,

    pub mount_num: u32,
    pub mount_kind: MountType,
    pub armor_face: u32,
    pub armor_other: u32,
    pub armor_barb: u32,

    pub groups: Vec<SubBattery>,
}

impl Default for Battery { // {{{1
    fn default() -> Self {
        Self {
            num: 0,
            cal: 0.0,
            len: 0,
            year: 1920,
            shells: 0,
            shell_wgt: None,
            kind: GunType::default(),

            mount_num: 0,
            mount_kind: MountType::default(),
            armor_face: 0,
            armor_other: 0,
            armor_barb: 0,

            groups: vec![
                SubBattery::default(),
                SubBattery::default(),
            ],
        }
    }
}

impl Battery { // {{{1
    const CORDITE_FACTOR: f64 = 2.444444;

    // super_ {{{2
    pub fn super_(&self, fwd_len: f64, fd: f64, fd_fwd: f64, fd_aft: f64, ad: f64, ad_fwd: f64, ad_aft: f64) -> f64 {
        let mut super_ = 0;
        for b in self.groups.iter() {
            super_ += b.super_()
        }

        let free = self.free(fwd_len, fd, fd_fwd, fd_aft, ad, ad_fwd, ad_aft);
        ((super_ / self.num) as f64 * (self.cal * 0.6).max(7.5) + free) / free
    }

    // free {{{2
    pub fn free(&self, fwd_len: f64, fd: f64, fd_fwd: f64, fd_aft: f64, ad: f64, ad_fwd: f64, ad_aft: f64) -> f64 {
        let mut f = 0.0;
        let mut mounts = 0;
        for b in self.groups.iter() {
            f += b.free(fwd_len, fd, fd_fwd, fd_aft, ad, ad_fwd, ad_aft);
            mounts += b.num_mounts();
        }

        f / mounts as f64
    }

    // wgt_adj {{{2
    pub fn wgt_adj(&self) -> f64 {
        let mut v = 0.0;
        let mut mounts = 0;
        for b in self.groups.iter() {
            v += b.wgt_adj();
            mounts += b.num_mounts();
        }

        v / mounts as f64
    }

    // date_factor {{{2
    fn date_factor(&self) -> f64 {
        Ship::year_adj(self.year).sqrt()
    }

    // set_shell_wgt {{{2
    pub fn set_shell_wgt(&mut self, wgt: f64) -> f64 {
        self.shell_wgt = Some(wgt);
        self.shell_wgt.unwrap()
    }

    // shell_wgt {{{2
    pub fn shell_wgt(&self) -> f64 {
        match self.shell_wgt {
            Some(wgt) => wgt,
            None      => {
                self.cal.powf(3.0) / 1.9830943211886 * self.date_factor() *
                (1.0 + ((45.0 - self.len as f64).abs().sqrt() / 45.0) * (self.len / self.len) as f64 )
            }, // self.len / self.len sets the addend sign as √ (45-len) fails if len > 45
        }
    }

    // gun_wgt {{{2
    pub fn gun_wgt(&self) -> f64 {
        self.shell_wgt() * (self.len as f64 / 812.289434917877 * (1.0 + (1.0 / self.len as f64).powf(2.3297949327695))) * self.num as f64
    }

    // mount_wgt {{{2
    pub fn mount_wgt(&self) -> f64 {
        let wgt = self.mount_kind.wgt() *
            if self.mount_kind.wgt_adj() < 0.6 {
                self.kind.wgt_sm()
            } else {
                self.kind.wgt_lg()
            };

        let wgt = (wgt + 1.0 / self.cal.powf(0.313068808543972)) * self.gun_wgt();

        let wgt =
            if self.cal > 10.0 {
                wgt * 1.0 - 2.1623769 * self.cal / 100.0
            } else if self.cal <= 1.0 {
                self.gun_wgt()
            } else {
                wgt
            };

        wgt * self.mount_kind.wgt_adj()
    }

    // broadside_wgt {{{2
    pub fn broadside_wgt(&self) -> f64 {
        self.num as f64 * self.shell_wgt()
    }

    // mag_wgt {{{2
    pub fn mag_wgt(&self) -> f64 {
        (self.num * self.shells) as f64 * self.shell_wgt() / crate::POUND2TON * (1.0 + Battery::CORDITE_FACTOR)
    }

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

