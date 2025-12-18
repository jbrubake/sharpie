use serde::{Serialize, Deserialize};

use std::f64::consts::PI;

use crate::SternType;
use crate::BowType;
use crate::unit_types::Units;

// Hull {{{1
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hull {
    /// Units
    pub units: Units,

    /// Block Coefficient at normal displacement.
        cb: Option<f64>,
    /// Normal Displacement (t)
        d: Option<f64>,
    /// Overall length including ram and any overhangs
        loa: Option<f64>,
    /// Length (waterline): Maximum length in water, including any ram.
        lwl: Option<f64>,

    /// Beam (hull): Maximum width in the water, excluding torpedo bulges and
    /// above water overhangs.
    pub b: f64,
    /// Beam (bulges): Maximum width in the water including torpedo bulges but
    /// excluding above water overhangs.
    pub bb: f64,
    /// Draft: Maximum hull draft at normal displacement.
    pub t: f64,

    /// The Waterplane Coefficient is calculated differently if the engine has
    /// less than two shafts. Set this to true if the engine has less than two
    /// shafts but set to false otherwise.
    /// Set to true if the engine has less than two shafts. Otherwise set to false.
    ///
    // TODO: Maybe add the shaft number to the constructor? Or have a set_boxy()
    // method?
    pub boxy: bool,

    pub bow_type: BowType,
    pub stern_type: SternType,

    /// Length of stern overhang
    pub stern_overhang: f64,

    /// Forecastle length.
    pub fc_len: f64,
    /// Height of forecastle forward.
    pub fc_fwd: f64,
    /// Height of forecastle aft.
    pub fc_aft: f64,

    /// Foredeck length.
    pub fd_len: f64,
    /// Height of foredeck forward.
    pub fd_fwd: f64,
    /// Height of foredeck aft.
    pub fd_aft: f64,
    /// Height of aftdeck forward.

    // NOTE: ad_len() is a method
    pub ad_fwd: f64,
    /// Height of aftdeck aft.
    pub ad_aft: f64,

    /// Quarterdeck length.
    pub qd_len: f64,
    /// Height of quarterdeck forward.
    pub qd_fwd: f64,
    /// Height of quarterdeck aft.
    pub qd_aft: f64,

    /// Average rake of stem from waterline to staff. Positive angles indicate
    /// an overhang.
    ///
    /// Range: (-90, 90)
    pub bow_angle: f64,
}

