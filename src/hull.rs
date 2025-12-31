use crate::units::Units;

use serde::{Serialize, Deserialize};

use std::f64::consts::PI;
use std::fmt;

// Hull {{{1
/// Hull characteristics.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hull {
    /// Units
    pub units: Units,

    /// Block Coefficient at normal displacement.
    ///
    /// This is None if d is set.
        cb: Option<f64>,
    /// Normal Displacement (t)
    ///
    /// This is None if cb is set.
        d: Option<f64>,

    /// Overall length including ram and any overhangs
    ///
    /// This is None if lwl is set.
        loa: Option<f64>,
    /// Maximum length in the water, including any ram.
    ///
    /// This is None if loa is set.
        lwl: Option<f64>,

    /// Beam (hull): Maximum width in the water, excluding torpedo bulges and
    /// above water overhangs.
    pub b: f64,
    /// Beam (bulges): Maximum width in the water including torpedo bulges but
    /// excluding above water overhangs.
    // TODO: This should be ignored if it is less than b but how does Springsharp do it?
    pub bb: f64,
    /// Draft: Maximum hull draft at normal displacement.
    pub t: f64,

    /// The Waterplane Coefficient is calculated differently if the engine has
    /// less than two shafts. Set this to true if the engine has less than two
    /// shafts but set to false otherwise.
    /// Set to true if the engine has less than two shafts. Otherwise set to false.
    ///
    // TODO: replace this with simply passing the number of shafts to the functions that currently use boxy
    pub boxy: bool,

    /// Type of bow.
    pub bow_type: BowType,
    /// Type of stern.
    pub stern_type: SternType,

    /// Length of stern overhang
    pub stern_overhang: f64,

    /// Forecastle length as a fraction of the total deck.
    pub fc_len: f64,
    /// Height of forecastle forward.
    pub fc_fwd: f64,
    /// Height of forecastle aft.
    pub fc_aft: f64,

    /// Foredeck length as a fraction of the total deck.
    pub fd_len: f64,
    /// Height of foredeck forward.
    pub fd_fwd: f64,
    /// Height of foredeck aft.
    pub fd_aft: f64,

    // NOTE: ad_len() is calculated from fc_len and fd_len
    /// Height of aftdeck forward.
    pub ad_fwd: f64,
    /// Height of aftdeck aft.
    pub ad_aft: f64,

    /// Quarterdeck length as a fraction of the total deck.
    pub qd_len: f64,
    /// Height of quarterdeck forward.
    pub qd_fwd: f64,
    /// Height of quarterdeck aft.
    pub qd_aft: f64,

    /// Average rake of stem from waterline to staff.
    /// Positive angles indicate an overhang.
    pub bow_angle: f64,
}

impl Default for Hull { // {{{2
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

impl Hull { // {{{2
    /// Volume of one long ton of seawater in cubic feet.
    pub const FT3_PER_TON_SEA: f64 = 35.0;

