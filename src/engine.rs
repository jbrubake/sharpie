use serde::{Serialize, Deserialize};

use crate::{FuelType, BoilerType, DriveType};

// Engine {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Engine {
    pub year: u32,

    pub fuel: FuelType,
    pub boiler: BoilerType,
    pub drive: DriveType,
    pub factor: u32,

    pub vmax: f64,
    pub vcruise: f64,
    pub range: u32,

    // Because SS3 uses shafts to calculate Cwp, if shafts < 2
    // Hull.boxy should be set to true as well
    pub shafts: u32,

    pub pct_coal: f64,
}

// Engine Implementation {{{1
impl Engine {
    const RANGE: f64 = 7000.0;
    // hp {{{2
    pub fn hp(&self, v: f64, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        let len_hp =
            if v <= 15.0 {
                lwl - (leff - lwl) 
            } else if v >= 25.0 {
                leff
            } else {
                (leff - lwl) * ((v - 20.0) / 5.0) + lwl
            };

        if len_hp == 0.0 { return 0.0; }

        let hp = (d.powf(2.0/3.0) / len_hp * cs * v.powf(4.0) + 0.01 * ws * v.powf(1.83)) *
            v / 184.1666667;

        if self.year < 1890 {
            hp * (1.0 + (1890 - self.year) as f64 / 100.0)
        } else {
            hp
        }
    }

    // hp_max {{{2
    pub fn hp_max(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.hp(self.vmax, d, lwl, leff, cs, ws)
    }

    // hp_cruise {{{2
    pub fn hp_cruise(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.hp(self.vcruise.min(self.vmax), d, lwl, leff, cs, ws)
    }

    // rf {{{2
    fn rf(v: f64, ws: f64) -> f64 {
        0.01 * ws * v.powf(1.83) 
    }

    // rf_max {{{2
    pub fn rf_max(&self, ws: f64) -> f64 {
        Engine::rf(self.vmax, ws)
    }

    // rf_cruise {{{2
    pub fn rf_cruise(&self, ws: f64) -> f64 {
        Engine::rf(self.vcruise, ws)
    }

    // rw {{{2
    fn rw(v: f64, d: f64, lwl: f64, cs: f64) -> f64 {
        if lwl == 0.0 { return 0.0; }
        d.powf(2.0/3.0) / lwl * cs * v.powf(4.0) 
    }

    // rw_max {{{2
    pub fn rw_max(&self, d: f64, lwl: f64, cs: f64) -> f64 {
        Engine::rw(self.vmax, d, lwl, cs)
    }

    // rw_cruise {{{2
    pub fn rw_cruise(&self, d: f64, lwl: f64, cs: f64) -> f64 {
        Engine::rw(self.vcruise, d, lwl, cs)
    }

    // pw {{{2
    pub fn pw(rw: f64, rf: f64) -> f64 {
        if rw * rf == 0.0 { return 0.0; }
        rw / (rw + rf) * 100.0
    }

    // pw_max {{{2
    pub fn pw_max(&self, d: f64, lwl: f64, cs: f64, ws: f64) -> f64 {
        Engine::pw(self.rw_max(d, lwl, cs), self.rf_max(ws))
    }

    // pw_cruise {{{2
    pub fn pw_cruise(&self, d: f64, lwl: f64, cs: f64, ws: f64) -> f64 {
        Engine::pw(self.rw_cruise(d, lwl, cs), self.rf_cruise(ws))
    }

    // bunker {{{2
    /// Calculate normal bunker
    ///
    // TODO: this might make more sense in Ship
    pub fn bunker(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        let bunker = self.range as f64 / (1.0 + 0.4 * (1.0 - self.pct_coal as f64));
        let bunker = bunker /
            // if self.engine ~= "reciprocating" {
                // 1.0 - (1910 - self.year) as f64 / 70.0 
            // } else if self.year < 1898 {
            if self.year < 1898 {
                1.0 - (1910 - self.year) as f64 / 70.0
            } else if self.year < 1920 {
                1.0 + (self.year - 1910) as f64 / 20.0
            } else if self.year < 1950 {
                1.5 + (self.year - 1920) as f64 / 60.0
            } else {
                2.0
            };

        bunker / 1.8 / self.hp_cruise(d, lwl, leff, cs, ws) * Engine::RANGE as f64 * self.vcruise * 0.1 + d * 0.005
    }

