use serde::{Serialize, Deserialize};

use std::f64::consts::PI;

use crate::SternType;
use crate::BowType;

// Hull {{{1
#[derive(Serialize, Deserialize, Debug)]
pub struct Hull {
        cb: Option<f64>,
        d: Option<f64>,
        loa: Option<f64>,
        lwl: Option<f64>,

    pub b: f64,
    pub bb: f64,
    pub t: f64,

    // in SS3, if # shafts is 0 or 1 AND Cb ≥ 0.75 then
    // cwp is calculated differently
    //
    // Because shafts should not be set in Hull, we use this for the same purpose
    // if # shafts < 2 then self.boxy should be true, otherwise it should be false
    pub boxy: bool,

    pub bow_type: BowType,
    pub stern_type: SternType,
    pub stern_overhang: f64,

    pub fc_len: f64, pub fc_fwd: f64, pub fc_aft: f64,
    pub fd_len: f64, pub fd_fwd: f64, pub fd_aft: f64,
                     pub ad_fwd: f64, pub ad_aft: f64,
    pub qd_len: f64, pub qd_fwd: f64, pub qd_aft: f64,

    pub bow_angle: f64,
}

impl Default for Hull { // {{{1
    fn default() -> Hull {
        Hull {
            cb: None,
            d: None,
            lwl: None,
            loa: None,

            b: 0.0,
            bb: 0.0,
            t: 0.0,

            boxy: false,

            bow_type: BowType::Normal,
            stern_type: SternType::Cruiser,
            stern_overhang: 0.0,

            fc_len: 0.0, fc_fwd: 0.0, fc_aft: 0.0,
            fd_len: 0.0, fd_fwd: 0.0, fd_aft: 0.0,
                         ad_fwd: 0.0, ad_aft: 0.0,
            qd_len: 0.0, qd_fwd: 0.0, qd_aft: 0.0,

            bow_angle: 0.0,
        }
    }
}

// Hull Implementation {{{1
impl Hull {
    pub const FT3_PER_TON_SEA: f64 = 35.0; // Volume of one long ton of seawater in ft³

    // new {{{2
    /// Create a new ship
    ///
    pub fn new() -> Hull {
        Default::default()
    }
    // cs {{{2
    /// Calculate Coefficient of Sharpness
    ///
    pub fn cs(&self) -> f64 {
        if self.lwl() == 0.0 { return 0.0; }
        0.4 * (self.bb / self.lwl() * 6.0).powf(1.0/3.0) * f64::sqrt(self.cb() / 0.52)
    }

    // cm {{{2
    /// Calculate Midships Coefficient
    ///
    pub fn cm(block: f64) -> f64 {
        // The float math doesn't work out if block == 0.0
        if block == 0.0 { return 1.006; }
        1.006 - 0.0056 * block.powf(-3.56)
    }

    // cp {{{2
    /// Calculate Prismatic Coefficient
    ///
    pub fn cp(block: f64) -> f64 {
        block / Hull::cm(block)
    }

    // cb_calc {{{2
    /// Calculate Cb for given displacment
    ///
    pub fn cb_calc(&self, d: f64) -> f64 {
        let volume = self.lwl() * self.bb * self.t;

        if volume == 0.0 {
            0.0
        } else {
            (d * Self::FT3_PER_TON_SEA / volume).min(1.0).max(0.0)
        }
    }

    // cb {{{2
    /// Calculate Block Coefficient
    ///
    /// Range: [0.0, 1.0]
    pub fn cb(&self) -> f64 {
        match self.cb {
            Some(cb) => cb,
            None     => self.cb_calc(self.d()),
        }
    }

    // set_d {{{2
    pub fn set_d(&mut self, d: f64) -> f64 {
        self.d = Some(d);
        self.cb = None;
        self.d.unwrap()
    }

    // set_cb {{{2
    pub fn set_cb(&mut self, cb: f64) -> f64 {
        self.cb = Some(cb);
        self.d = None;
        self.cb.unwrap()
    }

    // d {{{2
    pub fn d(&self) -> f64 {
        match self.d {
            Some(d) => d,
            None    => self.cb() * self.lwl() * self.bb * self.t / Self::FT3_PER_TON_SEA
        }
    }

    // cwp {{{2
    /// Calculate Waterplane Area Coefficient
    ///
    pub fn cwp(&self) -> f64 {
        let mut a: f64 = 0.262;
        let mut f: f64 = 0.76;

        if let SternType::TransomSm = self.stern_type {
            f = 0.79;
        } else if let SternType::TransomLg = self.stern_type {
            f = 0.81;
        } else if self.boxy || self.cb() >= 0.75 {
            a = 0.175;
            f = 0.875;
        }
        
        let cwp = a + f * Hull::cp(self.cb());
        if self.cb() < 0.4 {
            cwp - 0.0281 - (self.cb() - 0.3).powf(1.55)
        } else {
            cwp
        }
    }

