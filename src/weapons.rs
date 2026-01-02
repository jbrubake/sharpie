use crate::{Ship, Armor};
use crate::Hull;
use crate::units::Units;

use serde::{Serialize, Deserialize};

use std::f64::consts::PI;
use std::fmt;

// Battery {{{1
/// A battery of one type of gun.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Battery {
    /// Units
    pub units: Units,

    /// Number of guns in the battery.
    pub num: u32,

    /// Gun barrel diameter in inches.
    pub diam: f64,
    /// Gun barrel length in calibers.
    pub len: f64,

    /// Year gun was designed.
    pub year: u32,

    /// Number of shells in the magazine
    pub shells: u32,
    /// Weight of each shell.
        shell_wgt: Option<f64>,

    /// Type of gun.
    pub kind: GunType,

    /// Number of mounts in the battery.
    pub mount_num: u32,
    /// Kind of mounts.
    pub mount_kind: MountType,

    /// Armor thickness on mount face.
    pub armor_face: f64,
    /// Armor thickness elsewhere.
    // TODO: This should have a better name (other?)
    pub armor_back: f64,
    /// Armor thickness on barbette.
    pub armor_barb: f64,

    /// Separate groups of guns within the Battery
    pub groups: Vec<SubBattery>,
}

impl Default for Battery { // {{{2
    fn default() -> Self {
        Self {
            units: Units::Imperial, 

            num: 0,
            diam: 0.0,
            len: 0.0,
            year: 1920,
            shells: 0,
            shell_wgt: None,
            kind: GunType::default(),

            mount_num: 0,
            mount_kind: MountType::default(),
            armor_face: 0.0,
            armor_back: 0.0,
            armor_barb: 0.0,

            groups: vec![
                SubBattery::default(),
                SubBattery::default(),
            ],
        }
    }
}

impl Battery { // {{{2
    /// Factor to account for powder, etc. when calculating the magazine weight.
    ///
    const CORDITE_FACTOR: f64 = 0.2444444;

    // broad_and_below {{{3
    /// Returns true if the battery has Broadside mounts
    /// and any guns are mounted below the waterline.
    ///
    pub fn broad_and_below(&self) -> bool {
        if self.mount_kind == MountType::Broadside {
            for g in self.groups.iter() {
                if g.below != 0 { return true; }
            }
        }
        false
    }

    // concentration {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn concentration(&self, wgt_broad: f64) -> f64 {
        // Catch divide by zero
        if self.mount_num == 0 || wgt_broad == 0.0 { return 0.0; }

        (self.shell_wgt() * self.num as f64 / wgt_broad) *
            if self.mount_kind.wgt_adj() > 0.6 {
                (4.0 / self.mount_num as f64).powf(0.25) - 1.0
            } else {
                -0.1
            }
    }

    // super_ {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn super_(&self, hull: Hull) -> f64 {
        if self.num == 0 { return 0.0 } // catch divide by zero

        let mut super_ = 0;
        for g in self.groups.iter() {
            super_ += g.super_()
        }

        match self.free(hull) {
            0.0 => 0.0, // Catch divide by zero
            free => ((super_ as f64 / self.num as f64) * (self.diam * 0.6).max(7.5) + free) / free,
        }
    }

    // free {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn free(&self, hull: Hull) -> f64 {
        if self.mount_num == 0 { return 0.0 } // Catch divide by zero

        let mut f = 0.0;
        for b in self.groups.iter() {
            f += b.free(hull.clone());
        }

        f / self.mount_num as f64
    }

    // armor_face_wgt {{{3
    /// Weight of battery face armor.
    ///
    pub fn armor_face_wgt(&self) -> f64 {
        let wgt = self.mount_kind.armor_face_wgt(self.armor_back);

        let mut diameter_calc = 0.0;
        for g in self.groups.iter() {
            diameter_calc += g.diameter_calc(self.diam) * g.num_mounts() as f64;
        }

        let wgt = wgt * diameter_calc * self.house_hgt() * self.armor_face * Armor::INCH;

        wgt * self.kind.armor_face_wgt(self.armor_back)
    }

    // house_hgt {{{3
    /// XXX: I do not know what this does.
    ///
    fn house_hgt(&self) -> f64 {
        f64::max(
            7.5,
            0.625 * self.diam * self.mount_kind.gunhouse_hgt_factor(),
        )
    }

    // armor_back_wgt {{{3
    /// Weight of battery back armor.
    ///
    pub fn armor_back_wgt(&self) -> f64 {
        let (bw1, bw2) = self.mount_kind.armor_back_wgt();

        let mut a = 0.0;
        for g in self.groups.iter() {
            a += g.diameter_calc(self.diam) * g.num_mounts() as f64;
        }

        let mut b = 0.0;
        for g in self.groups.iter() {
            b += (g.diameter_calc(self.diam) / 2.0).powf(2.0) * g.num_mounts() as f64;
        }

        (bw1 * a * self.house_hgt() + PI * bw2 * b) * self.armor_back * Armor::INCH
    }
    // armor_barb_wgt {{{3
    /// Weight of battery barbette armor
    ///
    pub fn armor_barb_wgt(&self, hull: Hull) -> f64 {
        let mut guns = 0;
        for g in self.groups.iter() {
            guns += g.layout.guns_per() * g.num_mounts();
        }

        if self.mount_num == 0 { return 0.0; } // catch divide by zero

        let a = u32::min(
            if self.mount_kind.wgt_adj() > 0.5 { 4 } else { 5 },
            guns / self.mount_num,
        );

        let b = self.mount_kind.armor_barb_wgt();

        if self.free(hull.clone()) <= 0.0 {
            0.0
        } else {
            (1.0 - (a as f64 - 2.0) / 6.0) *
                self.armor_barb *
                 self.num as f64 *
                 self.diam.powf(1.2) *
                 b *
                 self.free(hull.clone()) / 16.0 *
                 self.super_(hull.clone()) *
                 b *
                 2.0 *
                 self.date_factor().sqrt()
                 
        }
    }
    // armor_wgt {{{3
    /// Total weight of the battery's armor.
    ///
    pub fn armor_wgt(&self, hull: Hull) -> f64 {
        self.armor_face_wgt() + self.armor_back_wgt() + self.armor_barb_wgt(hull)
    }

    // wgt_adj {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn wgt_adj(&self) -> f64 {
        if self.mount_num == 0 { return 0.0; } // Catch divide by zero

        let mut v = 0.0;
        for b in self.groups.iter() {
            v += b.wgt_adj();
        }

        v / self.mount_num as f64
    }

    // date_factor {{{3
    /// Factor used to adjust shell weight based on year.
    ///
    fn date_factor(&self) -> f64 {
        Ship::year_adj(self.year).sqrt()
    }

    // set_shell_wgt {{{3
    /// Set the shell weight.
    ///
    pub fn set_shell_wgt(&mut self, wgt: f64) -> f64 {
        self.shell_wgt = Some(wgt);
        
        wgt
    }

    // shell_wgt {{{3
    /// Get the shell weight.
    ///
    /// Return the value previously set by set_shell_wgt()
    /// or the estimated shell weight if unset.
    ///
    pub fn shell_wgt(&self) -> f64 {
        match self.shell_wgt {
            Some(wgt) => wgt,
            None      => self.shell_wgt_est(),
        }
    }

    // shell_wgt_est {{{3
    /// Estimated shell weight.
    ///
    pub fn shell_wgt_est(&self) -> f64 {
        self.diam.powf(3.0) / 1.9830943211886 * self.date_factor() *
            ( 1.0 + if self.len < 45.0 { -1.0 } else { 1.0 } * (45.0 - self.len).abs().sqrt() / 45.0 )
    }

    // gun_wgt {{{3
    /// Weight of the barrels in the battery.
    ///
    pub fn gun_wgt(&self) -> f64 {
        if self.diam == 0.0 { return 0.0; }

        self.shell_wgt_est() * (self.len as f64 / 812.289434917877 *
            (1.0 + (1.0 / self.diam as f64).powf(2.3297949327695))
            ) * self.num as f64
    }

    // mount_wgt {{{3
    /// Weight of a single gun mount.
    ///
    pub fn mount_wgt(&self) -> f64 {
        if self.diam == 0.0 { return 0.0; } // Catch divide by zero

        let wgt = self.mount_kind.wgt() *
            if self.mount_kind.wgt_adj() < 0.6 {
                self.kind.wgt_sm()
            } else {
                self.kind.wgt_lg()
            };

        let wgt = (wgt + 1.0 / self.diam.powf(0.313068808543972)) * self.gun_wgt();

        let wgt =
            if self.diam > 10.0 {
                wgt * (1.0 - 2.1623769 * self.diam / 100.0)
            } else if self.diam <= 1.0 {
                self.gun_wgt()
            } else {
                wgt
            };

        wgt * self.wgt_adj()
    }

    // broadside_wgt {{{3
    /// Weight of shells if each barrel fires a single shell.
    ///
    pub fn broadside_wgt(&self) -> f64 {
        self.num as f64 * self.shell_wgt()
    }

    // mag_wgt {{{3
    /// Weight of the battery magazine.
    ///
    pub fn mag_wgt(&self) -> f64 {
        (self.num * self.shells) as f64 * self.shell_wgt() / Ship::POUND2TON * (1.0 + Self::CORDITE_FACTOR)
    }
}