    // bunker_max {{{2
    pub fn bunker_max(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.bunker(d, lwl, leff, cs, ws) * 1.8
    }

    // d_factor {{{2
    pub fn d_factor(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64, wgt_borne: f64, wgt_armor: f64, wgt_misc: f64) -> f64 {
        (d / self.d_engine(d, lwl, leff, cs, ws) + 8.0 * wgt_borne + wgt_armor + wgt_misc).min(10.0)
    }

    // num_engines {{{2
    pub fn num_engines(&self) -> u32 {
        u8::count_ones(self.boiler.bits())
    }

    // d_engine {{{2
    pub fn d_engine(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        let a = 1.0; // lookup(year_engine, year_data[year], year_data[eng_wgt_{simple, complex, other}])
        let b = 1.0; // lookup(year_engine, year_data[year], year_data[eng_wgt_early])
        self.hp_max(d, lwl, leff, cs, ws) / a / self.num_engines() as f64 * (1.1 - self.pct_coal as f64 / 10.0) / b
    }

    // wgt {{{2
    pub fn wgt(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64, wgt_borne:f64, wgt_armor: f64, wgt_misc: f64) -> f64 {
        let d_factor = self.d_factor(d, lwl, leff, cs, ws, wgt_borne, wgt_armor, wgt_misc);

        let p =
            if d < 5000.0 && d >= 600.0 && d_factor < 1.0 {
                1.0 - d / 5000.0
            } else if d < 600.0 && d_factor < 1.0 {
                0.88
            } else {
                0.0
            };

        (self.d_engine(d, lwl, leff, cs, ws) / 2.0) * d_factor.powf(p)
    }

    // new {{{2
    pub fn new() -> Engine {
        Default::default()
    }
}

#[cfg(test)] // Engine {{{1
mod test {
    use super::*;

