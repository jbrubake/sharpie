use crate::{Ship, GunType, MountType, GunDistributionType, GunLayoutType, MineType, ASWType, TorpedoMountType, Armor};
use crate::Hull;
use crate::unit_types::Units;
use serde::{Serialize, Deserialize};
use std::f64::consts::PI;

// SubBattery {{{1
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SubBattery {
    /// Gun layout.
    pub layout: GunLayoutType,
    /// Distribution of guns on the ship.
    pub distribution: GunDistributionType,
    /// Number of mounts above the waterline.
    pub above: u32,
    /// Number of mounts on the waterline.
    pub on: u32,
    /// Number of mounts below the waterline.
    pub below: u32,
    /// If mounts above the deck are two mounts up
    pub two_mounts_up: bool,
    /// If mounts below the waterline are on the lower deck
    pub lower_deck: bool,
}

impl SubBattery { // Internals Output {{{1
    pub fn internals(&self, hull: Hull, cal: f64) -> () {
        eprintln!("layout = {}", self.layout);
        eprintln!("distribution = {}", self.distribution);
        eprintln!("above = {}", self.above);
        eprintln!("on = {}", self.on);
        eprintln!("below = {}", self.below);
        eprintln!("two_mounts_up = {}", self.two_mounts_up);
        eprintln!("lower_deck = {}", self.lower_deck);
        eprintln!("super_() = {}", self.super_());
        eprintln!("num_mounts() = {}", self.num_mounts());
        eprintln!("diameter_calc() = {}", self.diameter_calc(cal));
        eprintln!("wgt_adj() = {}", self.wgt_adj());
        eprintln!("free() = {}", self.free(hull.clone()));
        eprintln!("");
    }
}

impl SubBattery { // {{{1
    // super_ {{{2
    /// XXX: ???
    ///
    pub fn super_(&self) -> i32 {
        let above: i32 = (self.above * if self.two_mounts_up { 2 } else { 1 }) as i32;
        let below: i32 = (self.below * if self.lower_deck    { 2 } else { 1 }) as i32;

        (above - below) * self.layout.guns_per() as i32
    }

    // num_mounts {{{2
    /// Total number of gun mounts.
    ///
    pub fn num_mounts(&self) -> u32 {
        self.above + self.on + self.below
    }

    // diameter_calc {{{2
    /// XXX: ???
    ///
    pub fn diameter_calc(&self, cal: f64) -> f64 {
        if cal == 0.0 { return 0.0; }

        let (n1, n2) = self.layout.diameter_calc_nums();

        let mut calc = n1 * cal * (1.0 + (1.0/cal).powf(n2));

        if cal < 12.0                               { calc += 12.0 / cal; }
        if cal > 1.0 && self.layout.wgt_adj() < 1.0 { calc *= 0.9; }

        calc
    }

    // wgt_adj {{{2
    /// XXX: ???
    ///
    pub fn wgt_adj(&self) -> f64 {
        self.layout.wgt_adj() * self.num_mounts() as f64
    }

    // free {{{2
    /// XXX: ???
    ///
    pub fn free(&self, hull: Hull) -> f64 {
        let free = self.distribution.free(self.num_mounts(), hull);

        free * self.num_mounts() as f64
    }
}

#[cfg(test)] // {{{1
mod sub_battery {
    use super::*;
    use crate::test_support::*;

    // Test super_ {{{2
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

    // Test diameter_calc {{{2
    macro_rules! test_diameter_calc {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cal) = $value;

                    let mut sub_btry = SubBattery::default();
                    sub_btry.layout = GunLayoutType::Single;

                    assert!(expected == to_place(sub_btry.diameter_calc(cal), 2));
                }
            )*
        }
    }
    test_diameter_calc! {
        // name:      (diameter_calc, cal)
        diameter_calc_cal_eq_0: (0.0, 0.0),
        diameter_calc_cal_lt_12: (19.14, 10.0),
        diameter_calc_cal_gt_1:  (12.30, 5.0),
        diameter_calc_cal_sm:  (25.82, 0.5),
    }

    // Test wgt_adj {{{2
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

    // Test free {{{2
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