    // wp {{{2
    /// Calculate Waterplane Area
    ///
    pub fn wp(&self) -> f64 {
        self.cwp() * self.lwl() * self.b
    }

    // ws {{{2
    /// Calculate Wetted Surface Area
    ///
    pub fn ws(&self) -> f64 {
        if self.t == 0.0 { return 0.0; }
        self.lwl() * self.t * 1.7 + (self.d() * Self::FT3_PER_TON_SEA / self.t)
    }

    // set_lwl {{{2
    pub fn set_lwl(&mut self, len: f64) -> f64 {
        self.lwl = Some(len);
        self.loa = None;
        self.lwl.unwrap()
    }

    // set_loa {{{2
    pub fn set_loa(&mut self, len: f64) -> f64 {
        self.loa = Some(len);
        self.lwl = None;
        self.loa.unwrap()
    }

    // lwl {{{2
    /// Calculate Waterline Length
    ///
    /// lwl = loa - stern_overhang - max(ram_length, length_from_bow_angle)
    ///
    /// Range: [loa, ∞)
    pub fn lwl(&self) -> f64 {
        match self.lwl {
            Some(len) => len,
            None    => {
                let stem = match self.bow_type {
                    BowType::Ram(len) => len,
                    _                 => 0.0,
                };
                self.loa.unwrap() - stem.max(self.stem_len()).max(0.0) - self.stern_overhang.max(0.0)
            }
        }
    }
    // loa {{{2
    /// Calculate Overall Length
    ///
    /// loa = lwl + stern_overhang + max(ram_length, length_from_bow_angle)
    ///
    /// Range: [lwl, ∞)
    pub fn loa(&self) -> f64 {
        match self.loa {
            Some(len) => len,
            None    => {
                let stem = match self.bow_type {
                    BowType::Ram(len) => len,
                    _                 => 0.0,
                };
                self.lwl.unwrap() + stem.max(self.stem_len()).max(0.0) + self.stern_overhang.max(0.0)
            }
        }
    }

    // leff {{{2
    /// Calculate effective length
    ///
    pub fn leff(&self) -> f64 {
        match self.stern_type {
            SternType::TransomSm => self.bb * 0.5 / self.cs() + self.lwl(),
            SternType::TransomLg => self.bb / self.cs() + self.lwl(),
            _                    => self.lwl(),
        }
    }

    // t_calc {{{2
    /// Calculate draft at displacment
    ///
    pub fn t_calc(&self, d: f64) -> f64 {
        self.t + (d - self.d()) / (self.wp() * Hull::FT3_PER_TON_SEA)
    }

    // ts {{{2
    /// Calculate ts
    ///
    pub fn ts(&self) -> f64 {
        (Hull::cm(self.cb()) * 2.0 - 1.0) * self.t
    }

    // ad_len {{{2
    /// Calculate ad_len
    ///
    pub fn ad_len(&self) -> f64 {
        if self.fc_len + self.fd_len + self.qd_len > 1.0 {
            return 0.0;
        }
        1.0 - self.fc_len - self.fd_len - self.qd_len
    }

    // stem_len {{{2
    /// Calculate stem increase or decrease from bow angle
    ///
    /// Range: (-∞, +∞)
    pub fn stem_len(&self) -> f64 {
        if self.bow_angle.abs() == 90.0 {
            0.0
        } else {
            self.fc_fwd * f64::tan(self.bow_angle * PI / 100.0)
        }
    }

    // freeboard {{{2
    /// Calculate average freeboard
    ///
    pub fn freeboard(&self) -> f64 {
        if self.fc_len + self.fd_len + self.qd_len > 1.0 {
            return 0.0;
        }

        let fc = self.fc_aft + (self.fc_fwd - self.fc_aft) * 0.4;
        let fd = self.fd_fwd + (self.fd_aft - self.fd_fwd) * 0.5;
        let ad = self.ad_fwd + (self.ad_aft - self.ad_fwd) * 0.5;
        let qd = self.qd_fwd + (self.qd_aft - self.qd_fwd) * 0.5;

        fc * self.fc_len +
        fd * self.fd_len +
        ad * self.ad_len() +
        qd * self.qd_len
    }