// Inernals Output {{{2
#[cfg(debug_assertions)]
impl Battery {
    pub fn internals(&self, hull: Hull, wgt_broad: f64) -> () {
        eprintln!("units = {}", self.units);
        eprintln!("num = {}", self.num);
        eprintln!("diam = {}", self.diam);
        eprintln!("len = {}", self.len);
        eprintln!("year = {}", self.year);
        eprintln!("shells = {}", self.shells);
        eprintln!("kind = {}", self.kind);
        eprintln!("mount_num = {}", self.mount_num);
        eprintln!("mount_kind = {}", self.mount_kind);
        eprintln!("armor_face = {}", self.armor_face);
        eprintln!("armor_back = {}", self.armor_back);
        eprintln!("armor_barb = {}", self.armor_barb);

        eprintln!("broad_and_below() = {}", self.broad_and_below());
        eprintln!("concentration() = {}", self.concentration(wgt_broad));
        eprintln!("super_() = {}", self.super_(hull.clone()));
        eprintln!("free() = {}", self.free(hull.clone()));
        eprintln!("house_hgt() = {}", self.house_hgt());
        eprintln!("armor_face_wgt() = {}", self.armor_face_wgt());
        eprintln!("armor_back_wgt() = {}", self.armor_back_wgt());
        eprintln!("armor_barb_wgt() = {}", self.armor_barb_wgt(hull.clone()));
        eprintln!("armor_wgt() = {}", self.armor_wgt(hull.clone()));
        eprintln!("wgt_adj() = {}", self.wgt_adj());
        eprintln!("date_factor() = {}", self.date_factor());
        eprintln!("shell_wgt() = {}", self.shell_wgt());
        eprintln!("shell_wgt_est() = {}", self.shell_wgt_est());
        eprintln!("gun_wgt() = {}", self.gun_wgt());
        eprintln!("mount_wgt() = {}", self.mount_wgt());
        eprintln!("broadside_wgt() = {}", self.broadside_wgt());
        eprintln!("mag_wgt() = {}", self.mag_wgt());
        eprintln!("");

        for (i, g) in self.groups.iter().enumerate() {
            eprintln!("Group {}", i);
            eprintln!("--------");
            g.internals(hull.clone(), self.diam);
        }
    }
}

// Testing Battery {{{2
#[cfg(test)]
mod battery {
    use super::*;
    use crate::test_support::*;

    // Test broad_and_below {{{3
    macro_rules! test_broad_and_below {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount_kind, guns_below) = $value;

                    let mut btry = Battery::default();
                    btry.mount_kind = mount_kind;
                    btry.groups[0].below = guns_below;

                    assert!(expected == btry.broad_and_below());
                }
            )*
        }
    }
    test_broad_and_below! {
        // name:                             (broad_and_below, mount_kind, guns_below)
        broad_and_below_not_broadside:       (false, MountType::Deck, 0),
        broad_and_below_broadside_not_below: (false, MountType::Broadside, 0),
        broad_and_below_broadside_below:     (true, MountType::Broadside, 1),
    }

    // Test concentration {{{3
    macro_rules! test_concentration {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, shell_wgt, mount_kind, mount_num) = $value;

                    let mut btry = Battery::default();
                    btry.set_shell_wgt(shell_wgt);
                    btry.mount_kind = mount_kind;
                    btry.mount_num = mount_num;
                    btry.num = 10;

                    let wgt_broadside = 1000.0;

                    println!("{}", btry.concentration(wgt_broadside));
                    assert!(expected == to_place(btry.concentration(wgt_broadside), 5));
                }
            )*
        }
    }
    test_concentration! {
        // name: (concentration, shell_wgt, mount_kind, mount_num)
        concentration_chk_div_by_0: (0.0, 0.0, MountType::Broadside, 0),
        concentration_sm_mount:     (-0.01, 10.0, MountType::Broadside, 1),
        concentration_lg_mount:     (0.04142, 10.0, MountType::ColesTurret, 1),
    }

    // Test super_ {{{3
    macro_rules! test_super_ {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, group_1_mounts, group_2_mounts) = $value;

                    let mut btry = Battery::default();

                    // Assume they are all single mounts
                    btry.num = group_1_mounts + group_2_mounts;
                    btry.mount_num = group_1_mounts + group_2_mounts;

                    btry.groups[0].above = group_1_mounts;
                    btry.groups[1].on = group_2_mounts;

                    btry.groups[0].distribution = GunDistributionType::CenterlineEven;
                    btry.groups[1].distribution = GunDistributionType::CenterlineEven;

                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 0.0;

                    hull.ad_fwd = 20.0;
                    hull.ad_aft = 0.0;

                    hull.qd_len = 0.15;

                    assert!(expected == to_place(btry.super_(hull), 5));
                }
            )*
        }
    }
    test_super_! {
        // name: (super_, group_1_mounts, group_2_mounts)
        super_test_1: (1.3, 2, 5),
        super_test_2: (1.75, 5, 2),
    }

    // Test free {{{3
    macro_rules! test_free {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, group_1_mounts, group_2_mounts) = $value;

                    let mut btry = Battery::default();

                    btry.mount_num = group_1_mounts + group_2_mounts;

                    btry.groups[0].on = group_1_mounts;
                    btry.groups[1].on = group_2_mounts;

                    btry.groups[0].distribution = GunDistributionType::CenterlineEven;
                    btry.groups[1].distribution = GunDistributionType::CenterlineEven;

                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 0.0;

                    hull.ad_fwd = 20.0;
                    hull.ad_aft = 0.0;

                    hull.qd_len = 0.15;

                    assert!(expected == to_place(btry.free(hull), 3));
                }
            )*
        }
    }
    test_free! {
        // name: (free, group_1_mounts, group_2_mounts)
        free_test_1: (7.143, 2, 5), 
        free_test_2: (7.5, 2, 0), 
        free_test_3: (7.0, 0, 5), 
    }

    // Test armor_face_wgt {{{3
    macro_rules! test_armor_face_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun_kind, mount_kind, armor_face, armor_back) = $value;

                    let mut btry = Battery::default();

                    btry.kind = gun_kind;
                    btry.mount_kind = mount_kind;
                    btry.armor_face = armor_face;
                    btry.armor_back = armor_back;
                    btry.diam = 10.0;

                    btry.groups[0].on = 2;
                    btry.groups[1].on = 0;

                    btry.groups[0].layout = GunLayoutType::Single;

                    assert!(expected == to_place(btry.armor_face_wgt(), 2));
                }
            )*
        }
    }
    test_armor_face_wgt! {
        // name: (armor_face_wgt, gun_kind, mount_kind, armor_face, armor_back)
        armor_face_wgt_no_back: (7.97, GunType::BreechLoading, MountType::DeckAndHoist, 1.0, 0.0),
        armor_face_wgt_back: (2.66, GunType::BreechLoading, MountType::DeckAndHoist, 1.0, 1.0),
    }

    // Test house_hgt {{{3
    macro_rules! test_house_hgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, diam) = $value;

                    let mut btry = Battery::default();
                    btry.diam = diam;
                    btry.mount_kind = MountType::Broadside;

                    assert!(expected == to_place(btry.house_hgt(), 5));
                }
            )*
        }
    }
    test_house_hgt! {
        // name: (house_hgt, diam)
        house_hgt_1: (8.75, 14.0),
        house_hgt_2: (7.5, 10.0),
    }

    // Test armor_back_wgt {{{3
    macro_rules! test_armor_back_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun_kind, mount_kind, armor_back) = $value;

                    let mut btry = Battery::default();

                    btry.kind = gun_kind;
                    btry.mount_kind = mount_kind;
                    btry.armor_back = armor_back;
                    btry.diam = 10.0;

                    btry.groups[0].on = 2;
                    btry.groups[1].on = 0;

                    btry.groups[0].layout = GunLayoutType::Single;

                    assert!(expected == to_place(btry.armor_back_wgt(), 2));
                }
            )*
        }
    }
    test_armor_back_wgt! {
        // name: (armor_back_wgt, gun_kind, mount_kind, armor_back)
        armor_back_wgt_1: (21.26, GunType::BreechLoading, MountType::DeckAndHoist, 1.0),
    }

    // Test armor_barb_wgt {{{3
    macro_rules! test_armor_barb_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun_kind, mount_kind, armor_barb) = $value;

                    let mut btry = Battery::default();

                    btry.kind = gun_kind;
                    btry.mount_kind = mount_kind;
                    btry.armor_barb = armor_barb;
                    btry.diam = 10.0;
                    btry.year = 1920;
                    btry.num = 2;

                    // Assume they are all single mounts
                    btry.mount_num = btry.num;
                    btry.groups[0].on = btry.num;
                    btry.groups[1].on = 0;

                    btry.groups[0].layout = GunLayoutType::Single;

                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 0.0;

                    hull.ad_fwd = 20.0;
                    hull.ad_aft = 0.0;

                    hull.qd_len = 0.15;

                    assert!(expected == to_place(btry.armor_barb_wgt(hull), 2));
                }
            )*
        }
    }
    test_armor_barb_wgt! {
        // name: (armor_barb_wgt, gun_kind, mount_kind, armor_barb)
        armor_barb_wgt_1: (0.35, GunType::BreechLoading, MountType::DeckAndHoist, 1.0),
    }

    // Test wgt_adj {{{3
    macro_rules! test_wgt_adj {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, g0_mounts, g1_mounts) = $value;

                    let mut btry = Battery::default();
                    btry.mount_num = g0_mounts + g1_mounts;

                    btry.groups[0].on = g0_mounts;
                    btry.groups[1].on = g1_mounts;
                    btry.groups[0].layout = GunLayoutType::Twin;
                    btry.groups[1].layout = GunLayoutType::Twin;

                    assert!(expected == to_place(btry.wgt_adj(), 5));
                }
            )*
        }
    }
    test_wgt_adj! {
        // name: (wgt_adj, g0_mounts, g1_mounts)
        wgt_adj_no_mounts: (0.0, 0, 0),
        wgt_adj_test: (0.75, 1, 2),
    }

    // Test date_factor {{{3
    macro_rules! test_date_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, year) = $value;

                    let mut btry = Battery::default();
                    btry.year = year;

                    assert!(expected == to_place(btry.date_factor(), 5));
                }
            )*
        }
    }
    test_date_factor! {
        // name: (date_factor, year)
        date_factor_sm: (0.99247, 1889),
    }

    // Test shell_wgt_est {{{3
    macro_rules! test_shell_wgt_est {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, len) = $value;

                    let mut btry = Battery::default();
                    btry.len = len;
                    btry.diam = 10.0;
                    btry.year = 1920;

                    assert!(expected == to_place(btry.shell_wgt_est(), 2));
                }
            )*
        }
    }
    test_shell_wgt_est! {
        // name: (shell_wgt_est, len)
        shell_wgt_est_sm: (493.06, 44.0),
        shell_wgt_est_45: (504.26, 45.0),
        shell_wgt_est_lg: (515.47, 46.0),
    }

    // Test gun_wgt {{{3
    macro_rules! test_gun_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, diam, len) = $value;

                    let mut btry = Battery::default();
                    btry.len = len;
                    btry.diam = diam;
                    btry.num = 1;
                    btry.year = 1920;

                    assert!(expected == to_place(btry.gun_wgt(), 2));
                }
            )*
        }
    }
    test_gun_wgt! {
        // name: (gun_wgt, diam, len)
        gun_wgt_cal_eq_0: (0.0, 0.0, 0.0),
        gun_wgt_test: (28.07, 10.0, 45.0),
    }

    // Test mount_wgt {{{3
    macro_rules! test_mount_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount_kind, diam) = $value;

                    let mut btry = Battery::default();
                    btry.mount_kind = mount_kind;
                    btry.diam = diam;
                    btry.len = 45.0;
                    btry.num = 1;
                    btry.year = 1920;
                    btry.kind = GunType::AntiAir;

                    btry.groups[0].on = 1;
                    btry.groups[1].on = 0;
                    btry.groups[0].layout = GunLayoutType::Single;
                    btry.groups[1].layout = GunLayoutType::Single;

                    btry.mount_num = btry.groups[0].num_mounts() +
                        btry.groups[1].num_mounts();

                    println!("{}", btry.mount_wgt());
                    assert!(expected == to_place(btry.mount_wgt(), 2));
                }
            )*
        }
    }
    test_mount_wgt! {
        // name: (mount_wgt, num)
        mount_wgt_cal_eq_0: (0.0, MountType::Broadside, 0.0),
        mount_wgt_sm_mount: (47.19, MountType::Broadside, 10.0),
        mount_wgt_lg_mount: (111.88, MountType::ColesTurret, 10.0),
        mount_wgt_lg_cal: (112.98, MountType::ColesTurret, 11.0),
        mount_wgt_sm_cal: (0.06, MountType::ColesTurret, 1.0),
    }

    // Test broadside_wgt {{{3
    macro_rules! test_broadside_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, num) = $value;

                    let mut btry = Battery::default();
                    btry.set_shell_wgt(10.0);
                    btry.num = num;

                    assert!(expected == btry.broadside_wgt());
                }
            )*
        }
    }
    test_broadside_wgt! {
        // name: (broadside_wgt, num)
        broadside_wgt_test: (100.0, 10),
    }

    // Test mag_wgt {{{3
    macro_rules! test_mag_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, num, shells, shell_wgt) = $value;

                    let mut btry = Battery::default();
                    btry.num = num;
                    btry.shells = shells;
                    btry.set_shell_wgt(shell_wgt);

                    assert!(to_place(expected, 2) == to_place(btry.mag_wgt(), 2));
                }
            )*
        }
    }
    test_mag_wgt! {
        // name: (mag_wgt, num, shells, shell_wgt)
        mag_wgt_test_1: (5.56, 10, 10, 100.0),
        mag_wgt_test_2: (1.0+Battery::CORDITE_FACTOR, 1, 1, Ship::POUND2TON),
    }
}