// Battery {{{1
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Battery {
    /// Units
    pub units: Units,
    /// Number of guns in the battery.
    pub num: u32,
    // XXX: The naming of 'cal' and 'len' is unfortunate and confusing
    // XXX: I will change them to make more sense once all the formulas are verified
    /// Gun barrel diameter in inches.
    pub cal: f64,
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
    // XXX: top, side, rear?
    pub armor_back: f64,
    /// Armor thickness on barbette.
    pub armor_barb: f64,

    /// Sub-batteries to position the battery mounts.
    pub groups: Vec<SubBattery>,
}

impl Default for Battery { // {{{1
    fn default() -> Self {
        Self {
            units: Units::Imperial, 

            num: 0,
            cal: 0.0,
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

impl Battery { // Internals Output {{{1
    pub fn internals(&self, hull: Hull, wgt_broad: f64) -> () {
        eprintln!("units = {}", self.units);
        eprintln!("num = {}", self.num);
        eprintln!("cal = {}", self.cal);
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
            g.internals(hull.clone(), self.cal);
        }
    }
}

impl Battery { // {{{1
    /// XXX: ???
    ///
    const CORDITE_FACTOR: f64 = 0.2444444;

    // broad_and_below {{{2
    /// XXX: ???
    pub fn broad_and_below(&self) -> bool {
        if self.mount_kind == MountType::Broadside {
            for g in self.groups.iter() {
                if g.below != 0 { return true; }
            }
        }
        false
    }

    // concentration {{{2
    pub fn concentration(&self, wgt_broad: f64) -> f64 {
        if self.num as f64 * self.shell_wgt() == 0.0 ||
            self.mount_num == 0
            { return 0.0; }

        (self.shell_wgt() * self.num as f64 / wgt_broad) *
            if self.mount_kind.wgt_adj() > 0.6 {
                (4.0 / self.mount_num as f64).powf(0.25) - 1.0
            } else {
                -0.1
            }
    }

    // super_ {{{2
    /// XXX: ???
    ///
    pub fn super_(&self, hull: Hull) -> f64 {

        if self.num == 0 { return 0.0 } // catch divide by zero

        let mut super_ = 0;
        for g in self.groups.iter() {
            super_ += g.super_()
        }

        let free = self.free(hull);

        if free == 0.0 { return 0.0 } // catch divide by zero

        ((super_ as f64 / self.num as f64) * (self.cal * 0.6).max(7.5) + free) / free
    }

    // free {{{2
    /// XXX: ???
    ///
    pub fn free(&self, hull: Hull) -> f64 {

        let mut f = 0.0;
        let mut mounts = 0;
        for b in self.groups.iter() {
            f += b.free(hull.clone());
            mounts += b.num_mounts();
        }

        f / mounts as f64
    }

    // armor_face_wgt {{{2
    /// Weight of face armor
    ///
    pub fn armor_face_wgt(&self) -> f64 {
        // TODO: Combine this logic into a single table
        let wgt = self.mount_kind.armor_face_wgt() +
            if self.armor_back == 0.0 {
                self.mount_kind.armor_face_wgt_if_no_back()
            } else { 0.0 };

        let mut diameter_calc = 0.0;
        for g in self.groups.iter() {
            diameter_calc += g.diameter_calc(self.cal) * g.num_mounts() as f64;
        }

        let wgt = wgt * diameter_calc * self.house_hgt() * self.armor_face * Armor::INCH;

        // TODO: Combine this logic into a single table
        wgt * self.kind.armor_face_wgt() * 
            if self.armor_back == 0.0 {
                self.kind.armor_face_wgt_if_no_back()
            } else { 1.0 }
    }

    // house_hgt {{{2
    fn house_hgt(&self) -> f64 {
        f64::max(
            7.5,
            0.625 * self.cal * self.mount_kind.gunhouse_hgt_factor(),
        )
    }

    // armor_back_wgt {{{2
    /// Weight of back armor
    ///
    pub fn armor_back_wgt(&self) -> f64 {
        let mut a = 0.0;
        for g in self.groups.iter() {
            a += PI * (g.diameter_calc(self.cal) / 2.0).powf(2.0) * g.num_mounts() as f64;
        }

        let b = self.mount_kind.armor_back_wgt();

        let mut diameter_calc = 0.0;
        for g in self.groups.iter() {
            diameter_calc += g.diameter_calc(self.cal) * g.num_mounts() as f64;
        }
        let c = b * diameter_calc * self.house_hgt();

        let d = c + a * self.mount_kind.armor_back_wgt_factor();

        d * self.armor_back * Armor::INCH
    }
    // armor_barb_wgt {{{2
    /// Weight of barbette armor
    ///
    pub fn armor_barb_wgt(&self, hull: Hull) -> f64 {
        let mut guns = 0;
        let mut mounts = 0;
        for g in self.groups.iter() {
            guns += g.layout.guns_per() * g.num_mounts();
            mounts += g.num_mounts();
        }

        if mounts == 0 { return 0.0; } // catch divide by zero

        let a = u32::min(
            if self.mount_kind.wgt_adj() > 0.5 { 4 } else { 5 },
            guns / mounts,
        );

        let b = self.mount_kind.armor_barb_wgt();

        if self.free(hull.clone()) <= 0.0 {
            0.0
        } else {
            (1.0 - (a as f64 - 2.0) / 6.0) *
                self.armor_barb *
                 self.num as f64 *
                 self.cal.powf(1.2) *
                 b *
                 self.free(hull.clone()) / 16.0 *
                 self.super_(hull.clone()) *
                 b *
                 2.0 *
                 self.date_factor().sqrt()
                 
        }
    }
    // armor_wgt {{{2
    /// Weight of the battery's armor
    ///
    pub fn armor_wgt(&self, hull: Hull) -> f64 {
        self.armor_face_wgt() + self.armor_back_wgt() + self.armor_barb_wgt(hull)
    }

    // wgt_adj {{{2
    /// XXX: ???
    ///
    pub fn wgt_adj(&self) -> f64 {
        let mut v = 0.0;
        let mut mounts = 0;
        for b in self.groups.iter() {
            v += b.wgt_adj();
            mounts += b.num_mounts();
        }

        if mounts == 0 { return 0.0; }

        v / mounts as f64
    }

    // date_factor {{{2
    /// Factor to adjust shell weight by year.
    ///
    fn date_factor(&self) -> f64 {
        Ship::year_adj(self.year).sqrt()
    }

    // set_shell_wgt {{{2
    /// Override the default shell weight calculation.
    ///
    pub fn set_shell_wgt(&mut self, wgt: f64) -> f64 {
        self.shell_wgt = Some(wgt);
        
        wgt
    }

    // shell_wgt {{{2
    /// Get the shell weight.
    ///
    /// Return the value set previously be set_shell_wgt() or the default if
    /// unset.
    ///
    pub fn shell_wgt(&self) -> f64 {
        match self.shell_wgt {
            Some(wgt) => wgt,
            None      => self.shell_wgt_est(),
        }
    }

    // shell_wgt_est {{{2
    /// Estimated shell weight.
    ///
    pub fn shell_wgt_est(&self) -> f64 {
        self.cal.powf(3.0) / 1.9830943211886 * self.date_factor() *
            ( 1.0 + if self.len < 45.0 { -1.0 } else { 1.0 } * (45.0 - self.len).abs().sqrt() / 45.0 )
    }

    // gun_wgt {{{2
    /// XXX: Weight of something related to estimated shell weight, length and number of guns
    ///
    pub fn gun_wgt(&self) -> f64 {
        if self.cal == 0.0 { return 0.0; }

        self.shell_wgt_est() * (self.len as f64 / 812.289434917877 *
            (1.0 + (1.0 / self.cal as f64).powf(2.3297949327695))
            ) * self.num as f64
    }

    // mount_wgt {{{2
    /// Weight of a single gun mount.
    ///
    pub fn mount_wgt(&self) -> f64 {
        if self.cal == 0.0 { return 0.0; }

        let wgt = self.mount_kind.wgt() *
            if self.mount_kind.wgt_adj() < 0.6 {
                self.kind.wgt_sm()
            } else {
                self.kind.wgt_lg()
            };

        let wgt = (wgt + 1.0 / self.cal.powf(0.313068808543972)) * self.gun_wgt();

        let wgt =
            if self.cal > 10.0 {
                wgt * (1.0 - 2.1623769 * self.cal / 100.0)
            } else if self.cal <= 1.0 {
                self.gun_wgt()
            } else {
                wgt
            };

        wgt * self.wgt_adj()
    }

    // broadside_wgt {{{2
    /// Weight of shells fired by the battery.
    ///
    pub fn broadside_wgt(&self) -> f64 {
        self.num as f64 * self.shell_wgt()
    }

    // mag_wgt {{{2
    /// Weight of the battery magazine
    ///
    pub fn mag_wgt(&self) -> f64 {
        (self.num * self.shells) as f64 * self.shell_wgt() / Ship::POUND2TON * (1.0 + Self::CORDITE_FACTOR)
    }

    // new {{{2
    pub fn new() -> Battery {
        Default::default()
    }
}

#[cfg(test)] // {{{1
mod battery {
    use super::*;
    use crate::test_support::*;

    // Test broad_and_below {{{2
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

    // Test concentration {{{2
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

    // Test super_ {{{2
    macro_rules! test_super_ {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, group_1_mounts, group_2_mounts) = $value;

                    let mut btry = Battery::default();

                    btry.num = group_1_mounts + group_2_mounts;

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
        // name: (super_, shell_wgt, mount_kind, mount_num)
        super_test_1: (1.3, 2, 5),
        super_test_2: (1.75, 5, 2),
    }

    // Test free {{{2
    macro_rules! test_free {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, group_1_mounts, group_2_mounts) = $value;

                    let mut btry = Battery::default();

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

    // Test armor_face_wgt {{{2
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
                    btry.cal = 10.0;

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

    // Test house_hgt {{{2
    macro_rules! test_house_hgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cal) = $value;