    // free_cap {{{2
    pub fn free_cap(&self, b: f64, cap_calc_broadside: bool) -> f64 {
        if self.freeboard() > (b/3.0) {
            self.freeboard().powf(2.0) * 3.0 / b
        } else if cap_calc_broadside {
            self.freeboard() - 6.0
        } else {
            self.freeboard()
        }
    }

    // dist {{{2
    /// Calculate freeboard dist
    ///
    /// Range: [0, ∞)
    pub fn dist(&self) -> f64 {
        if self.fc_len + self.fd_len + self.qd_len > 1.0 {
            return 0.0;
        }

        let fd = self.fd_fwd + (self.fd_aft - self.fd_fwd) * 0.5;
        let ad = self.ad_fwd + (self.ad_aft - self.ad_fwd) * 0.5;

        (fd * self.fd_len + ad * self.ad_len()) / (self.fd_len + self.ad_len())
    }

    // vn {{{2
    /// Calculate natural speed
    ///
    pub fn vn(&self) -> f64 {
        self.leff().sqrt()
    }

    // len2beam {{{2
    /// Calculate length to beam ratio
    ///
    pub fn len2beam(&self) -> f64 {
        if self.bb == 0.0 { return 0.0; }
        self.lwl() / self.bb
    }

}
#[cfg(test)] // Hull {{{1
mod ship {
    use super::*;

    // Test dist {{{2
    macro_rules! test_dist {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected,
                        fc_len,
                        fd_len, fd_fwd, fd_aft,
                                ad_fwd, ad_aft,
                        qd_len) = $value;
                    let mut hull = Hull::default();

                    hull.fc_len = fc_len;

                    hull.fd_len = fd_len;
                    hull.fd_fwd = fd_fwd;
                    hull.fd_aft = fd_aft;

                    hull.ad_fwd = ad_fwd;
                    hull.ad_aft = ad_aft;

                    hull.qd_len = qd_len;