// GunType {{{1
/// Type of gun
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum GunType {
    MuzzleLoading,
    #[default]
    BreechLoading,
    QuickFiring,
    AntiAir,
    DualPurpose,
    RapidFire,
    MachineGun,
}

impl From<String> for GunType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for GunType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::BreechLoading,
            "2" => Self::QuickFiring,
            "3" => Self::AntiAir,
            "4" => Self::DualPurpose,
            "5" => Self::RapidFire,
            "6" => Self::MachineGun,
            "0" | _ => Self::MuzzleLoading,
        }
    }
}

impl fmt::Display for GunType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::MuzzleLoading => "Muzzle loading",
                Self::BreechLoading => "Breech loading",
                Self::QuickFiring   => "Quick-firing",
                Self::AntiAir       => "Anti-air",
                Self::DualPurpose   => "Dual-purpose",
                Self::RapidFire     => "Automatic rapid-fire",
                Self::MachineGun    => "Machine",
            }
        )
    }
}

impl GunType { // {{{2
    // armor_face_wgt {{{3
    /// Multiplier for determing the weight of a mount's face armor.
    ///
    pub fn armor_face_wgt(&self, armor_back: f64) -> f64 {
        let mut wgt =
            match self {
                Self::MuzzleLoading => 1.0,
                Self::BreechLoading => 1.0,
                Self::QuickFiring   => 1.0,
                Self::AntiAir       => 0.333,
                Self::DualPurpose   => 1.0,
                Self::RapidFire     => 1.0,
                Self::MachineGun    => 1.0,
            };

        if armor_back == 0.0 {
            wgt *=
                match self {
                    Self::MuzzleLoading => 1.0,
                    Self::BreechLoading => 1.0,
                    Self::QuickFiring   => 1.0,
                    Self::AntiAir       => 1.0,
                    Self::DualPurpose   => 1.0,
                    Self::RapidFire     => 1.0,
                    Self::MachineGun    => 0.333,
                };
        }

        wgt
    }

    // wgt_sm {{{3
    /// Multipler to adjust mount weight for small mounts.
    ///
    pub fn wgt_sm(&self) -> f64 {
        match self {
            Self::MuzzleLoading => 0.9,
            Self::BreechLoading => 1.0,
            Self::QuickFiring   => 1.35,
            Self::AntiAir       => 1.44,
            Self::DualPurpose   => 1.57,
            Self::RapidFire     => 2.16,
            Self::MachineGun    => 1.0,
        }
    }

    // wgt_lg {{{3
    /// Multipler to adjust mount weight for large mounts.
    ///
    pub fn wgt_lg(&self) -> f64 {
        match self {
            Self::MuzzleLoading => 0.98,
            Self::BreechLoading => 1.0,
            Self::QuickFiring   => 1.0,
            Self::AntiAir       => 1.0,
            Self::DualPurpose   => 1.1,
            Self::RapidFire     => 1.5,
            Self::MachineGun    => 1.0,
        }
    }
}

// Testing GunType {{{2
#[cfg(test)]
mod gun_type {
    use super::*;

    // Test armor_face_wgt {{{3
    macro_rules! test_armor_face_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun, back_armor) = $value;

                    assert_eq!(expected, gun.armor_face_wgt(back_armor));
                }
            )*
        }
    }

    test_armor_face_wgt! {
        // name:         (factor, mount, back_armor)
        face_wgt_muzzle: (1.0, GunType::MuzzleLoading, 1.0),
        face_wgt_breech: (1.0, GunType::BreechLoading, 1.0),
        face_wgt_qf:     (1.0, GunType::QuickFiring, 1.0),
        face_wgt_aa:     (0.333, GunType::AntiAir, 1.0),
        face_wgt_dp:     (1.0, GunType::DualPurpose, 1.0),
        face_wgt_rapdi:  (1.0, GunType::RapidFire, 1.0),
        face_wgt_mg:     (1.0, GunType::MachineGun, 1.0),

        // name:                 (factor, mount, back_armor)
        face_wgt_muzzle_no_back: (1.0, GunType::MuzzleLoading, 0.0),
        face_wgt_breech_no_back: (1.0, GunType::BreechLoading, 0.0),
        face_wgt_qf_no_back:     (1.0, GunType::QuickFiring, 0.0),
        face_wgt_aa_no_back:     (0.333, GunType::AntiAir, 0.0),
        face_wgt_dp_no_back:     (1.0, GunType::DualPurpose, 0.0),
        face_wgt_rapdi_no_back:  (1.0, GunType::RapidFire, 0.0),
        face_wgt_mg_no_back:     (0.333, GunType::MachineGun, 0.0),
    }
    // Test wgt_sm {{{3
    macro_rules! test_wgt_sm {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun) = $value;

                    assert_eq!(expected, gun.wgt_sm());
                }
            )*
        }
    }

    test_wgt_sm! {
        // name:       (factor, gun)
        wgt_sm_muzzle: (0.9, GunType::MuzzleLoading),
        wgt_sm_breech: (1.0, GunType::BreechLoading),
        wgt_sm_qf:     (1.35, GunType::QuickFiring),
        wgt_sm_aa:     (1.44, GunType::AntiAir),
        wgt_sm_dp:     (1.57, GunType::DualPurpose),
        wgt_sm_rf:     (2.16, GunType::RapidFire),
        wgt_sm_mg:     (1.0, GunType::MachineGun),
    }

    // Test wgt_lg {{{3
    macro_rules! test_wgt_lg {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, gun) = $value;

                    assert_eq!(expected, gun.wgt_lg());
                }
            )*
        }
    }

    test_wgt_lg! {
        // name:       (factor, gun)
        wgt_lg_muzzle: (0.98, GunType::MuzzleLoading),
        wgt_lg_breech: (1.0, GunType::BreechLoading),
        wgt_lg_qf:     (1.0, GunType::QuickFiring),
        wgt_lg_aa:     (1.0, GunType::AntiAir),
        wgt_lg_dp:     (1.1, GunType::DualPurpose),
        wgt_lg_rf:     (1.5, GunType::RapidFire),
        wgt_lg_mg:     (1.0, GunType::MachineGun),
    }
}

// MountType {{{1
/// Type of gun mount.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum MountType {
    Broadside,
    ColesTurret,
    OpenBarbette,
    ClosedBarbette,
    DeckAndHoist,
    #[default]
    Deck,
    Casemate,
}