                    let mut btry = Battery::default();
                    btry.cal = cal;
                    btry.mount_kind = MountType::Broadside;

                    assert!(expected == to_place(btry.house_hgt(), 5));
                }
            )*
        }
    }
    test_house_hgt! {
        // name: (house_hgt, cal)
        house_hgt_1: (8.75, 14.0),
        house_hgt_2: (7.5, 10.0),
    }

    // Test armor_back_wgt {{{2
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
                    btry.cal = 10.0;

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

    // Test armor_barb_wgt {{{2
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
                    btry.cal = 10.0;
                    btry.year = 1920;
                    btry.num = 2;

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

    // Test wgt_adj {{{2
    macro_rules! test_wgt_adj {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, g0_mounts, g1_mounts) = $value;

                    let mut btry = Battery::default();
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

    // Test date_factor {{{2
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

    // Test shell_wgt_est {{{2
    macro_rules! test_shell_wgt_est {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, len) = $value;

                    let mut btry = Battery::default();
                    btry.len = len;
                    btry.cal = 10.0;
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

    // Test gun_wgt {{{2
    macro_rules! test_gun_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cal, len) = $value;

                    let mut btry = Battery::default();
                    btry.len = len;
                    btry.cal = cal;
                    btry.num = 1;
                    btry.year = 1920;

                    assert!(expected == to_place(btry.gun_wgt(), 2));
                }
            )*
        }
    }
    test_gun_wgt! {
        // name: (gun_wgt, cal, len)
        gun_wgt_cal_eq_0: (0.0, 0.0, 0.0),
        gun_wgt_test: (28.07, 10.0, 45.0),
    }

    // Test mount_wgt {{{2
    macro_rules! test_mount_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount_kind, cal) = $value;

                    let mut btry = Battery::default();
                    btry.mount_kind = mount_kind;
                    btry.cal = cal;
                    btry.len = 45.0;
                    btry.num = 1;
                    btry.year = 1920;
                    btry.kind = GunType::AntiAir;

                    btry.groups[0].on = 1;
                    btry.groups[1].on = 0;
                    btry.groups[0].layout = GunLayoutType::Single;
                    btry.groups[1].layout = GunLayoutType::Single;

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

    // Test broadside_wgt {{{2
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

    // Test mag_wgt {{{2
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