impl Default for Hull { // {{{1
    fn default() -> Hull {
        Hull {
            units: Units::Imperial,

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

// Hull API {{{1
impl Hull {
    pub fn new(
        d: f64,
        b: f64,
        bb: f64,
        t: f64,
    ) -> Hull {
        Hull {
            units: Units::Imperial,

            cb: None,
            d: Some(d),
            lwl: None,
            loa: None,

            b: b.into(),
            bb: bb.into(),
            t: t.into(),

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
    /// Volume of one long ton of seawater in ft³
    pub const FT3_PER_TON_SEA: f64 = 35.0;

    // freeboard_desc {{{2
    pub fn freeboard_desc(&self) -> String {
        let mut s: Vec<String> = Vec::new();

        if self.fc_aft == self.fd_fwd &&
           self.fd_aft == self.ad_fwd &&
           self.ad_aft == self.qd_fwd {

            s.push("flush deck".into());
        } else {
            if self.fc_aft != self.fd_fwd {
                if self.fc_aft > self.fd_fwd {
                    s.push("raised forecastle".into());
                } else if self.fc_aft < self.fd_fwd {
                    s.push("low forecastle".into());
                }
            }

            if self.fd_aft != self.ad_fwd {
                if self.fd_aft > self.ad_fwd {
                    s.push("rise forward of midbreak".into());
                } else if self.fd_aft < self.ad_fwd {
                    s.push("rise aft of midbreak".into());
                }
            }

            if self.ad_aft > self.qd_fwd {
                s.push("low quarterdeck".into());
            }

            if self.ad_aft < self.qd_fwd {
                s.push("raised quarterdeck".into());
            }
        }

        s.join(", ")
    }

    // cs {{{2
    /// Coefficient of Sharpness.
    ///
    pub fn cs(&self) -> f64 {
        if self.lwl() == 0.0 { return 0.0; }
        0.4 * (self.bb / self.lwl() * 6.0).powf(1.0/3.0) * f64::sqrt(self.cb() / 0.52)
    }

    // cm {{{2
    /// Misdhip section area Coefficient (Keslen).
    ///
    pub fn cm(block: f64) -> f64 {
        match block {
            0.0 => 1.006, // The float math doesn't work out if block == 0.0
            _   => 1.006 - 0.0056 * block.powf(-3.56),
        }
    }

    // cp {{{2
    /// Prismatic Coefficient.
    ///
    pub fn cp(block: f64) -> f64 {
        block / Hull::cm(block)
    }

    // cb_calc {{{2
    /// Calculate the Block Coefficient for a given displacment.
    ///
    // XXX: Should the minimum be clamped to 0.3?
    pub fn cb_calc(&self, d: f64, t: f64) -> f64 {
        let volume = self.lwl() * self.bb * t;

        if volume == 0.0 {
            0.0
        } else {
            (d * Self::FT3_PER_TON_SEA / volume).min(1.0).max(0.0)
        }
    }

    // cb {{{2
    /// Block Coefficient at normal displacement.
    ///
    /// Range: [0.0, 1.0]
    pub fn cb(&self) -> f64 {
        match self.cb {
            Some(cb) => cb,
            None     => self.cb_calc(self.d(), self.t),
        }
    }

    // set_d {{{2
    /// Set the Displacement and unset the Block Coefficient.
    ///
    pub fn set_d(&mut self, d: f64) -> f64 {
        self.d = Some(d);
        self.cb = None;

        d
    }

    // set_cb {{{2
    /// Set the Block Coefficient and unset the Displacement.
    ///
    pub fn set_cb(&mut self, cb: f64) -> f64 {
        self.cb = Some(cb);
        self.d = None;

        cb
    }

    // d {{{2
    /// Normal Displacement (t).
    ///
    pub fn d(&self) -> f64 {
        match self.d {
            Some(d) => d,
            None    => self.cb() * self.lwl() * self.bb * self.t / Self::FT3_PER_TON_SEA
        }
    }

    // cwp {{{2
    /// Waterplane Area Coefficient (Parsons).
    ///
    pub fn cwp(&self) -> f64 {
        let (mut a, mut f) = self.stern_type.wp_calc();

        if self.boxy || self.cb() >= 0.75 {
            a = 0.175;
            f = 0.875;
        }
        
        let cwp = f64::min(
            a + f * Hull::cp( f64::max(self.cb(), 0.4) ),
            1.0
        );

        cwp - if self.cb() < 0.4 {
                0.0281 - (self.cb() - 0.3).powf(1.55)
            } else {
                0.0
            }
    }

    // wp {{{2
    /// Waterplane Area.
    ///
    pub fn wp(&self) -> f64 {
        self.cwp() * self.lwl() * self.b
    }

    // ws {{{2
    /// Wetted Surface Area (Mumford).
    ///
    pub fn ws(&self) -> f64 {
        if self.t == 0.0 { return 0.0; } // catch divide by zero
        self.lwl() * self.t * 1.7 + (self.d() * Self::FT3_PER_TON_SEA / self.t)
    }

    // set_lwl {{{2
    /// Set the waterline length and unset the overall length.
    ///
    pub fn set_lwl(&mut self, len: f64) -> f64 {
        self.lwl = Some(len);
        self.loa = None;

        len
    }

    // set_loa {{{2
    /// Set the overall length and unset the waterline length.
    ///
    pub fn set_loa(&mut self, len: f64) -> f64 {
        self.loa = Some(len);
        self.lwl = None;

        len
    }

    // lwl {{{2
    /// Length at the waterline.
    ///
    /// lwl = loa - max(ram_length, length_from_bow_angle, 0) - max(stern_overhang, 0)
    ///
    /// Range: (0, ∞)
    pub fn lwl(&self) -> f64 {
        match (self.lwl, self.loa) {
            (None, None)      => 0.0,
            (Some(len), _)    => len,
            (None, Some(loa)) =>
                loa -
                f64::max(
                    self.bow_type.ram_len(),
                    self.stem_len()
                ).max(0.0) -
                self.stern_overhang.max(0.0),
        }
    }

    // loa {{{2
    /// Overall length.
    ///
    /// loa = lwl + max(ram_length, length_from_bow_angle, 0) + max(stern_overhang, 0)
    ///
    /// Range: [lwl, ∞)
    pub fn loa(&self) -> f64 {
        match (self.loa, self.lwl) {
            (None, None)      => 0.0,
            (Some(len), _)    => len,
            (None, Some(lwl)) =>
                lwl +
                f64::max(
                    self.bow_type.ram_len(),
                    self.stem_len()
                ).max(0.0) +
                self.stern_overhang.max(0.0),
        }
    }

    // leff {{{2
    /// Effective length based on waterline length, bulge width, sharpness
    /// coefficient and stern type.
    ///
    pub fn leff(&self) -> f64 {
        self.stern_type.leff(self.lwl(), self.bb, self.cs())
    }

    // t_calc {{{2
    /// Draft at given displacment.
    ///
    pub fn t_calc(&self, d: f64) -> f64 {
        self.t + (d - self.d()) / (self.wp() / Hull::FT3_PER_TON_SEA)
    }

    // ts {{{2
    /// Draft at side.
    ///
    pub fn ts(&self) -> f64 {
        (Hull::cm(self.cb()) * 2.0 - 1.0) * self.t
    }

    // ad_len {{{2
    /// Length of the after deck based on other deck lengths.
    ///
    pub fn ad_len(&self) -> f64 {
        1.0 - self.fc_len - self.fd_len - self.qd_len
    }

    // stem_len {{{2
    /// Ship stem adjustment from bow angle.
    ///
    /// Range: (-∞, +∞)
    pub fn stem_len(&self) -> f64 {
        if self.bow_angle.abs() >= 90.0 { // Avoid returning infity
            0.0
        } else {
            self.fc_fwd * f64::tan(self.bow_angle * PI / 180.0)
        }
    }

    // freeboard {{{2
    /// Average freeboard.
    ///
    pub fn freeboard(&self) -> f64 {
        self.fc() * self.fc_len +
        self.fd() * self.fd_len +
        self.ad() * self.ad_len() +
        self.qd() * self.qd_len
    }

    // freeboard_dist {{{2
    /// Freeboard dist (I have no idea what this is).
    ///
    /// Range: [0, ∞)
    pub fn freeboard_dist(&self) -> f64 {
       (self.fd() * self.fd_len + self.ad() * self.ad_len()) / (self.fd_len + self.ad_len()) 
    }

    // is_wet_fwd {{{2
    pub fn is_wet_fwd(&self) -> bool {
        self.fc_fwd < (1.1 * self.lwl().sqrt())
    }

    // fc {{{2
    /// Average forecastle height (weighted to slope up toward the bow).
    ///
    pub fn fc(&self) -> f64 {
        self.fc_aft + (self.fc_fwd - self.fc_aft) * 0.4
    }

    // fd {{{2
    /// Average foredeck height.
    ///
    pub fn fd(&self) -> f64 {
        self.fd_fwd + (self.fd_aft - self.fd_fwd) * 0.5
    }

    // ad {{{2
    /// Average afterdeck height.
    ///
    pub fn ad(&self) -> f64 {
        self.ad_fwd + (self.ad_aft - self.ad_fwd) * 0.5
    }

    // qd {{{2
    /// Average quarterdeck height.
    ///
    pub fn qd(&self) -> f64 {
        self.qd_fwd + (self.qd_aft - self.qd_fwd) * 0.5
    }


    // free_cap {{{2
    /// I have no idea what this is.
    ///
    pub fn free_cap(&self, cap_calc_broadside: bool) -> f64 {
        if self.freeboard() > (self.b / 3.0) {
            self.freeboard().powf(2.0) * 3.0 / self.b
        } else if cap_calc_broadside {
            self.freeboard() - 6.0
        } else {
            self.freeboard()
        }
    }


    // vn {{{2
    /// Natural speed.
    ///
    pub fn vn(&self) -> f64 {
        self.leff().sqrt()
    }

    // len2beam {{{2
    /// Length to beam ratio.
    ///
    pub fn len2beam(&self) -> f64 {
        if self.bb == 0.0 { return 0.0; }
        self.lwl() / self.bb
    }

}
#[cfg(test)] // Hull {{{1
mod hull {
    use super::*;
    use crate::test_support::*;

    // Cs {{{2
    macro_rules! test_cs {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, lwl) = $value;
                    hull.set_lwl(lwl);
                    hull.set_cb(0.55);
                    hull.bb = 10.0;

                    assert!(expected == to_place(hull.cs(), 5));
                }
            )*
        }
    }
    test_cs! {
        //     name:    (cs, lwl)
        cs_lwl_eq_zero: (0.0, 0.0),
        cs_test:        (0.34697, 100.0),
    }

    // Cm {{{2
    macro_rules! test_cm {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cb) = $value;

                    assert_eq!(expected, to_place(Hull::cm(cb), 5));
                }
            )*
        }
    }

    test_cm! {
        // name:    (cm, cb)
        cm_eq_zero: (1.006, 0.0),
        cm_eq_one:  (1.0004, 1.0),
        cm_eq_half: (0.93995, 0.5),
    }

    // Cp {{{2
    macro_rules! test_cp {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cb) = $value;

                    assert_eq!(expected, to_place(Hull::cp(cb), 5));
                }
            )*
        }
    }

    test_cp! {
        // name:       (cm, cb)
        cp_cb_eq_zero: (0.0, 0.0),
        cp_cb_eq_one:  (0.99960, 1.0),
        cp_cb_eq_half: (0.53194, 0.5),
    }

    // Cb {{{2
    macro_rules! test_cb_calc {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, d, lwl, bb, t,) = $value;

                    let mut hull = Hull::default();
                    hull.set_d(d);
                    hull.set_lwl(lwl);
                    hull.bb = bb;
                    hull.t = t;

                    assert!(expected == to_place(hull.cb(), 5));
                }
            )*
        }
    }
    test_cb_calc! {
        //     name:      (cb, d, lwl, bb, t)
        // Volume == 0
        cb_vol_eq_zero_1: (0.0, 1.0, 0.0, 1.0, 1.0),
        cb_vol_eq_zero_2: (0.0, 1.0, 1.0, 0.0, 1.0),
        cb_vol_eq_zero_3: (0.0, 1.0, 1.0, 1.0, 0.0),

        cb_d_eq_zero:     (0.0, 0.0, 1.0, 1.0, 0.0),

        cb_test:          (0.7, 8000.0, 800.0, 50.0, 10.0),

        // Clamping
        cb_negative:      (0.0, -1.0, 1.0, 1.0, 1.0),
        cb_maximum:       (1.0, 100.0, 1.0, 1.0, 1.0),
        // By definition: lwl * bb * t == Hull::FT3_PER_TON_SEA => 1.0
        cb_solid_block:   (1.0, 100.0, Hull::FT3_PER_TON_SEA, 1.0, 1.0),
    }

    // d {{{2
    macro_rules! test_d {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, cb, lwl, bb, t) = $value;

                    let mut hull = Hull::default();
                    hull.set_cb(cb);
                    hull.set_lwl(lwl);
                    hull.bb = bb;
                    hull.t = t;

                    assert!(expected == to_place(hull.d(), 2));
                }
            )*
        }
    }
    test_d! {
        //     name:   (d, cb, lwl, bb, t)
        d_cb_eq_zero:  (0.0, 0.0, 1.0, 1.0, 1.0),
        d_lwl_eq_zero: (0.0, 1.0, 0.0, 1.0, 1.0),
        d_bb_eq_zero:  (0.0, 1.0, 1.0, 0.0, 1.0),
        d_teq_zero:    (0.0, 1.0, 1.0, 1.0, 0.0),
        d_test:        (14.29, 0.5, 100.0, 5.0, 2.0),
        // By definition: lwl * bb * t == Hull::FT3_PER_TON_SEA => d == cb
        d_eq_cb_1: (0.5, 0.5, Hull::FT3_PER_TON_SEA, 1.0, 1.0),
        d_eq_cb_2: (1.0, 1.0, Hull::FT3_PER_TON_SEA, 1.0, 1.0),
    }

    // cwp {{{2
    macro_rules! test_cwp {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, boxy, cb) = $value;

                    let mut hull = Hull::default();
                    hull.set_cb(cb);
                    hull.boxy = boxy;

                    println!("{}", hull.cwp());
                    assert!(expected == to_place(hull.cwp(), 5));
                }
            )*
        }
    }
    test_cwp! {
        // name: (cwp, boxy, cb)
        cwp_test_1: (0.64045, true, 0.5),
        cwp_test_2: (0.83761, false, 0.75),
        cwp_test_3: (0.66628, false, 0.5),
        cwp_test_4: (0.59708, false, 0.35),
    }

    // ws {{{2
    macro_rules! test_ws {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, t) = $value;
                    hull.set_d(1000.0);
                    hull.set_lwl(100.0);
                    hull.t = t;

                    assert_eq!(expected, to_place(hull.ws(), 2));
                }
            )*
        }
    }

    test_ws! {
        // name:   (ws, lwl, t)
        ws_t_eq_0: (0.0, 0.0),
        ws_test:   (5200.0, 10.0),
    }

    // lwl {{{2
    macro_rules! test_lwl {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, angle, stern, ram) = $value;

                    let mut hull = Hull::default();
                    hull.set_loa(100.0);
                    hull.fc_fwd = 10.0;
                    hull.bow_angle = angle;
                    hull.stern_overhang = stern;

                    if ram != 0.0 {
                        hull.bow_type = BowType::Ram(ram);
                    } else {
                        hull.bow_type = BowType::Normal;
                    }

                    assert!(expected == to_place(hull.lwl(), 2));
                }
            )*
        }
    }
    test_lwl! {
        // name:         (lwl, angle, stern, ram)
        lwl_eq_loa:      (100.0, 0.0, 0.0, 0.0),
        lwl_overhang:    (90.0, 0.0, 10.0, 0.0),
        lwl_ram:         (90.0, 0.0, 0.0, 10.0),
        lwl_stem:        (90.0, 45.0, 0.0, 0.0),
        lwl_ram_lt_stem: (90.0, 45.0, 0.0, 5.0),
        lwl_ram_gt_stem: (85.0, 45.0, 0.0, 15.0),
        lwl_ram_stern:   (80.0, 0.0, 10.0, 10.0),
    }

    // loa {{{2
    macro_rules! test_loa {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, angle, stern, ram) = $value;

                    let mut hull = Hull::default();
                    hull.set_lwl(100.0);
                    hull.fc_fwd = 10.0;
                    hull.bow_angle = angle;
                    hull.stern_overhang = stern;

                    if ram != 0.0 {
                        hull.bow_type = BowType::Ram(ram);
                    } else {
                        hull.bow_type = BowType::Normal;
                    }

                    assert!(expected == to_place(hull.loa(), 2));
                }
            )*
        }
    }
    test_loa! {
        // name:         (lwl, angle, stern, ram)
        loa_eq_loa:      (100.0, 0.0, 0.0, 0.0),
        loa_overhang:    (110.0, 0.0, 10.0, 0.0),
        loa_ram:         (110.0, 0.0, 0.0, 10.0),
        loa_stem:        (110.0, 45.0, 0.0, 0.0),
        loa_ram_lt_stem: (110.0, 45.0, 0.0, 5.0),
        loa_ram_gt_stem: (115.0, 45.0, 0.0, 15.0),
        loa_ram_stern:   (120.0, 0.0, 10.0, 10.0),
    }

    // t {{{2
    macro_rules! test_t {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, d_plus) = $value;

                    let mut hull = Hull::default();
                    hull.set_d(5000.0);
                    hull.set_lwl(500.0);
                    hull.b = 50.0;
                    hull.bb = hull.b;
                    hull.t = 10.0;

                    assert!(expected == to_place(hull.t_calc(hull.d() + d_plus), 2));
                }
            )*
        }
    }
    test_t! {
        // name: (t, d_plus),
        t_test_1: (10.87, 500.0),
    }

    // ts {{{2
    macro_rules! test_ts {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, t) = $value;

                    let mut hull = Hull::default();
                    hull.set_d(5000.0);
                    hull.set_lwl(500.0);
                    hull.t = t;
                    hull.b = 50.0;
                    hull.bb = hull.b;

                    assert_eq!(expected, to_place(hull.ts(), 2));
                }
            )*
        }
    }

    test_ts! {
        // name:      (ts, t)
        ts_t_eq_zero: (0.0, 0.0),
        ts_t:         (9.72, 10.0),
    }

    // ad_len {{{2
    macro_rules! test_ad_len {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_len) = $value;

                    let mut hull = Hull::default();
                    hull.fc_len = fc_len;
                    hull.fd_len = 0.25;
                    hull.qd_len = 0.25;

                    assert!(expected == to_place(hull.ad_len(), 2));
                }
            )*
        }
    }
    test_ad_len! {
        // name: (ad_len, fc_len)
        ad_len_zero: (0.0, 0.5),
        ad_len_test: (0.25, 0.25),
    }

    // stem_len {{{2
    macro_rules! test_stem_len {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, angle) = $value;

                    let mut hull = Hull::default();
                    hull.fc_fwd = 10.0;
                    hull.bow_angle = angle;

                    assert!(expected == to_place(hull.stem_len(), 2));
                }
            )*
        }
    }
    test_stem_len! {
        //name:             (stem_len, bow_angle)
        stem_len_neg_angle: (-10.0, -45.0),
        stem_len_pos_angle: (10.0, 45.0),
        stem_len_no_angle:  (0.0, 0.0),
        stem_len_90:        (0.0, 90.0),
    }

    // freeboard {{{2
    macro_rules! test_freeboard {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_len) = $value;

                    let mut hull = Hull::default();

                    hull.fc_len = fc_len;
                    hull.fc_fwd = 10.0;
                    hull.fc_aft = 10.0;

                    hull.fd_len = (1.0 - fc_len) * 0.4;
                    hull.fd_fwd = hull.fc_fwd + 5.0;
                    hull.fd_aft = hull.fc_fwd;

                    hull.ad_fwd = hull.fc_fwd + 10.0;
                    hull.ad_aft = hull.fc_fwd;

                    hull.qd_len = (1.0 - fc_len) * 0.4;
                    hull.qd_fwd = hull.fc_fwd - 5.0;
                    hull.qd_aft = hull.fc_fwd;

                    assert!(expected == to_place(hull.freeboard(), 3));
                }
            )*
        }
    }
    test_freeboard! {
        // name: (freeboard, fc_len)
        freeboard_test: (10.75, 0.25),
    }

    // freeboard_dist {{{2
    macro_rules! test_freeboard_dist {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_len) = $value;

                    let mut hull = Hull::default();

                    hull.fc_len = fc_len;
                    hull.fc_fwd = 10.0;
                    hull.fc_aft = 10.0;

                    hull.fd_len = (1.0 - fc_len) * 0.4;
                    hull.fd_fwd = hull.fc_fwd + 5.0;
                    hull.fd_aft = hull.fc_fwd;

                    hull.ad_fwd = hull.fc_fwd + 10.0;
                    hull.ad_aft = hull.fc_fwd;

                    hull.qd_len = (1.0 - fc_len) * 0.4;
                    hull.qd_fwd = hull.fc_fwd - 5.0;
                    hull.qd_aft = hull.fc_fwd;

                    assert!(expected == to_place(hull.freeboard_dist(), 2));
                }
            )*
        }
    }
    test_freeboard_dist! {
        // name:             (dist, fc_len)
        freeboard_dist_test: (13.33, 10.0),
    }
    // is_wet_fwd {{{2
    macro_rules! test_is_wet_fwd {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_fwd) = $value;

                    let mut hull = Hull::default();
                    hull.fc_fwd = fc_fwd;
                    hull.set_lwl(100.0);

                    assert_eq!(expected, hull.is_wet_fwd());
                }
            )*
        }
    }

    test_is_wet_fwd! {
        // name:          (is_wet_fwd, fc_fwd)
        is_wet_fwd_true:  (true, 0.0),
        is_wet_fwd_false: (false, 20.0),
    }

    // fc {{{2
    macro_rules! test_fc {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fc_fwd, fc_aft) = $value;

                    let mut hull = Hull::default();

                    hull.fc_fwd = fc_fwd;
                    hull.fc_aft = fc_aft;

                    assert_eq!(expected, hull.fc());
                }
            )*
        }
    }

    test_fc! {
        // name:           (fc, fc_fwd, fc_aft)
        fc_test_eq:        (10.0, 10.0, 10.0),
        fc_test_slope_fwd: (4.0, 10.0, 0.0),
        fc_test_slope_aft: (6.0, 0.0, 10.0),
    }

    // fd {{{2
    macro_rules! test_fd {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fd_fwd, fd_aft) = $value;

                    let mut hull = Hull::default();

                    hull.fd_fwd = fd_fwd;
                    hull.fd_aft = fd_aft;

                    assert_eq!(expected, hull.fd());
                }
            )*
        }
    }

    test_fd! {
        // name:           (fd, fd_fwd, fd_aft)
        fd_test_eq:        (10.0, 10.0, 10.0),
        fd_test_slope_fwd: (5.0, 10.0, 0.0),
        fd_test_slope_aft: (5.0, 0.0, 10.0),
    }

    // ad {{{2
    macro_rules! test_ad {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, ad_fwd, ad_aft) = $value;

                    let mut hull = Hull::default();
                    hull.ad_fwd = ad_fwd;
                    hull.ad_aft = ad_aft;

                    assert_eq!(expected, hull.ad());
                }
            )*
        }
    }

    test_ad! {
        // name:           (ad, ad_fwd, ad_aft)
        ad_test_eq:        (10.0, 10.0, 10.0),
        ad_test_slope_fwd: (5.0, 10.0, 0.0),
        ad_test_slope_aft: (5.0, 0.0, 10.0),
    }

    // qd {{{2
    macro_rules! test_qd {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, qd_fwd, qd_aft) = $value;

                    let mut hull = Hull::default();

                    hull.qd_fwd = qd_fwd;
                    hull.qd_aft = qd_aft;

                    assert_eq!(expected, hull.qd());
                }
            )*
        }
    }

    test_qd! {
        // name:           (qd, qd_fwd, qd_aft)
        qd_test_eq:        (10.0, 10.0, 10.0),
        qd_test_slope_fwd: (5.0, 10.0, 0.0),
        qd_test_slope_aft: (5.0, 0.0, 10.0),
    }

    // free_cap {{{2
    macro_rules! test_free_cap {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, b, cap_calc_broad) = $value;

                    let mut hull = Hull::default();

                    hull.b = b;

                    hull.fc_len = 0.25;
                    hull.fc_fwd = 10.0;
                    hull.fc_aft = 10.0;

                    hull.fd_len = 0.25;
                    hull.fd_fwd = hull.fc_fwd;
                    hull.fd_aft = hull.fc_fwd;

                    hull.ad_fwd = hull.fc_fwd;
                    hull.ad_aft = hull.fc_fwd;

                    hull.qd_len = 0.25;
                    hull.qd_fwd = hull.fc_fwd;
                    hull.qd_aft = hull.fc_fwd;

                    assert_eq!(expected, hull.free_cap(cap_calc_broad));
                }
            )*
        }
    }

    test_free_cap! {
        // name:    (free_cap, b, cap_calc_broad)
        free_cap_case_1: (100.0, 3.0, true),
        free_cap_case_2: (4.0, 70.0, true),
        free_cap_case_3: (10.0, 70.0, false),
    }

    // vn {{{2
    macro_rules! test_vn {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, lwl) = $value;
                    hull.set_lwl(lwl);
                    hull.bb = 10.0;
                    hull.set_cb(0.55);
                    hull.stern_type = SternType::Cruiser;

                    assert_eq!(expected, to_place(hull.vn(), 2));
                }
            )*
        }
    }

    test_vn! {
        // name:   (vn, lwl),
        vn_test_1: (10.0, 100.0),
        vn_test_2: (14.14, 200.0),
    }

    // len2beam {{{2
    macro_rules! test_len2beam {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut hull = Hull::default();

                    let (expected, bb) = $value;
                    hull.set_lwl(100.0);
                    hull.bb = bb;

                    assert_eq!(expected, to_place(hull.len2beam(), 2));
                }
            )*
        }
    }

    test_len2beam! {
        // name:              (len2beam, bb)
        len2beam_bb_eq_zero:  (0.0, 0.0),
        len2beam_test:        (5.0, 20.0),
    }

}