    // Test hp {{{2
    macro_rules! test_hp {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 627.735687; let cs = 0.3914332884; let ws = 26000.0;
                    let (expected, v, year) = $value;
                    let mut eng = Engine::default();
                    eng.year = year;
                    println!("{}", eng.hp(v, d, lwl, leff, cs, ws));
                    assert!(expected == eng.hp(v, d, lwl, leff, cs, ws));
                }
            )*
        }
    }
    test_hp! {
        // name: (hp, v, year)
        hp_zero: (0.0, 0.0, 1920),
        hp_year_early: (10872.10266257719, 20.0, 1889),
        hp_year_late: (10764.458081759594, 20.0, 1890),
    }

    // Test hp_max {{{2
    macro_rules! test_hp_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 627.735687; let cs = 0.3914332884; let ws = 26000.0;

                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise; eng.year = 1920;

                    println!("{}", eng.hp_max(d, lwl, leff, cs, ws));
                    assert!(expected == eng.hp_max(d, lwl, leff, cs, ws));
                }
            )*
        }
    }
    test_hp_max! {
        // name: (hp, vmax, vcruise)
        hp_max_test: (10764.458081759594, 20.0, 10.0),
    }

    // Test hp_cruise {{{2
    macro_rules! test_hp_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 627.735687; let cs = 0.3914332884; let ws = 26000.0;

                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise; eng.year = 1920;

                    println!("{}", eng.hp_cruise(d,lwl,leff,cs,ws));
                    assert!(expected == eng.hp_cruise(d, lwl, leff, cs, ws));
                }
            )*
        }
    }
    test_hp_cruise! {
        // name: (hp, vmax, vcruise)
        hp_cruise_cruise_is_less: (1121.4158170637938, 20.0, 10.0),
        hp_cruise_max_is_less: (4274.516765999324, 15.0, 20.0),
    }

    // Test rf {{{2
    macro_rules! test_rf {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, v, ws) = $value;
                    assert!(expected == Engine::rf(v, ws));
                }
            )*
        }
    }
    test_rf! {
        // name: (rf, v, ws)
        rf_v_zero: (0.0, 0.0, 1.0),
        rf_ws_zero: (0.0, 1.0, 0.0),
        rf_test: (2028.2489261759456, 10.0, 3000.0),
    }

    // Test rw {{{2
    macro_rules! test_rw {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, v, d, lwl, cs) = $value;
                    assert!(expected == Engine::rw(v, d, lwl, cs));
                }
            )*
        }
    }
    test_rw! {
        // name: (rw, v, d, lwl, cs)
        rw_v_zero: (0.0, 0.0, 1.0, 1.0, 1.0),
        rw_d_zero: (0.0, 1.0, 0.0, 1.0, 1.0),
        rw_lwl_zero: (0.0, 1.0, 1.0, 0.0, 1.0),
        rw_cs_zero: (0.0, 1.0, 1.0, 1.0, 0.0),
        rw_test: (799.9999999999999, 10.0, 1000.0, 500.0, 0.4),
    }

    // Test pw {{{2
    macro_rules! test_pw {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, rw, rf) = $value;
                    assert!(expected == Engine::pw(rw, rf));
                }
            )*
        }
    }
    test_pw! {
        // name: (pw, rw, rf)
        pw_rw_zero: (0.0, 0.0, 1.0),
        pw_rf_zero: (0.0, 1.0, 0.0),
        pw_test: (28.57142857142857, 800.0, 2000.0),
    }

    // Test rf_max {{{2
    macro_rules! test_rf_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let ws = 3000.0;

                    assert!(expected == eng.rf_max(ws));
                }
            )*
        }
    }
    test_rf_max! {
        // name: (rf, vmax, vcruise)
        rf_max_test: (7211.176854461778, 20.0, 10.0),
    }

    // Test rf_cruise {{{2
    macro_rules! test_rf_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let ws = 3000.0;

                    assert!(expected == eng.rf_cruise(ws));
                }
            )*
        }
    }
    test_rf_cruise! {
        // name: (rf, vmax, vcruise)
        rf_cruise_test: (2028.2489261759456, 20.0, 10.0),
    }

    // Test rw_max {{{2
    macro_rules! test_rw_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4;

                    assert!(expected == eng.rw_max(d, lwl, cs));
                }
            )*
        }
    }
    test_rw_max! {
        // name: (rw, vmax, vcruise)
        rw_max_test: (37427.42704912468, 20.0, 10.0),
    }

    // Test rw_cruise {{{2
    macro_rules! test_rw_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4;

                    assert!(expected == eng.rw_cruise(d, lwl, cs));
                }
            )*
        }
    }
    test_rw_cruise! {
        // name: (rw, vmax, vcruise)
        rw_cruise_test: (2339.2141905702924, 20.0, 10.0),
    }

    // Test pw_max {{{2
    macro_rules! test_pw_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4; let ws = 3000.0;

                    assert!(expected == eng.pw_max(d, lwl, cs, ws));
                }
            )*
        }
    }
    test_pw_max! {
        // name: (pw, vmax, vcruise)
        pw_max_test: (83.84542475827206, 20.0, 10.0),
    }

    // Test pw_cruise {{{2
    macro_rules! test_pw_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4; let ws = 3000.0;

                    assert!(expected == eng.pw_cruise(d, lwl, cs, ws));
                }
            )*
        }
    }
    test_pw_cruise! {
        // name: (pw, vmax, vcruise)
        pw_cruise_test: (53.56002164279313, 20.0, 10.0),
    }

}