// Torpedoes {{{1
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Torpedoes {
    /// Units
    pub units: Units,
    /// Year torpedo was designed.
    pub year: u32,
    /// Number of torpedoes in a mount.
    pub num: u32,
    /// Number of mounts.
    pub mounts: u32,
    /// Torpedo diameter.
    pub diam: f64,
    /// Torpedo length.
    pub len: f64,
    /// Type of mount.
    pub mount_kind: TorpedoMountType,
}

impl Torpedoes {
    // new {{{2
    pub fn new() -> Torpedoes {
        Default::default()
    }

    // wgt {{{2
    /// Weight of torpedoes and mounts in the set.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    // wgt_weaps {{{2
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

    // wgt_mounts {{{2
    /// Weight of mounts in the set.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.mount_kind.wgt_factor() * self.wgt_weaps()
    }

    // hull_space {{{2
    /// Hull space taken up by the torpedo set.
    ///
    pub fn hull_space(&self) -> f64 {
        self.mount_kind.hull_space(self.len, self.diam) * self.num as f64
    }

    // deck_space {{{2
    /// Deck space taken up by the torpedo set.
    ///
    pub fn deck_space(&self, b: f64) -> f64 {
        self.mount_kind.deck_space(b, self.num, self.len, self.diam, self.mounts)
    }
}