impl From<String> for MountType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for MountType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::ColesTurret,
            "2" => Self::OpenBarbette,
            "3" => Self::ClosedBarbette,
            "4" => Self::DeckAndHoist,
            "5" => Self::Deck,
            "6" => Self::Casemate,
            "0" | _ => Self::Broadside,
        }
    }
}
impl fmt::Display for MountType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::Broadside      => "broadside",
                Self::ColesTurret    => "Coles/Ericsson turret",
                Self::OpenBarbette   => "open barbette",
                Self::ClosedBarbette => "turret on barbette",
                Self::DeckAndHoist   => "deck and hoist",
                Self::Deck           => "deck",
                Self::Casemate       => "casemate",
            }
        )
    }
}
impl MountType { // {{{2
    // gunhouse_hgt_factor {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn gunhouse_hgt_factor(&self) -> f64 {
        match self {
            Self::Broadside      => 1.0,
            Self::ColesTurret    => 2.0,
            Self::OpenBarbette   => 1.0,
            Self::ClosedBarbette => 1.0,
            Self::DeckAndHoist   => 1.0,
            Self::Deck           => 1.0,
            Self::Casemate       => 1.0,
        }
    }

    // armor_face_wgt {{{3
    /// Multiplier for determing the weight of a mount's face armor.
    ///
    pub fn armor_face_wgt(&self, armor_back: f64 ) -> f64 {
        use std::f64::consts::PI;

        let mut wgt = 
            match self {
                Self::Broadside      => 1.0,
                Self::ColesTurret    => PI / 2.0,
                Self::OpenBarbette   => 0.0,
                Self::ClosedBarbette => 0.5,
                Self::DeckAndHoist   => 0.5,
                Self::Deck           => 0.5,
                Self::Casemate       => 1.0,
            };

        if armor_back == 0.0 {
            wgt +=
                match self {
                    Self::Broadside      => 0.0,
                    Self::ColesTurret    => 0.0,
                    Self::OpenBarbette   => 0.0,
                    Self::ClosedBarbette => 1.0,
                    Self::DeckAndHoist   => 1.0,
                    Self::Deck           => 1.0,
                    Self::Casemate       => 0.0,
                }
        }

        wgt
    }

    // armor_back_wgt {{{3
    /// Multipliers needed to determine back armor weight for the mount.
    ///
    pub fn armor_back_wgt(&self) -> (f64, f64) {
        let a = match self {
            Self::Broadside      => 0.0,
            Self::ColesTurret    => 0.0,
            Self::OpenBarbette   => 0.0,
            Self::ClosedBarbette => 2.5,
            Self::DeckAndHoist   => 2.5,
            Self::Deck           => 2.5,
            Self::Casemate       => 0.0,
        };

        let b = match self {
            Self::Broadside      => 0.75,
            Self::ColesTurret    => 1.0,
            Self::OpenBarbette   => 0.75,
            Self::ClosedBarbette => 0.75,
            Self::DeckAndHoist   => 0.75,
            Self::Deck           => 0.75,
            Self::Casemate       => 0.75,
        };

        (a, b)
    }

    // armor_barb_wgt {{{3
    /// Multiplier to determine barbette armor weight.
    ///
    pub fn armor_barb_wgt(&self) -> f64 {
        match self {
            Self::Broadside      => 0.0,
            Self::ColesTurret    => 0.0,
            Self::OpenBarbette   => 0.6416,
            Self::ClosedBarbette => 0.5,
            Self::DeckAndHoist   => 0.1,
            Self::Deck           => 0.0,
            Self::Casemate       => 0.1,
        }
    }

    // wgt {{{3
    /// Multiplier for weight calculations.
    ///
    pub fn wgt(&self) -> f64 {
        match self {
            Self::Broadside      =>0.83,
            Self::ColesTurret    =>3.5,
            Self::OpenBarbette   =>3.33,
            Self::ClosedBarbette =>3.5,
            Self::DeckAndHoist   =>3.15,
            Self::Deck           =>1.08,
            Self::Casemate       =>1.08,
        }
    }
    // wgt_adj {{{3
    /// Multiplier for weight calculations.
    ///
    pub fn wgt_adj(&self) -> f64 {
        match self {
            Self::Broadside      =>0.5,
            Self::ColesTurret    =>1.0,
            Self::OpenBarbette   =>0.7,
            Self::ClosedBarbette =>1.0,
            Self::DeckAndHoist   =>1.0,
            Self::Deck           =>0.5,
            Self::Casemate       =>0.5,
        }
    }
}

// Testing MountType {{{2
#[cfg(test)]
mod mount_type {
    use super::*;

    use std::f64::consts::PI;

    // Test armor_wgt_adj {{{3
    macro_rules! test_armor_wgt_adj {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.wgt_adj());
                }
            )*
        }
    }

    test_armor_wgt_adj! {
        // name:             (factor, mount)
        wgt_adj_broad:       (0.5, MountType::Broadside),
        wgt_adj_coles:       (1.0, MountType::ColesTurret),
        wgt_adj_open_barb:   (0.7, MountType::OpenBarbette),
        wgt_adj_closed_barb: (1.0, MountType::ClosedBarbette),
        wgt_adj_deckhoist:   (1.0, MountType::DeckAndHoist),
        wgt_adj_deck:        (0.5, MountType::Deck),
        wgt_adj_casemate:    (0.5, MountType::Casemate),
    }

    // Test armor_wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.wgt());
                }
            )*
        }
    }

    test_wgt! {
        // name:         (factor, mount)
        wgt_broad:       (0.83, MountType::Broadside),
        wgt_coles:       (3.5, MountType::ColesTurret),
        wgt_open_barb:   (3.33, MountType::OpenBarbette),
        wgt_closed_barb: (3.5, MountType::ClosedBarbette),
        wgt_deckhoist:   (3.15, MountType::DeckAndHoist),
        wgt_deck:        (1.08, MountType::Deck),
        wgt_casemate:    (1.08, MountType::Casemate),
    }

    // Test armor_face_wgt {{{3
    macro_rules! test_armor_face_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount, back_armor) = $value;

                    assert_eq!(expected, mount.armor_face_wgt(back_armor));
                }
            )*
        }
    }

    test_armor_face_wgt! {
        // name:              (factor, mount, back_armor)
        face_wgt_broad:       (1.0, MountType::Broadside, 1.0),
        face_wgt_coles:       (PI / 2.0, MountType::ColesTurret, 1.0),
        face_wgt_open_barb:   (0.0, MountType::OpenBarbette, 1.0),
        face_wgt_closed_barb: (0.5, MountType::ClosedBarbette, 1.0),
        face_wgt_deckhoist:   (0.5, MountType::DeckAndHoist, 1.0),
        face_wgt_deck:        (0.5, MountType::Deck, 1.0),
        face_wgt_casemate:    (1.0, MountType::Casemate, 1.0),

        // name:                      (factor, mount, back_armor)
        face_wgt_broad_no_back:       (1.0, MountType::Broadside, 0.0),
        face_wgt_coles_no_back:       (PI / 2.0, MountType::ColesTurret, 0.0),
        face_wgt_open_barb_no_back:   (0.0, MountType::OpenBarbette, 0.0),
        face_wgt_closed_barb_no_back: (1.5, MountType::ClosedBarbette, 0.0),
        face_wgt_deckhoist_no_back:   (1.5, MountType::DeckAndHoist, 0.0),
        face_wgt_deck_no_back:        (1.5, MountType::Deck, 0.0),
        face_wgt_casemate_no_back:    (1.0, MountType::Casemate, 0.0),
    }

}

// SubBattery {{{1
/// Gun grouping within a battery.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SubBattery {
    /// Layout of guns within a turret.
    pub layout: GunLayoutType,
    /// Placement of guns on the ship.
    pub distribution: GunDistributionType,

    /// Number of mounts above the waterline.
    pub above: u32,
    /// Number of mounts on the waterline.
    pub on: u32,
    /// Number of mounts below the waterline.
    pub below: u32,

    /// If mounts above the deck are superfiring
    pub two_mounts_up: bool,
    /// If mounts below the waterline are on the lower deck
    pub lower_deck: bool,
}

// Internals Output {{{2
#[cfg(debug_assertions)]
impl SubBattery {
    pub fn internals(&self, hull: Hull, diam: f64) -> () {
        eprintln!("layout = {}", self.layout);
        eprintln!("distribution = {}", self.distribution);
        eprintln!("above = {}", self.above);
        eprintln!("on = {}", self.on);
        eprintln!("below = {}", self.below);
        eprintln!("two_mounts_up = {}", self.two_mounts_up);
        eprintln!("lower_deck = {}", self.lower_deck);
        eprintln!("super_() = {}", self.super_());
        eprintln!("num_mounts() = {}", self.num_mounts());
        eprintln!("diameter_calc() = {}", self.diameter_calc(diam));
        eprintln!("wgt_adj() = {}", self.wgt_adj());
        eprintln!("free() = {}", self.free(hull.clone()));
        eprintln!("");
    }
}

impl SubBattery { // {{{2
    // super_ {{{3
    /// Number of barrels above the waterline, reduced by the number of barrels
    /// below the waterline. Superfiring and lower deck barrels count double.
    ///
    pub fn super_(&self) -> i32 {
        let above: i32 = (self.above * if self.two_mounts_up { 2 } else { 1 }) as i32;
        let below: i32 = (self.below * if self.lower_deck    { 2 } else { 1 }) as i32;

        (above - below) * self.layout.guns_per() as i32
    }

    // num_mounts {{{3
    /// Total number of gun mounts.
    ///
    pub fn num_mounts(&self) -> u32 {
        self.above + self.on + self.below
    }

    // diameter_calc {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn diameter_calc(&self, diam: f64) -> f64 {
        if diam == 0.0 { return 0.0; } // Catch divide by zero