                    assert!(expected == hull.dist());
                }
            )*
        }
    }
    test_dist! {
        //              name: (dist, fc_len, fd_len, fd_fwd, fd_aft, ad_fwd, ad_aft, qd_len)
                dist_too_big: (0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
              dist_fd_factor: (0.5, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0),
              dist_ad_factor: (0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
             dist_fd_ad_only: (7.5, 0.0, 0.5, 10.0, 5.0, 10.0, 5.0, 0.0),
                    dist_all: (7.5, 0.25, 0.25, 10.0, 5.0, 10.0, 5.0, 0.25),
        dist_fd_ad_only_diff: (9.25, 0.0, 0.35, 15.0, 10.0, 10.0, 5.0, 0.0),
               dist_all_diff: (11.0, 0.25, 0.35, 15.0, 10.0, 10.0, 5.0, 0.25),
    }
    // Test freeboard {{{2
    macro_rules! test_freeboard {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected,
                        fc_len, fc_fwd, fc_aft,
                        fd_len, fd_fwd, fd_aft,
                                ad_fwd, ad_aft,
                        qd_len, qd_fwd, qd_aft) = $value;

                    let mut hull = Hull::default();

                    hull.fc_len = fc_len;
                    hull.fc_fwd = fc_fwd;
                    hull.fc_aft = fc_aft;

                    hull.fd_len = fd_len;
                    hull.fd_fwd = fd_fwd;
                    hull.fd_aft = fd_aft;

                    hull.ad_fwd = ad_fwd;
                    hull.ad_aft = ad_aft;

                    hull.qd_len = qd_len;
                    hull.qd_fwd = qd_fwd;
                    hull.qd_aft = qd_aft;

                    assert!(expected == hull.freeboard());
                }
            )*
        }
    }
    test_freeboard! {
        //   name: (freeboard, fc_len, fc_fwd, fc_aft, fd_len, fd_fwd, fd_aft, ad_fwd, ad_aft, qd_len, qd_fwd, qd_aft)
             same: (10.0, 0.25, 10.0, 10.0, 0.25, 10.0, 10.0, 10.0, 10.0, 0.25, 10.0, 10.0),
        different: (5.275, 0.25, 10.0, 1.0, 0.25, 10.0, 1.0, 10.0, 1.0, 0.25, 10.0, 1.0),
          too_big: (0.0, 0.5, 10.0, 10.0, 0.5, 10.0, 10.0, 10.0, 10.0, 0.5, 10.0, 10.0),
        fc_factor: (0.4, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        fd_factor: (0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        ad_factor: (0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        qd_factor: (0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0),
    }

    // Test stem_len {{{2
    macro_rules! test_stem_len {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_fwd, angle) = $value;
                    let mut hull = Hull::default();
                    hull.fc_fwd = fc_fwd; hull.bow_angle = angle;

                    assert!(expected == hull.stem_len());
                }
            )*
        }
    }
    test_stem_len! {
        //          name: (stem_len, fc_fwd, bow_angle)
        stem_fc_eq_zero: (0.0, 0.0, 0.0),
        stem_angle_eq_180: (0.0, 10.0, 0.0),
        stem_angle_eq_neg_180: (0.0, 10.0, 0.0),
        stem_angle_eq_zero: (0.0, 10.0, 0.0),
        stem_plus_angle: (63.13751514675041, 10.0, 45.0),
        stem_neg_angle: (-63.13751514675041, 10.0, -45.0),
    }

    // Test lwl {{{2
    macro_rules! test_lwl {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, loa, angle, stern, ram) = $value;
                    hull.set_loa(loa); hull.bow_angle = angle; hull.stern_overhang = stern;
                    hull.fc_fwd = 10.0;
                    if ram != 0.0 {
                        hull.bow_type = BowType::Ram(ram);
                    } else {
                        hull.bow_type = BowType::Normal;
                    }
                    println!("{}",hull.lwl());
                    assert!(expected == hull.lwl());
                }
            )*
        }
    }
    test_lwl! {
        //       name: (lwl, loa, bow_angle, stern_overhang, ram_len)
        lwl_lwl_eq_lwl: (500.0, 500.0, 0.0, 0.0, 0.0),
        lwl_plus_stern: (490.0, 500.0, 0.0, 10.0, 0.0),
        lwl_neg_stern: (500.0, 500.0, 0.0, -10.0, 0.0),
        lwl_plus_ram: (490.0, 500.0, 0.0, 0.0, 10.0),
        lwl_neg_ram: (500.0, 500.0, 0.0, 0.0, -10.0),
        lwl_plus_angle: (496.75080303767095, 500.0, 10.0, 0.0, 0.0),
        lwl_neg_angle: (500.0, 500.0, -10.0, 0.0, 0.0),
        lwl_ram_and_angle: (490.0, 500.0, 10.0, 0.0, 10.0),
        lwl_angle_and_ram: (496.75080303767095, 500.0, 10.0, 0.0, 1.0),
        lwl_all_neg: (500.0, 500.0, -10.0, -10.0, -10.0),
    }

    // Test loa {{{2
    macro_rules! test_loa {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, lwl, angle, stern, ram) = $value;
                    hull.set_lwl(lwl); hull.bow_angle = angle; hull.stern_overhang = stern;
                    hull.fc_fwd = 10.0;
                    if ram != 0.0 {
                        hull.bow_type = BowType::Ram(ram);
                    } else {
                        hull.bow_type = BowType::Normal;
                    }
                    assert!(expected == hull.loa());
                }
            )*
        }
    }
    test_loa! {
        //       name: (loa, lwl, bow_angle, stern_overhang, ram_len)
        loa_loa_eq_lwl: (500.0, 500.0, 0.0, 0.0, 0.0),
        loa_plus_stern: (510.0, 500.0, 0.0, 10.0, 0.0),
        loa_neg_stern: (500.0, 500.0, 0.0, -10.0, 0.0),
        loa_plus_ram: (510.0, 500.0, 0.0, 0.0, 10.0),
        loa_neg_ram: (500.0, 500.0, 0.0, 0.0, -10.0),
        loa_plus_angle: (503.24919696232905, 500.0, 10.0, 0.0, 0.0),
        loa_neg_angle: (500.0, 500.0, -10.0, 0.0, 0.0),
        loa_ram_and_angle: (510.0, 500.0, 10.0, 0.0, 10.0),
        loa_angle_and_ram: (503.24919696232905, 500.0, 10.0, 0.0, 1.0),
        loa_all_neg: (500.0, 500.0, -10.0, -10.0, -10.0),
    }

    // Test d {{{2
    macro_rules! test_d {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, cb, lwl, bb, t) = $value;
                    hull.set_cb(cb); hull.set_lwl(lwl); hull.bb = bb; hull.t = t;
                    println!("{}", hull.d());
                    assert!(expected == hull.d());
                }
            )*
        }
    }
    test_d! {
        //     name: (d, cb, lwl, bb, t)
        d_cb_eq_zero: (0.0, 0.0, 1.0, 1.0, 1.0),
        d_lwl_eq_zero: (0.0, 1.0, 0.0, 1.0, 1.0),
        d_bb_eq_zero: (0.0, 1.0, 1.0, 0.0, 1.0),
        d_teq_zero: (0.0, 1.0, 1.0, 1.0, 0.0),
        d_test: (14.285714285714286, 0.5, 100.0, 5.0, 2.0),
    }

    // Test Cb {{{2
    macro_rules! test_cb {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, d, lwl, bb, t,) = $value;
                    hull.set_d(d); hull.set_lwl(lwl); hull.bb = bb; hull.t = t;
                    assert!(expected == hull.cb());
                }
            )*
        }
    }
    test_cb! {
        //     name: (cb, d, lwl, bb, t)
        cb_d_eq_zero: (0.0, 0.0, 1.0, 1.0, 1.0),
        cb_lwl_eq_zero: (0.0, 1.0, 0.0, 1.0, 1.0),
        cb_bb_eq_zero: (0.0, 1.0, 1.0, 0.0, 1.0),
        cb_t_eq_zero: (0.0, 1.0, 1.0, 1.0, 0.0),
        cb_solid_block: (1.0, 1.0, Hull::FT3_PER_TON_SEA, 1.0, 1.0), // lwl * bb * t == Hull::FT3_PER_TON_SEA => 1.0
        cb_negative: (0.0, -1.0, 1.0, 1.0, 1.0),
        cb_maximum: (1.0, 100.0, 1.0, 1.0, 1.0),
        cb_generic: (0.7, 8000.0, 800.0, 50.0, 10.0),
    }

    // Test Cs {{{2
    macro_rules! test_cs {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, lwl, bb) = $value;
                    hull.set_lwl(lwl); hull.bb = bb;
                    hull.set_d(5000.0); hull.t = 10.0;
                    assert!(expected == hull.cs());
                }
            )*
        }
    }
    test_cs! {
        //     name: (cs, lwl, bb)
        cs_bb_eq_zero: (0.0, 100.0, 0.0),
        cs_lwl_eq_zero: (0.0, 0.0, 10.0),
        cs_general: (0.3914332884034534, 500.0, 50.0),
    }

    // Test Cm {{{2
    macro_rules! test_cm {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cb) = $value;
                    assert_eq!(expected, Hull::cm(cb));
                }
            )*
        }
    }

    test_cm! {
        // name:  (cm, cb)
        cb_eq_zero: (1.006, 0.0),
        cb_eq_one: (1.0004, 1.0),
        cb_eq_half: (0.9399527390653587, 0.5),
    }

    // Test Cp {{{2
    macro_rules! test_cp {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cb) = $value;
                    assert_eq!(expected, Hull::cp(cb));
                }
            )*
        }
    }

    test_cp! {
        // name:  (cm, cb)
        cp_cb_eq_zero: (0.0, 0.0),
        cp_cb_eq_one: (0.9996001599360256, 1.0),
        cp_cb_eq_half: (0.5319416383606421, 0.5),
    }

    // Test len2beam {{{2
    macro_rules! test_len2beam {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, lwl, bb) = $value;
                    hull.set_lwl(lwl); hull.bb = bb;
                    assert_eq!(expected, hull.len2beam());
                }
            )*
        }
    }

    test_len2beam! {
        // name:  (len2beam, lwl, bb)
        len2beam_lwl_eq_zero: (0.0, 0.0, 10.0),
        len2beam_bb_eq_zero: (0.0, 10.0, 0.0),
        len2beam_same: (1.0, 1.0, 1.0),
        len2beam_diff: (0.5, 1.0, 2.0),
    }

    // Test ws {{{2
    macro_rules! test_ws {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, d, lwl, t) = $value;
                    hull.set_d(d); hull.set_lwl(lwl); hull.t = t;
                    assert_eq!(expected, hull.ws());
                }
            )*
        }
    }

    test_ws! {
        // name:  (ws, d, lwl, t)
        ws_d_eq_zero: (1.7, 0.0, 1.0, 1.0),
        ws_lwl_eq_zero: (Hull::FT3_PER_TON_SEA, 1.0, 0.0, 1.0),
        ws_t_eq_zero: (0.0, 1.0, 1.0, 0.0),
        ws_test: (26000.0, 5000.0, 500.0, 10.0),
    }

    // Test ts {{{2
    macro_rules! test_ts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, t) = $value;
                    hull.t = t;
                    hull.set_d(5000.0); hull.set_lwl(500.0); hull.bb = 50.0;
                    println!("{}", Hull::cm(hull.cb()));
                    assert_eq!(expected, hull.ts());
                }
            )*
        }
    }

    test_ts! {
        // name:  (ts, t)
        ts_t_eq_zero: (0.0, 0.0),
        ts_t: (9.72127910066292, 10.0),
    }

}