// Mines {{{1
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

impl Mines {
    // new {{{2
    pub fn new() -> Mines {
        Default::default()
    }

    // wgt {{{2
    /// Weight of mines, reloads and deployment system.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    pub fn wgt_weaps(&self) -> f64 {
        (self.num + self.reload) as f64 * self.wgt / Ship::POUND2TON
    }

    pub fn wgt_mounts(&self) -> f64 {
        self.wgt_weaps() * self.mount_kind.wgt_factor()
    }
}

// ASW {{{1
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

impl ASW {
    // new {{{2
    pub fn new() -> ASW {
        Default::default()
    }

    // wgt {{{2
    /// Weight of weapons, reloads and mounts.
    ///
    pub fn wgt(&self) -> f64 {
        self.wgt_weaps() + self.wgt_mounts()
    }

    // wgt_weaps {{{2
    /// Weight of weapons and reloads.
    ///
    pub fn wgt_weaps(&self) -> f64 {
        (self.num + self.reload) as f64 * self.wgt / Ship::POUND2TON
    }

    // wgt_mounts {{{2
    /// Weight of mounts.
    ///
    pub fn wgt_mounts(&self) -> f64 {
        self.wgt_weaps() * self.kind.mount_wgt_factor()
    }
}

#[cfg(test)] // {{{1
mod weapons {
    use super::*;
    use crate::test_support::*;

    // Test mines_wgt_weaps {{{2
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

    // Test mines_wgt_mounts {{{2
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

    // Test mines_wgt {{{2
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

    // Test asw_wgt_weaps {{{2
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

    // Test asw_wgt_mounts {{{2
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

    // Test asw_wgt {{{2
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

    // Test torpedo_wgt_weaps {{{2
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

    // Test torpedo_wgt_mounts {{{2
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

    // Test torpedo_wgt {{{2
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

    // Test torpedo_hull_space {{{2
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

    // Test torpedo_deck_space {{{2
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