        let (factor, power) = self.layout.diameter_calc_nums();

        let mut calc = factor * diam * (1.0 + (1.0/diam).powf(power));

        if diam < 12.0                               { calc += 12.0 / diam; }
        if diam > 1.0 && self.layout.wgt_adj() < 1.0 { calc *= 0.9; }

        calc
    }

    // wgt_adj {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn wgt_adj(&self) -> f64 {
        self.layout.wgt_adj() * self.num_mounts() as f64
    }

    // free {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn free(&self, hull: Hull) -> f64 {
        let free = self.distribution.free(self.num_mounts(), hull);

        free * self.num_mounts() as f64
    }
}

// Testing SubBattery {{{2
#[cfg(test)]
mod sub_battery {
    use super::*;
    use crate::test_support::*;

    // Test super_ {{{3
    macro_rules! test_super_ {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, above, two_mounts_up, below, lower_deck) = $value;

                    let mut sub_btry = SubBattery::default();
                    sub_btry.layout = GunLayoutType::Single;

                    sub_btry.above = above;
                    sub_btry.below = below;
                    sub_btry.two_mounts_up = two_mounts_up;
                    sub_btry.lower_deck = lower_deck;

                    assert!(expected == sub_btry.super_());
                }
            )*
        }
    }
    test_super_! {
        // name:      (super_, above, two_mounts_up, below, lower_deck)
        super_test_1: ( 1, 1, false, 0, false),
        super_test_2: (-1, 0, false, 1, false),
        super_test_3: ( 2, 1, true, 0, true),
        super_test_4: (-2, 0, true, 1, true),
        super_test_5: ( 0, 1, false, 1, false),
        super_test_6: ( 0, 1, true, 1, true),
        super_test_7: (-1, 1, false, 1, true),
        super_test_8: ( 1, 1, true, 1, false),
    }

    // Test diameter_calc {{{3
    macro_rules! test_diameter_calc {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, diam) = $value;

                    let mut sub_btry = SubBattery::default();
                    sub_btry.layout = GunLayoutType::Single;

                    assert!(expected == to_place(sub_btry.diameter_calc(diam), 2));
                }
            )*
        }
    }
    test_diameter_calc! {
        // name:      (diameter_calc, diam)
        diameter_calc_cal_eq_0: (0.0, 0.0),
        diameter_calc_cal_lt_12: (19.14, 10.0),
        diameter_calc_cal_gt_1:  (12.30, 5.0),
        diameter_calc_cal_sm:  (25.82, 0.5),
    }

    // Test wgt_adj {{{3
    macro_rules! test_wgt_adj {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, num_mounts) = $value;

                    let mut sub_btry = SubBattery::default();
                    sub_btry.layout = GunLayoutType::Single;
                    sub_btry.above = num_mounts;
                    sub_btry.on = 0;
                    sub_btry.below = 0;

                    assert!(expected == to_place(sub_btry.wgt_adj(), 2));
                }
            )*
        }
    }
    test_wgt_adj! {
        // name:      (wgt_adj, num_mounts)
        wgt_adj_test: (10.0, 10),
    }

    // Test free {{{3
    macro_rules! test_free {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, num_mounts) = $value;

                    let mut sub_btry = SubBattery::default();
                    sub_btry.distribution = GunDistributionType::CenterlineEven;
                    sub_btry.above = num_mounts;
                    sub_btry.on = 0;
                    sub_btry.below = 0;

                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 0.0;

                    hull.ad_fwd = 20.0;
                    hull.ad_aft = 0.0;

                    hull.qd_len = 0.15;

                    assert!(expected == to_place(sub_btry.free(hull), 2));
                }
            )*
        }
    }
    test_free! {
        // name:   (free, num_mounts)
        free_test: (35.0, 5),
    }
}

// GunDistributionType {{{1
/// Distribution of gun mounts on the deck.
///
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum GunDistributionType {
    #[default]
    CenterlineEven,
    CenterlineEndsFD,
    CenterlineEndsAD,
    CenterlineFDFwd,
    CenterlineFD,
    CenterlineFDAft,
    CenterlineADFwd,
    CenterlineAD,
    CenterlineADAft,
    SidesEven,
    SidesEndsFD,
    SidesEndsAD,
    SidesFDFwd,
    SidesFD,
    SidesFDAft,
    SidesADFwd,
    SidesAD,
    SidesADAft,
}

impl From<String> for GunDistributionType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for GunDistributionType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::CenterlineEndsFD,
            "2" => Self::CenterlineEndsAD,
            "3" => Self::CenterlineFDFwd,
            "4" => Self::CenterlineFD,
            "5" => Self::CenterlineFDAft,
            "6" => Self::CenterlineADFwd,
            "7" => Self::CenterlineAD,
            "8" => Self::CenterlineADAft,
            "9" => Self::SidesEven,
            "10" => Self::SidesEndsFD,
            "11" => Self::SidesEndsAD,
            "12" => Self::SidesFDFwd,
            "13" => Self::SidesFD,
            "14" => Self::SidesFDAft,
            "15" => Self::SidesADFwd,
            "16" => Self::SidesAD,
            "17" => Self::SidesADAft,
            "0" | _ => Self::CenterlineEven,
        }
    }
}

impl fmt::Display for GunDistributionType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::CenterlineEven   => "centerline - distributed",
                Self::CenterlineEndsFD => "centerline - ends (fore ≥ aft)",
                Self::CenterlineEndsAD => "centerline - ends (aft ≥ fore)",
                Self::CenterlineFDFwd  => "centerline - foredeck forward",
                Self::CenterlineFD     => "centerline - foredeck",
                Self::CenterlineFDAft  => "centerline - foredeck aft",
                Self::CenterlineADFwd  => "centerline - afterdeck forward",
                Self::CenterlineAD     => "centerline - afterdeck",
                Self::CenterlineADAft  => "centerline - afterdeck aft",
                Self::SidesEven        => "sides - distributed",
                Self::SidesEndsFD      => "sides - ends (fore ≥ aft)",
                Self::SidesEndsAD      => "sides - ends (aft ≥ fore)",
                Self::SidesFDFwd       => "sides - foredeck forward",
                Self::SidesFD          => "sides - foredeck",
                Self::SidesFDAft       => "sides - foredeck aft",
                Self::SidesADFwd       => "sides - afterdeck forared",
                Self::SidesAD          => "sides - afterdeck",
                Self::SidesADAft       => "sides - afterdeck aft",
            }
        )
    }
}

impl GunDistributionType { // {{{2
    // desc {{{3
    /// Description of type based on number of mounts and length of decks.
    ///
    pub fn desc(&self, mounts: u32, fwd_len: f64) -> String {
        let s = match self {
            Self::CenterlineEven =>
                if mounts == 1 {
                    if fwd_len >= 0.5 {
                        "centreline amidships (forward deck)"
                    } else {
                        "centreline amidships (aft deck)"
                    }
                } else {
                    "centreline, evenly spread"
                },
            Self::CenterlineEndsFD =>
                if mounts == 1 {
                    "centreline forward"
                } else if mounts % 2 == 0 {
                    "centreline ends, evenly spread"
                } else {
                    "centreline ends, majority forward"
                },
            Self::CenterlineEndsAD =>
                if mounts == 1 {
                    "centreline aft"
                } else if mounts % 2 == 0 {
                    "centrelineends, evenly spread"
                } else {
                    "centreline ends, majority aft"
                },
            Self::CenterlineFDFwd => "centreline, forward deck forward",
            Self::CenterlineFD =>
                if mounts == 1 {
                    "centreline, forward deck centre"
                } else {
                    "centreline, forward evenly spread"
                },
            Self::CenterlineFDAft => "centreline, forward deck aft",
            Self::CenterlineADFwd => "centreline, aft deck forward",
            Self::CenterlineAD =>
                if mounts == 1 {
                    "centreline, aft deck centre"
                } else {
                    "centreline, aft evenly spread"
                },
            Self::CenterlineADAft => "cenreline, aft deck aft",
            Self::SidesEven =>
                if mounts < 3 {
                    "sides amidships"
                } else {
                    "sides, evenly spread"
                },
            Self::SidesEndsFD =>
                if mounts < 3 {
                    "sides, forward"
                } else if mounts % 4 == 0 {
                    "side ends, evenly spread"
                } else {
                    "side ends, majority forward"
                },
            Self::SidesEndsAD =>
                if mounts < 3 {
                    "sides aft"
                } else if mounts % 4 == 0 {
                    "side ends, evenly spread"
                } else {
                    "side ends, majority aft"
                },
            Self::SidesFDFwd => "sides, forward deck forward",
            Self::SidesFD =>
                if mounts < 3 {
                    "sides, forward deck centre"
                } else {
                    "sides, forward evenly spread"
                },
            Self::SidesFDAft => "sides, forward deck aft",
            Self::SidesADFwd => "sides, aft deck forward",
            Self::SidesAD =>
                if mounts < 3 {
                    "sides, aft deck centre"
                } else {
                    "sides, aft evenly spread"
                },
            Self::SidesADAft => "sides, aft deck aft",
        };

        s.into()
    }

    // super_aft {{{3
    /// True if the type would place guns aft.
    ///
    pub fn super_aft(&self) -> bool {
        let s = match self {
            Self::CenterlineEndsAD |
            Self::CenterlineADFwd |
            Self::CenterlineAD |
            Self::CenterlineADAft |
            Self::SidesEndsAD |
            Self::SidesADFwd |
            Self::SidesAD |
            Self::SidesADAft => true,

            _ => false,
        };

        s.into()
    }