    // freeboard_desc {{{3
    /// Get a description of the freeboard.
    ///
    pub fn freeboard_desc(&self) -> String {
        let mut s: Vec<String> = Vec::new();

        // XXX: Non-flush decks could still match here
        if self.fc_aft == self.fd_fwd &&
           self.fd_aft == self.ad_fwd &&
           self.ad_aft == self.qd_fwd {

            s.push("flush deck".into());
        } else {
            if self.fc_aft > self.fd_fwd {
                s.push("raised forecastle".into());
            } else if self.fc_aft < self.fd_fwd {
                s.push("low forecastle".into());
            }

            if self.fd_aft > self.ad_fwd {
                s.push("rise forward of midbreak".into());
            } else if self.fd_aft < self.ad_fwd {
                s.push("rise aft of midbreak".into());
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

    // cs {{{3
    /// Coefficient of Sharpness.
    ///
    pub fn cs(&self) -> f64 {
        if self.lwl() == 0.0 { return 0.0; } // Catch divide by zero

        0.4 * (self.bb / self.lwl() * 6.0).powf(1.0/3.0) * f64::sqrt(self.cb() / 0.52)
    }

    // cm {{{3
    /// Misdhip section area Coefficient (Keslen).
    ///
    // XXX: Should this be a method?
    pub fn cm(block: f64) -> f64 {
        match block {
            // XXX: Does this matter? cb should never by less than 0.3
            0.0 => 1.006, // The float math doesn't work out if block == 0.0
            _   => 1.006 - 0.0056 * block.powf(-3.56),
        }
    }

    // cp {{{3
    /// Prismatic Coefficient.
    ///
    // XXX: Should this be a method?
    pub fn cp(block: f64) -> f64 {
        block / Hull::cm(block)
    }

    // cb {{{3
    /// Block Coefficient at normal displacement.
    ///
    /// Return a perviously set value or cb_calc() if unset.
    pub fn cb(&self) -> f64 {
        match self.cb {
            Some(cb) => cb,
            None     => self.cb_calc(self.d(), self.t),
        }
    }

    // cb_calc {{{3
    /// Calculate the Block Coefficient for a given displacment.
    ///
    // XXX: Should this only return values between 0.3 and 1.0 (inclusive)?
    pub fn cb_calc(&self, d: f64, t: f64) -> f64 {
        let volume = self.lwl() * self.bb * t;

        if volume == 0.0 {
            0.0
        } else {
            (d * Self::FT3_PER_TON_SEA / volume).min(1.0).max(0.0)
        }
    }

    // set_cb {{{3
    /// Set the Block Coefficient and unset the Displacement.
    ///
    pub fn set_cb(&mut self, cb: f64) -> f64 {
        self.cb = Some(cb);
        self.d = None;

        cb
    }

    // d {{{3
    /// Normal Displacement (t).
    ///
    /// Return a perviously set value or caluculate from cb if unset.
    pub fn d(&self) -> f64 {
        match self.d {
            Some(d) => d,
            None    => self.d_calc(self.cb(),),
        }
    }

    // d_calc {{{3
    /// Calculate the displacement for a given Block Coefficient.
    ///
    pub fn d_calc(&self, cb: f64) -> f64 {
        cb * self.lwl() * self.bb * self.t / Self::FT3_PER_TON_SEA
    }

    // set_d {{{3
    /// Set the Displacement and unset the Block Coefficient.
    ///
    pub fn set_d(&mut self, d: f64) -> f64 {
        self.d = Some(d);
        self.cb = None;

        d
    }

    // cwp {{{3
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

    // wp {{{3
    /// Waterplane Area.
    ///
    pub fn wp(&self) -> f64 {
        self.cwp() * self.lwl() * self.b
    }

    // ws {{{3
    /// Wetted Surface Area (Mumford).
    ///
    pub fn ws(&self) -> f64 {
        if self.t == 0.0 { return 0.0; } // catch divide by zero
                                         //
        self.lwl() * self.t * 1.7 + (self.d() * Self::FT3_PER_TON_SEA / self.t)
    }

    // set_lwl {{{3
    /// Set the waterline length and unset the overall length.
    ///
    pub fn set_lwl(&mut self, len: f64) -> f64 {
        self.lwl = Some(len);
        self.loa = None;

        len
    }

    // set_loa {{{3
    /// Set the overall length and unset the waterline length.
    ///
    pub fn set_loa(&mut self, len: f64) -> f64 {
        self.loa = Some(len);
        self.lwl = None;

        len
    }

    // lwl {{{3
    /// Length at the waterline.
    ///
    /// lwl = loa - max(ram_length, length_from_bow_angle, 0) - max(stern_overhang, 0)
    ///
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

    // loa {{{3
    /// Overall length.
    ///
    /// loa = lwl + max(ram_length, length_from_bow_angle, 0) + max(stern_overhang, 0)
    ///
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

    // leff {{{3
    /// Effective length based on waterline length, bulge width, sharpness
    /// coefficient and stern type.
    ///
    pub fn leff(&self) -> f64 {
        self.stern_type.leff(self.lwl(), self.bb, self.cs())
    }

    // t_calc {{{3
    /// Draft at given displacment.
    ///
    pub fn t_calc(&self, d: f64) -> f64 {
        self.t + (d - self.d()) / (self.wp() / Hull::FT3_PER_TON_SEA)
    }

    // ts {{{3
    /// Draft at side.
    ///
    pub fn ts(&self) -> f64 {
        (Hull::cm(self.cb()) * 2.0 - 1.0) * self.t
    }

    // ad_len {{{3
    /// Length of the after deck as a fraction of the total
    /// deck based on forecastle, fore and aft decks.
    ///
    pub fn ad_len(&self) -> f64 {
        1.0 - self.fc_len - self.fd_len - self.qd_len
    }

    // stem_len {{{3
    /// Increase or decrease to length due to the angle of the bow.
    ///
    pub fn stem_len(&self) -> f64 {
        if self.bow_angle.abs() >= 90.0 { // Avoid returning infity
            0.0
        } else {
            self.fc_fwd * f64::tan(self.bow_angle * PI / 180.0)
        }
    }

    // freeboard {{{3
    /// Average freeboard.
    ///
    pub fn freeboard(&self) -> f64 {
        self.fc() * self.fc_len +
        self.fd() * self.fd_len +
        self.ad() * self.ad_len() +
        self.qd() * self.qd_len
    }

    // freeboard_dist {{{3
    /// XXX: I do not know what this does.
    ///
    pub fn freeboard_dist(&self) -> f64 {
       (self.fd() * self.fd_len + self.ad() * self.ad_len()) / (self.fd_len + self.ad_len()) 
    }

    // is_wet_fwd {{{3
    /// Does the ship tend to be wet forward?
    ///
    pub fn is_wet_fwd(&self) -> bool {
        self.fc_fwd < (1.1 * self.lwl().sqrt())
    }

    // fc {{{3
    /// Average forecastle height (weighted to slope up toward the bow).
    ///
    pub fn fc(&self) -> f64 {
        self.fc_aft + (self.fc_fwd - self.fc_aft) * 0.4
    }

    // fd {{{3
    /// Average foredeck height.
    ///
    pub fn fd(&self) -> f64 {
        self.fd_fwd + (self.fd_aft - self.fd_fwd) * 0.5
    }

    // ad {{{3
    /// Average afterdeck height.
    ///
    pub fn ad(&self) -> f64 {
        self.ad_fwd + (self.ad_aft - self.ad_fwd) * 0.5
    }

    // qd {{{3
    /// Average quarterdeck height.
    ///
    pub fn qd(&self) -> f64 {
        self.qd_fwd + (self.qd_aft - self.qd_fwd) * 0.5
    }


    // free_cap {{{3
    /// XXX: I do not know what this does.
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


    // vn {{{3
    /// Natural speed of the hull.
    ///
    pub fn vn(&self) -> f64 {
        self.leff().sqrt()
    }

    // len2beam {{{3
    /// Length to beam ratio.
    ///
    pub fn len2beam(&self) -> f64 {
        if self.bb == 0.0 { return 0.0; } // Catch divide by zero.

        self.lwl() / self.bb
    }

}

// Testing Hull {{{2
#[cfg(test)]
mod hull {
    use super::*;
    use crate::test_support::*;

    // Cs {{{3
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

    // Cm {{{3
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

    // Cp {{{3
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

    // Cb {{{3
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

    // d {{{3
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

    // cwp {{{3
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

    // ws {{{3
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

    // lwl {{{3
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

    // loa {{{3
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

    // t {{{3
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

    // ts {{{3
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

    // ad_len {{{3
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

    // stem_len {{{3
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

    // freeboard {{{3
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

    // freeboard_dist {{{3
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
    // is_wet_fwd {{{3
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

    // fc {{{3
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

    // fd {{{3
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

    // ad {{{3
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

    // qd {{{3
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

    // free_cap {{{3
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

    // vn {{{3
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

    // len2beam {{{3
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
// SternType {{{1
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum SternType {
    /// Transom stern (small).
    TransomSm,
    /// Transom stern (large).
    TransomLg,
    #[default]
    /// Cruiser stern (default).
    Cruiser,
    /// Round stern.
    Round,
}

impl From<String> for SternType {
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for SternType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::TransomSm,
            "2" => Self::TransomLg,
            "3" => Self::Round,
            "0" | _ => Self::Cruiser,
        }
    }
}

impl fmt::Display for SternType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            SternType::TransomSm => "a small transom stern",
            SternType::TransomLg => "a large transom stern",
            SternType::Cruiser   => "a cruiser stern",
            SternType::Round     => "a round stern",
        })
    }
}