    // mounts_fwd {{{3
    /// Number of mounts that are placed forward.
    ///
    fn mounts_fwd(&self, tot: u32, fwd_len: f64) -> u32 {
        // Divide n by 2 and round
        fn half(n: u32) -> u32 {
            f64::round(n as f64 / 2.0) as u32
        }

        match self {
            Self::CenterlineFDFwd  => tot,
            Self::CenterlineFD     => tot,
            Self::CenterlineFDAft  => tot,
            Self::CenterlineADFwd  => tot,
            Self::SidesFDFwd       => tot,
            Self::SidesFD          => tot,
            Self::SidesFDAft       => tot,

            Self::CenterlineAD     => 0,
            Self::CenterlineADAft  => 0,
            Self::SidesADFwd       => 0,
            Self::SidesAD          => 0,
            Self::SidesADAft       => 0,

            Self::CenterlineEndsFD | Self::SidesEndsFD =>
                if tot == 1 { tot } else { half(tot) },

            Self::CenterlineEndsAD | Self::SidesEndsAD =>
                if tot == 1 { 0 } else { tot - half(tot) },

            Self::CenterlineEven | Self::SidesEven =>
                if tot == 1 && fwd_len >= 0.5 {
                    tot
                } else if fwd_len >= 0.5 {
                    half(tot)
                } else if tot == 1 && fwd_len < 0.5 {
                    0
                } else {
                    tot - half(tot)
                },
        }
    }

    // free {{{3
    /// XXX: I do not know what this does
    ///
    pub fn free(&self, num_mounts: u32, hull: Hull) -> f64 {

        if num_mounts == 0 { return 0.0; } // catch divide by zero

        // Get these as floats to avoid casts later
        let fwd = self.mounts_fwd(num_mounts, hull.fc_len + hull.fd_len) as f64;
        let tot = num_mounts as f64;

        let fd     = hull.fd();
        let ad     = hull.ad();
        let fd_fwd = hull.fd_fwd;
        let fd_aft = hull.fd_aft;
        let ad_fwd = hull.ad_fwd;
        let ad_aft = hull.ad_aft;

        match self {
            Self::CenterlineEven | Self::SidesEven =>
                (fwd * fd + (tot - fwd) * ad) / tot,

            Self::CenterlineEndsFD | Self::CenterlineEndsAD |
            Self::SidesEndsFD | Self::SidesEndsAD =>
                (
                    if fwd > 0.0 {
                        fwd * ( (fd_fwd - fd) / fwd * 0.5 + (fd_fwd + fd) * 0.5)
                    } else {
                        0.0
                    }
                    + (tot - fwd) * ((ad_aft - ad) * 1.0 / (tot - fwd) * 0.5 + (ad_aft + ad) * 0.5)
                ) / tot,

            Self::CenterlineFDFwd | Self::SidesFDFwd =>
                if fwd > 0.0 {
                    (fd_fwd - fd) / fwd * 0.5 + (fd_fwd + fd) * 0.5
                } else {
                    0.0
                },

            Self::CenterlineFD | Self::SidesFD =>
                fd,

            Self::CenterlineFDAft | Self::SidesFDAft =>
                if fwd > 0.0 {
                    (fd_aft - fd) / fwd * 0.5 + (fd_aft + fd) * 0.5
                } else {
                    0.0
                },

            Self::CenterlineADFwd | Self::SidesADFwd =>
                if (tot - fwd) > 0.0 {
                    (ad_fwd - ad) / (tot - fwd) * 0.5 + (ad_fwd + ad) * 0.5
                } else {
                    0.0
                },

            Self::CenterlineAD | Self::SidesAD =>
                ad,

            Self::CenterlineADAft | Self::SidesADAft =>
                if (tot - fwd) > 0.0 {
                    (ad_aft - ad) / (tot - fwd) * 0.5 + (ad_aft + ad) * 0.5
                } else {
                    0.0
                }
        }

    }

    // gun_position {{{3
    /// XXX: I do not know what this does.
    ///
    fn gun_position(&self, fd_len: f64, ad_len: f64) -> f64 {
        match self {
            Self::CenterlineFDFwd  => 0.25 * fd_len,
            Self::CenterlineFD     => 0.5  * fd_len,
            Self::CenterlineFDAft  => 0.75 * fd_len,
            Self::CenterlineADFwd  => 0.25 * ad_len,
            Self::CenterlineAD     => 0.5  * ad_len,
            Self::CenterlineADAft  => 0.75 * ad_len,
            Self::SidesFDFwd       => 0.25 * fd_len,
            Self::SidesFD          => 0.5  * fd_len,
            Self::SidesFDAft       => 0.75 * fd_len,
            Self::SidesADFwd       => 0.25 * ad_len,
            Self::SidesAD          => 0.5  * ad_len,
            Self::SidesADAft       => 0.75 * ad_len,
            _                      => 0.0 // It is an error if we get here
        }
    }

    // g1_gun_position {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn g1_gun_position(&self, fd_len: f64, ad_len: f64) -> f64 {
        match self {
            Self::CenterlineEven |
            Self::CenterlineEndsFD |
            Self::CenterlineEndsAD |
            Self::SidesEven |
            Self::SidesEndsFD |
            Self::SidesEndsAD => 1.0,
            _ => self.gun_position(fd_len, ad_len),
        }
    }
    // g2_gun_position {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn g2_gun_position(&self, fd_len: f64, ad_len: f64) -> f64 {
        match self {
            Self::CenterlineEven |
            Self::CenterlineEndsFD |
            Self::CenterlineEndsAD |
            Self::SidesEven |
            Self::SidesEndsFD |
            Self::SidesEndsAD => 0.0,
            _ => self.gun_position(fd_len, ad_len),
        }
    }

    // super_factor_long {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn super_factor_long(&self) -> bool {
        match self {
            Self::CenterlineEven   => false,
            Self::CenterlineEndsFD => false,
            Self::CenterlineEndsAD => true,
            Self::CenterlineFDFwd  => true,
            Self::CenterlineFD     => true,
            Self::CenterlineFDAft  => true,
            Self::CenterlineADFwd  => true,
            Self::CenterlineAD     => true,
            Self::CenterlineADAft  => true,
            Self::SidesEven        => false,
            Self::SidesEndsFD      => false,
            Self::SidesEndsAD      => false,
            Self::SidesFDFwd       => true,
            Self::SidesFD          => true,
            Self::SidesFDAft       => true,
            Self::SidesADFwd       => true,
            Self::SidesAD          => true,
            Self::SidesADAft       => true,
        }
    }

}

// Testing GunD
#[cfg(test)] // GunDistributionType {{{2
mod gun_dist_type {
    use super::*;
    use crate::test_support::*;

    // Test g1_gun_position {{{3
    macro_rules! test_gun_position {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, dist) = $value;
                    let fd_len = 0.2; let ad_len = 0.2;

                    assert_eq!(expected, (
                            to_place(dist.g1_gun_position(fd_len, ad_len), 2),
                            to_place(dist.g2_gun_position(fd_len, ad_len), 2)
                            ));
                }
            )*
        }
    }

    test_gun_position! {
        // name:                       ((g1_pos, g2_pos), dist)
        gun_position_center:        ((1.0, 0.0), GunDistributionType::CenterlineEven),
        gun_position_center_end_fd: ((1.0, 0.0), GunDistributionType::CenterlineEndsFD),
        gun_position_center_end_ad: ((1.0, 0.0), GunDistributionType::CenterlineEndsAD),
        gun_position_sides:         ((1.0, 0.0), GunDistributionType::SidesEven),
        gun_position_sides_end_fd:  ((1.0, 0.0), GunDistributionType::SidesEndsFD),
        gun_position_sides_end_ad:  ((1.0, 0.0), GunDistributionType::SidesEndsAD),

        gun_position_center_fd_fwd: ((0.05, 0.05), GunDistributionType::CenterlineFDFwd),
        gun_position_center_fd:     ((0.1, 0.1), GunDistributionType::CenterlineFD),
        gun_position_center_fd_aft: ((0.15, 0.15), GunDistributionType::CenterlineFDAft),
        gun_position_center_ad_fwd: ((0.05, 0.05), GunDistributionType::CenterlineADFwd),
        gun_position_center_ad:     ((0.1, 0.1), GunDistributionType::CenterlineAD),
        gun_position_center_ad_aft: ((0.15, 0.15), GunDistributionType::CenterlineADAft),

        gun_position_sides_fd_fwd:  ((0.05, 0.05), GunDistributionType::SidesFDFwd),
        gun_position_sides_fd:      ((0.1, 0.1), GunDistributionType::SidesFD),
        gun_position_sides_fd_aft:  ((0.15, 0.15), GunDistributionType::SidesFDAft),
        gun_position_sides_ad_fwd:  ((0.05, 0.05), GunDistributionType::SidesADFwd),
        gun_position_sides_ad:      ((0.1, 0.1), GunDistributionType::SidesAD),
        gun_position_sides_ad_aft:  ((0.15, 0.15), GunDistributionType::SidesADAft),
    }

    // Test mounts_fwd {{{3
    macro_rules! test_mounts_fwd {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, tot, fwd_len, dist) = $value;

                    assert_eq!(expected, dist.mounts_fwd(tot, fwd_len));
                }
            )*
        }
    }

    test_mounts_fwd! {
        // name:                    (fwd, tot, fwd_len, mount)
        mounts_fwd_center_1:        (1, 1, 0.5, GunDistributionType::CenterlineEven),
        mounts_fwd_center_2:        (2, 3, 0.5, GunDistributionType::CenterlineEven),
        mounts_fwd_center_3:        (0, 1, 0.4, GunDistributionType::CenterlineEven),
        mounts_fwd_center_4:        (1, 3, 0.4, GunDistributionType::CenterlineEven),
        mounts_fwd_center_end_fd_1: (1, 1, 0.0, GunDistributionType::CenterlineEndsFD),
        mounts_fwd_center_end_fd_2: (2, 3, 0.0, GunDistributionType::CenterlineEndsFD),
        mounts_fwd_center_end_ad_1: (0, 1, 0.0, GunDistributionType::CenterlineEndsAD),
        mounts_fwd_center_end_ad_2: (1, 3, 0.0, GunDistributionType::CenterlineEndsAD),
        mounts_fwd_center_fd_fwd:   (3, 3, 0.0, GunDistributionType::CenterlineFDFwd),
        mounts_fwd_center_fd:       (3, 3, 0.0, GunDistributionType::CenterlineFD),
        mounts_fwd_center_fd_aft:   (3, 3, 0.0, GunDistributionType::CenterlineFDAft),
        mounts_fwd_center_ad_fwd:   (3, 3, 0.0, GunDistributionType::CenterlineADFwd),
        mounts_fwd_center_ad:       (0, 3, 0.0, GunDistributionType::CenterlineAD),
        mounts_fwd_center_ad_aft:   (0, 3, 0.0, GunDistributionType::CenterlineADAft),

        mounts_fwd_sides_1:         (1, 1, 0.5, GunDistributionType::SidesEven),
        mounts_fwd_sides_2:         (2, 3, 0.5, GunDistributionType::SidesEven),
        mounts_fwd_sides_3:         (0, 1, 0.4, GunDistributionType::SidesEven),
        mounts_fwd_sides_4:         (1, 3, 0.4, GunDistributionType::SidesEven),
        mounts_fwd_sides_end_fd_1:  (1, 1, 0.0, GunDistributionType::SidesEndsFD),
        mounts_fwd_sides_end_fd_2:  (2, 3, 0.0, GunDistributionType::SidesEndsFD),
        mounts_fwd_sides_end_ad_1:  (0, 1, 0.0, GunDistributionType::SidesEndsAD),
        mounts_fwd_sides_end_ad_2:  (1, 3, 0.0, GunDistributionType::SidesEndsAD),
        mounts_fwd_sides_fd_fwd:    (3, 3, 0.0, GunDistributionType::SidesFDFwd),
        mounts_fwd_sides_fd:        (3, 3, 0.0, GunDistributionType::SidesFD),
        mounts_fwd_sides_fd_aft:    (3, 3, 0.0, GunDistributionType::SidesFDAft),
        mounts_fwd_sides_ad_fwd:    (0, 3, 0.0, GunDistributionType::SidesADFwd),
        mounts_fwd_sides_ad:        (0, 3, 0.0, GunDistributionType::SidesAD),
        mounts_fwd_sides_ad_aft:    (0, 3, 0.0, GunDistributionType::SidesADAft),
    }

    // Test free {{{3
    macro_rules! test_free {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, num, dist) = $value;
                    let mut hull = Hull::default();
                    hull.fc_len = 0.2;

                    hull.fd_len = 0.3;
                    hull.fd_fwd = 10.0;
                    hull.fd_aft = 0.0;

                    hull.ad_fwd = 20.0;
                    hull.ad_aft = 0.0;

                    hull.qd_len = 0.15;

                    assert_eq!(expected, to_place(dist.free(num, hull), 3));
                }
            )*
        }
    }

    test_free! {
        // name:       (free, mounts, fd, ad, fd_fwd, fd_aft, ad_fwd, ad_aft, dist)
        free_tot_eq_0: (0.0, 0, GunDistributionType::CenterlineEven),
        free_case_1_1: (7.0, 5, GunDistributionType::CenterlineEven),
        free_case_1_2: (7.0, 5, GunDistributionType::SidesEven),
        free_case_2_1: (6.0, 5, GunDistributionType::CenterlineEndsFD),
        free_case_2_2: (5.5, 5, GunDistributionType::CenterlineEndsAD),
        free_case_2_3: (6.0, 5, GunDistributionType::SidesEndsFD),
        free_case_2_4: (5.5, 5, GunDistributionType::SidesEndsAD),
        free_case_3_1: (8.0, 5, GunDistributionType::CenterlineFDFwd),
        free_case_3_2: (8.0, 5, GunDistributionType::SidesFDFwd),
        free_case_4_1: (5.0, 5, GunDistributionType::CenterlineFD),
        free_case_4_2: (5.0, 5, GunDistributionType::SidesFD),
        free_case_5_1: (2.0, 5, GunDistributionType::CenterlineFDAft),
        free_case_5_2: (2.0, 5, GunDistributionType::SidesFDAft),
        free_case_6_1: (0.0, 5, GunDistributionType::CenterlineADFwd),
        free_case_6_2: (16.0, 5, GunDistributionType::SidesADFwd),
        free_case_7_1: (10.0, 5, GunDistributionType::CenterlineAD),
        free_case_7_2: (10.0, 5, GunDistributionType::SidesAD),
        free_case_8_1: (4.0, 5, GunDistributionType::CenterlineADAft),
        free_case_8_2: (4.0, 5, GunDistributionType::SidesADAft),
    }
}

// GunLayoutType {{{1
/// Layout of guns within a mount.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum GunLayoutType {
    #[default]
    Single,
    Twin2Row,
    Quad4Row,
    Twin,
    TwoGun,
    Quad2Row,
    Triple,
    ThreeGun,
    Sex2Row,
    Quad,
    FourGun,
    Oct2Row,
    Quint,
    FiveGun,
    Dec2Row,
}

impl From<String> for GunLayoutType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for GunLayoutType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::Twin2Row,
            "2" => Self::Quad4Row,
            "3" => Self::Twin,
            "4" => Self::TwoGun,
            "5" => Self::Quad2Row,
            "6" => Self::Triple,
            "7" => Self::ThreeGun,
            "8" => Self::Sex2Row,
            "9" => Self::Quad,
            "10" => Self::FourGun,
            "11" => Self::Oct2Row,
            "12" => Self::Quint,
            "13" => Self::FiveGun,
            "14" => Self::Dec2Row,
            "0" | _ => Self::Single,
        }
    }
}

impl fmt::Display for GunLayoutType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::Single   => "Single",
                Self::Twin2Row => "2 row, twin",
                Self::Quad4Row => "4 row, quad",
                Self::Twin     => "Twin",
                Self::TwoGun   => "2-gun",
                Self::Quad2Row => "2 row, quad",
                Self::Triple   => "Triple",
                Self::ThreeGun => "3-gun",
                Self::Sex2Row  => "2 row, sextuple",
                Self::Quad     => "quad",
                Self::FourGun  => "4-gun",
                Self::Oct2Row  => "2 row, octuple",
                Self::Quint    => "quintuple",
                Self::FiveGun  => "5-gun",
                Self::Dec2Row  => "2 row, decuple",
            }
        )
    }
}

impl GunLayoutType { // {{{2
    // num_guns {{{3
    /// Number of guns per mount.
    ///
    pub fn guns_per(&self) -> u32 {
        match self {
            Self::Single   => 1,
            Self::Twin2Row => 2,
            Self::Twin     => 2,
            Self::TwoGun   => 2,
            Self::Triple   => 3,
            Self::ThreeGun => 3,
            Self::Quad2Row => 4,
            Self::Quad4Row => 4,
            Self::Quad     => 4,
            Self::FourGun  => 4,
            Self::Quint    => 5,
            Self::FiveGun  => 5,
            Self::Sex2Row  => 6,
            Self::Oct2Row  => 8,
            Self::Dec2Row  => 10,
        }
    }

    // diameter_calc_nums {{{3
    /// Return values needed for SubBattery::diameter_calc().
    ///
    pub fn diameter_calc_nums(&self) -> (f64, f64) {
        match self {
            Self::Single   => (1.44, 0.609725),
            Self::Twin2Row => (1.44, 0.609725),
            Self::Quad4Row => (1.44, 0.609725),
            Self::Twin     => (1.52, 0.4205),
            Self::TwoGun   => (1.52, 0.4205),
            Self::Quad2Row => (1.52, 0.4205),
            Self::Triple   => (1.64, 0.29),
            Self::ThreeGun => (1.64, 0.29),
            Self::Sex2Row  => (1.64, 0.29),
            Self::Quad     => (1.8, 0.2),
            Self::FourGun  => (1.8, 0.2),
            Self::Oct2Row  => (1.8, 0.2),
            Self::Quint    => (2.0, 0.14),
            Self::FiveGun  => (2.0, 0.14),
            Self::Dec2Row  => (2.0, 0.14),
        }
    }

    // wgt_adj {{{3
    /// Return values needed by SubBattery::wgt_adj().
    ///
    pub fn wgt_adj(&self) -> f64 {
        match self {
            Self::Single   => 1.0,
            Self::Twin2Row => 1.0,
            Self::Quad4Row => 1.0,
            Self::Twin     => 0.75,
            Self::TwoGun   => 1.0,
            Self::Quad2Row => 1.0,
            Self::Triple   => 0.75,
            Self::ThreeGun => 1.0,
            Self::Sex2Row  => 1.0,
            Self::Quad     => 0.75,
            Self::FourGun  => 1.0,
            Self::Oct2Row  => 1.0,
            Self::Quint    => 0.75,
            Self::FiveGun  => 1.0,
            Self::Dec2Row  => 1.0,
        }
    }
}

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
    pub mount_kind: TorpedoMountType,

    /// Number of torpedoes in the set
    pub num: u32,

    /// Torpedo diameter.
    pub diam: f64,
    /// Torpedo length.
    pub len: f64,
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
            PI * self.diam.powf(2.0) * self.len /
            (
                (f64::max(1907.0 - self.year as f64, 0.0) + 25.0) * 937.0
            ) + (self.year as f64 - 1890.0) * 0.004
        ) * self.num as f64
    }

    // wgt_mounts {{{3
    /// Weight of mounts in the set.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.mount_kind.wgt_factor() * self.wgt_weaps()
    }

    // hull_space {{{3
    /// Hull space taken up by the set.
    ///
    pub fn hull_space(&self) -> f64 {
        self.mount_kind.hull_space(self.len, self.diam) * self.num as f64
    }

    // deck_space {{{3
    /// Deck space taken up by the set.
    ///
    pub fn deck_space(&self, b: f64) -> f64 {
        self.mount_kind.deck_space(b, self.num, self.len, self.diam, self.mounts)
    }
}