impl SternType {
    // wp_calc {{{2
    /// XXX: ???
    ///
    pub fn wp_calc(&self) -> (f64, f64) {
        match self {
            Self::TransomSm => (0.262, 0.79),
            Self::TransomLg => (0.262, 0.81),
            Self::Cruiser   => (0.262, 0.76),
            Self::Round     => (0.262, 0.76),
        }
    }

    // leff {{{2
    /// XXX: ???
    ///
    pub fn leff(&self, lwl: f64, bb: f64, cs: f64) -> f64 {
        if cs == 0.0 { return 0.0 } // catch divide by zero

        match self {
            Self::TransomSm => bb * 0.5 / cs + lwl,
            Self::TransomLg => bb / cs + lwl,
            _               => lwl,
        }
    }
}

#[cfg(test)] // SternType {{{1
mod stern_type {
    use super::*;
    use crate::test_support::*;

    // Test wp_calc {{{2
    macro_rules! test_wp_calc {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, stern) = $value;

                    assert_eq!(expected, stern.wp_calc());
                }
            )*
        }
    }

    test_wp_calc! {
        // name:                 (factors, stern)
        wp_calc_transom_sm: ((0.262, 0.79), SternType::TransomSm),
        wp_calc_transom_lg: ((0.262, 0.81), SternType::TransomLg),
        wp_calc_cruiser:    ((0.262, 0.76), SternType::Cruiser),
        wp_calc_round:      ((0.262, 0.76), SternType::Round),
    }

    // Test leff {{{2
    macro_rules! test_leff {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, stern) = $value;

                    let lwl = 500.0; let bb = 50.0; let cs = 0.2563;
                    assert_eq!(expected, to_place(stern.leff(lwl, bb, cs), 2));
                }
            )*
        }
    }

    test_leff! {
        // name:         (leff, stern, fuel, year)
        leff_transom_lg: (695.08, SternType::TransomLg),
        leff_transom_sm: (597.54, SternType::TransomSm),
        leff_cruiser:    (500.0, SternType::Cruiser),
        leff_round:      (500.0, SternType::Round),
    }
}


// BowType {{{1
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum BowType {
    /// Ram bow, including length.
    Ram(f64),
    /// Bulbous, straight bow.
    BulbStraight,
    /// Bulbous, forward bow.
    BulbForward,
    #[default]
    /// Normal bow (default).
    Normal,
}

impl From<String> for BowType {
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for BowType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::BulbStraight,
            "2" => Self::BulbForward,
            "3" => Self::Ram(0.0),
            "0" | _ => Self::Normal,
        }
    }
}

impl fmt::Display for BowType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Ram(_)       => "a ram bow",
            Self::BulbStraight => "a straight bulbous bow",
            Self::BulbForward  => "an extended bulbous bow",
            Self::Normal       => "a normal bow",
        })
    }
}

impl BowType {
    // ram_len {{{2
    /// Return length of the ram.
    ///
    pub fn ram_len(&self) -> f64 {
        match self {
            Self::Ram(len) => *len,
            _              => 0.0,
        }
    }
}