// TorpedoMountType {{{1
/// Type of torpedo mount.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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

impl From<String> for TorpedoMountType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for TorpedoMountType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::DeckSideTubes,
            "2" => Self::CenterTubes,
            "3" => Self::DeckReloads,
            "4" => Self::BowTubes,
            "5" => Self::SternTubes,
            "6" => Self::BowAndSternTubes,
            "7" => Self::SubmergedSideTubes,
            "8" => Self::SubmergedReloads,
            "0" | _ => Self::FixedTubes,
        }
    }
}

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
        use std::f64::consts::PI;

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
                let x = f64::powf(len,2.0);
                let y = f64::powf((num/mounts)*diam/12.0 + (num/mounts-1.0)*0.5, 2.0);

                f64::sqrt(x + y)*b * mounts
            },

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
            Self::DeckReloads =>
                if tubes > 1 {
                    format!("In {} sets of ", mounts)
                } else {
                    "In a ".into()
                },

            _ => "".into(),
        };

        prefix + desc + if tubes > 1 { "s" } else { "" }
    }
}

// Testing Torpedo MountType {{{2
#[cfg(test)]
mod torpedo_mount_type {
    use super::*;
    use crate::test_support::*;

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
}

// Mines {{{1
/// Mines and deployement gear.
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
    pub wgt: f64,

    /// Type of mine deployment system.
    pub mount_kind: MineType,
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
        (self.num + self.reload) as f64 * self.wgt / Ship::POUND2TON
    }

    // wgt_mounts {{{3
    /// Weight of deployment gear.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.wgt_weaps() * self.mount_kind.wgt_factor()
    }
}

// MineType {{{1
/// Types of mine deployment gear.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum MineType {
    #[default]
    SternRails,
    BowTubes,
    SternTubes,
    SideTubes,
}

impl From<String> for MineType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for MineType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::BowTubes,
            "2" => Self::SternTubes,
            "3" => Self::SideTubes,
            "0" | _ => Self::SternRails,
        }
    }
}

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
    pub wgt: f64,

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
        (self.num + self.reload) as f64 * self.wgt / Ship::POUND2TON
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
mod weapons {
    use super::*;
    use crate::test_support::*;

    // Test mines_wgt_weaps {{{3
    macro_rules! test_mines_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.mount_kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = wgt;

                    assert!(expected == to_place(mines.wgt_weaps(), 3));
                }
            )*
        }
    }
    test_mines_wgt_weaps! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_weaps_mines_stern_rails: (0.893, MineType::SternRails, 100, 100, 10.0),
        wgt_weaps_mines_bow_tubes:   (0.893, MineType::BowTubes, 100, 100, 10.0),
        wgt_weaps_mines_stern_tubes: (0.893, MineType::SternTubes, 100, 100, 10.0),
        wgt_weaps_mines_side_tubes:  (0.893, MineType::SideTubes, 100, 100, 10.0),
    }

    // Test mines_wgt_mounts {{{3
    macro_rules! test_mines_wgt_mounts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.mount_kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = wgt;

                    assert!(expected == to_place(mines.wgt_mounts(), 3));
                }
            )*
        }
    }
    test_mines_wgt_mounts! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_mounts_mines_stern_rails: (0.223, MineType::SternRails, 100, 100, 10.0),
        wgt_mounts_mines_bow_tubes:   (0.893, MineType::BowTubes, 100, 100, 10.0),
        wgt_mounts_mines_stern_tubes: (0.893, MineType::SternTubes, 100, 100, 10.0),
        wgt_mounts_mines_side_tubes:  (0.893, MineType::SideTubes, 100, 100, 10.0),
    }

    // Test mines_wgt {{{3
    macro_rules! test_mines_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut mines = Mines::default();
                    mines.mount_kind = kind;
                    mines.num = num;
                    mines.reload = reload;
                    mines.wgt = wgt;

                    assert!(expected == to_place(mines.wgt(), 3));
                }
            )*
        }
    }
    test_mines_wgt! {
        // name:                    (expected, kind, num, reload, wgt)
        wgt_mines_stern_rails: (1.116, MineType::SternRails, 100, 100, 10.0),
        wgt_mines_bow_tubes:   (1.786, MineType::BowTubes, 100, 100, 10.0),
        wgt_mines_stern_tubes: (1.786, MineType::SternTubes, 100, 100, 10.0),
        wgt_mines_side_tubes:  (1.786, MineType::SideTubes, 100, 100, 10.0),
    }

    // Test asw_wgt_weaps {{{3
    macro_rules! test_asw_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, num, reload, wgt) = $value;

                    let mut asw = ASW::default();
                    asw.kind = kind; asw.num = num; asw.reload = reload; asw.wgt = wgt;

                    assert!(expected == to_place(asw.wgt_weaps(), 3));
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
                    asw.kind = kind; asw.num = num; asw.reload = reload; asw.wgt = wgt;

                    assert!(expected == to_place(asw.wgt_mounts(), 3));
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
                    asw.kind = kind; asw.num = num; asw.reload = reload; asw.wgt = wgt;

                    assert!(expected == to_place(asw.wgt(), 3));
                }
            )*
        }
    }
    test_asw_wgt! {
        // name:                      (expected, kind, num, reload, wgt)
        wgt_asw_stern_racks:   (1.116, ASWType::SternRacks, 100, 100, 10.0),
        wgt_asw_throwers:      (1.339, ASWType::Throwers, 100, 100, 10.0),
        wgt_asw_hedgehogs:     (1.339, ASWType::Hedgehogs, 100, 100, 10.0),
        wgt_asw_squid_mortars: (9.821, ASWType::SquidMortars, 100, 100, 10.0),
    }

    // Test torpedo_wgt_weaps {{{3
    macro_rules! test_torpedo_wgt_weaps {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.mount_kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.year = year;

                    assert!(expected == to_place(torp.wgt_weaps(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt_weaps! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_weaps_torps_fixed_tubes:         (4.450, TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_weaps_torps_deck_side_tubes:     (4.450, TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_weaps_torps_center_tubes:        (4.450, TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_weaps_torps_deck_reloads:        (4.450, TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_weaps_torps_bow_tubes:           (4.450, TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_weaps_torps_stern_tubes:         (4.450, TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_weaps_torps_bow_and_stern_tubes: (4.450, TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_weaps_torps_submerged_tubes:     (4.450, TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_weaps_torps_submerged_reloads:   (4.450, TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_wgt_mounts {{{3
    macro_rules! test_torpedo_wgt_mounts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.mount_kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.year = year;

                    assert!(expected == to_place(torp.wgt_mounts(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt_mounts! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_mounts_torps_fixed_tubes:         (1.113, TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_mounts_torps_deck_side_tubes:     (4.450, TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_mounts_torps_center_tubes:        (4.450, TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_mounts_torps_deck_reloads:        (1.113, TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_mounts_torps_bow_tubes:           (4.450, TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_mounts_torps_stern_tubes:         (4.450, TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_mounts_torps_bow_and_stern_tubes: (4.450, TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_mounts_torps_submerged_tubes:     (4.450, TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_mounts_torps_submerged_reloads:   (1.113, TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_wgt {{{3
    macro_rules! test_torpedo_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num, year) = $value;

                    let mut torp = Torpedoes::default();
                    torp.mount_kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.year = year;

                    assert!(expected == to_place(torp.wgt(), 3));
                }
            )*
        }
    }
    test_torpedo_wgt! {
        // name:                       (wgt, kind, diam, len, num, year)
        wgt_torps_fixed_tubes:         (5.563, TorpedoMountType::FixedTubes,         18.0, 21.0, 4, 1940),
        wgt_torps_deck_side_tubes:     (8.900, TorpedoMountType::DeckSideTubes,      18.0, 21.0, 4, 1940),
        wgt_torps_center_tubes:        (8.900, TorpedoMountType::CenterTubes,        18.0, 21.0, 4, 1940),
        wgt_torps_deck_reloads:        (5.563, TorpedoMountType::DeckReloads,        18.0, 21.0, 4, 1940),
        wgt_torps_bow_tubes:           (8.900, TorpedoMountType::BowTubes,           18.0, 21.0, 4, 1940),
        wgt_torps_stern_tubes:         (8.900, TorpedoMountType::SternTubes,         18.0, 21.0, 4, 1940),
        wgt_torps_bow_and_stern_tubes: (8.900, TorpedoMountType::BowAndSternTubes,   18.0, 21.0, 4, 1940),
        wgt_torps_submerged_tubes:     (8.900, TorpedoMountType::SubmergedSideTubes, 18.0, 21.0, 4, 1940),
        wgt_torps_submerged_reloads:   (5.563, TorpedoMountType::SubmergedReloads,   18.0, 21.0, 4, 1940),
    }

    // Test torpedo_hull_space {{{3
    macro_rules! test_torpedo_hull_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind, diam, len, num) = $value;
                    let mut torp = Torpedoes::default();
                    torp.mount_kind = kind; torp.diam = diam; torp.len = len; torp.num = num;

                    assert!(expected == to_place(torp.hull_space(), 3));
                }
            )*
        }
    }
    test_torpedo_hull_space! {
        // name:                             (space, kind, diam, len, num, year)
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
                    torp.mount_kind = kind; torp.diam = diam; torp.len = len; torp.num = num; torp.mounts = mounts;

                    let b = 10.0;
                    assert!(expected == to_place(torp.deck_space(b), 3));
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ASWType {
    #[default]
    SternRacks,
    Throwers,
    Hedgehogs,
    SquidMortars,
}

impl From<String> for ASWType { // {{{2
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for ASWType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::Throwers,
            "2" => Self::Hedgehogs,
            "3" => Self::SquidMortars,
            "0" | _ => Self::SternRacks,
        }
    }
}

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
}

