use serde::{Serialize, Deserialize};

use crate::{FuelType, BoilerType, DriveType};

// Engine {{{1
/// The ship's engine and speed and range characteristics.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Engine {
    /// Year engine built.
    pub year: u32,

    /// Type of fuel.
    pub fuel: FuelType,
    /// Type of steam boilers.
    pub boiler: BoilerType,
    /// Type of engine drive.
    pub drive: DriveType,

    /// TODO: Unimplemented
    pub factor: u32,

    /// Maximum speed (not maximum trial speed).
    pub vmax: f64,
    /// Crusing speed.
    pub vcruise: f64,
    /// Maximum range at crusing speed.
    pub range: u32,

    /// Number of properllor shafts.
    ///
    // TODO: If this is < 2, the 'boxy' field in the corresponding Hull should be set to true.
    pub shafts: u32,

    /// Percentage of bunker weight devoted to coal.
    pub pct_coal: f64,
}

impl Engine { // {{{2
    /// XXX: self.range is divided by this in bunker()
    const RANGE: f64 = 7000.0;

    // hp {{{3
    /// Horsepower required to achieve a given speed.
    ///
    fn hp(&self, v: f64, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
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

        hp * if self.year < 1890 {
                1.0 + (1890 - self.year) as f64 / 100.0
            } else {
                1.0
            }
    }

    // hp_max {{{3
    /// Horsepower required to achieve maximum speed.
    ///
    pub fn hp_max(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.hp(self.vmax, d, lwl, leff, cs, ws)
    }

    // hp_cruise {{{3
    /// Horsepower required to achieve crusing speed.
    ///
    // XXX: Should vcruise be set to a minimum somewhere else?
    pub fn hp_cruise(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.hp(self.vcruise.min(self.vmax), d, lwl, leff, cs, ws)
    }

    // rf {{{3
    /// Friction resistance at a given speed.
    ///
    fn rf(v: f64, ws: f64) -> f64 {
        0.01 * ws * v.powf(1.83) 
    }

    // rf_max {{{3
    /// Friction resistance at maximum speed.
    ///
    pub fn rf_max(&self, ws: f64) -> f64 {
        Self::rf(self.vmax, ws)
    }

    // rf_cruise {{{3
    /// Friction resistance at crusing speed.
    ///
    pub fn rf_cruise(&self, ws: f64) -> f64 {
        Self::rf(self.vcruise, ws)
    }

    // rw {{{3
    /// Wave resistance at a given speed.
    ///
    fn rw(v: f64, d: f64, lwl: f64, cs: f64) -> f64 {
        if lwl == 0.0 { return 0.0; }
        d.powf(2.0/3.0) / lwl * cs * v.powf(4.0) 
    }

    // rw_max {{{3
    /// Wave resistance at maximum speed.
    ///
    pub fn rw_max(&self, d: f64, lwl: f64, cs: f64) -> f64 {
        Self::rw(self.vmax, d, lwl, cs)
    }

    // rw_cruise {{{3
    /// Wave resistance at crusing speed.
    ///
    pub fn rw_cruise(&self, d: f64, lwl: f64, cs: f64) -> f64 {
        Self::rw(self.vcruise, d, lwl, cs)
    }

    // pw {{{3
    /// Power to wave ratio.
    ///
    fn pw(rw: f64, rf: f64) -> f64 {
        match rw + rf {
            0.0 => 0.0, // Protect against divide by 0
            _   => rw / (rw + rf),
        }
    }

    // pw_max {{{3
    /// Power to wave ratio at maximum speed.
    ///
    pub fn pw_max(&self, d: f64, lwl: f64, cs: f64, ws: f64) -> f64 {
        Self::pw(self.rw_max(d, lwl, cs), self.rf_max(ws))
    }

    // pw_cruise {{{3
    /// Power to wave ratio at cruising speed.
    ///
    pub fn pw_cruise(&self, d: f64, lwl: f64, cs: f64, ws: f64) -> f64 {
        Self::pw(self.rw_cruise(d, lwl, cs), self.rf_cruise(ws))
    }

    // bunker {{{3
    /// Bunkerage weight.
    ///
    pub fn bunker(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        if self.vcruise == 0.0 { return 0.0; } // catch divide by zero

        let bunker = self.range as f64 / (1.0 + 0.4 * (1.0 - self.pct_coal as f64));
        let bunker = bunker / self.boiler.bunker_factor(self.year);

        bunker /
            (1.8 / self.hp_cruise(d, lwl, leff, cs, ws) * Self::RANGE as f64 * self.vcruise * 0.1) +
            d * 0.005
    }

    // bunker_max {{{3
    /// Bunkerage weight at maximum displacement.
    ///
    pub fn bunker_max(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        self.bunker(d, lwl, leff, cs, ws) * 1.8
    }


    // num_engines {{{3
    /// Number of steam engines.
    ///
    pub fn num_engines(&self) -> u32 {
        self.boiler.num_engines()
    }

    // d_engine {{{3
    /// Displacement of the engine.
    ///
    pub fn d_engine(&self, d: f64, lwl: f64, leff: f64, cs: f64, ws: f64) -> f64 {
        let factor = self.boiler.d_engine_factor(self.year, self.fuel.clone());
        let early =
            if self.year <= 1889 {
                1.0 + (1890 - self.year) as f64 / 100.0
            } else {
                1.0
            };

        (
            self.hp_max(d, lwl, leff, cs, ws) /
            (factor /self.num_engines() as f64 * (1.1 - self.pct_coal / 10.0))
        ) / early
    }

    // new {{{2
    pub fn new() -> Engine {
        Default::default()
    }
}

// Testing Engine {{{2
#[cfg(test)]
mod engine {
    use super::*;
    use crate::test_support::*;

    // Test hp {{{3
    macro_rules! test_hp {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 500.0; let cs = 0.2576602375; let ws = 30050.0;
                    let (expected, v, year) = $value;
                    let mut eng = Engine::default();
                    eng.year = year;

                    assert!(expected == to_place(eng.hp(v, d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_hp! {
        // name:                     (hp, v, year)
        hp_v_le_15:                  (4096.44, 15.0, 1900),
        hp_v_ge_25:                  (22740.39, 25.0, 1900),
        hp_v_other_low:              (5029.43, 16.0, 1900),
        hp_year_early:               (10566.98, 20.0, 1889),
        hp_year_late:                (10462.36, 20.0, 1891),
        hp_v_other_hi_year_boundary: (19655.91, 24.0, 1890),
    }

    // Test hp_max {{{3
    macro_rules! test_hp_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 500.0; let cs = 0.2576602375; let ws = 30050.0;

                    let (expected, vmax) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.year = 1920;

                    assert!(expected == to_place(eng.hp_max(d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_hp_max! {
        // name:     (hp, vmax)
        hp_max_test: (10462.36, 20.0),
    }

    // Test hp_cruise {{{3
    macro_rules! test_hp_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 5000.0; let lwl = 500.0; let leff = 500.0; let cs = 0.2576602375; let ws = 30050.0;

                    let (expected, vmax, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax; eng.vcruise = vcruise; eng.year = 1920;

                    assert!(expected == to_place(eng.hp_cruise(d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_hp_cruise! {
        // name:                  (hp, vmax, vcruise)
        hp_cruise_cruise_is_less: (1184.96, 20.0, 10.0),
        hp_cruise_max_is_less:    (1184.96, 10.0, 20.0),
    }

    // Test rf {{{3
    macro_rules! test_rf {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, v, ws) = $value;

                    assert!(expected == to_place(Engine::rf(v, ws), 2));
                }
            )*
        }
    }
    test_rf! {
        // name:    (rf, v, ws)
        rf_test:    (2028.25, 10.0, 3000.0),
    }

    // Test rf_max {{{3
    macro_rules! test_rf_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax;
                    let ws = 3000.0;

                    assert!(expected == to_place(eng.rf_max(ws), 2));
                }
            )*
        }
    }
    test_rf_max! {
        // name:     (rf, vmax)
        rf_max_test: (7211.18, 20.0),
    }

    // Test rf_cruise {{{3
    macro_rules! test_rf_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vcruise = vcruise;
                    let ws = 3000.0;

                    assert!(expected == to_place(eng.rf_cruise(ws), 2));
                }
            )*
        }
    }
    test_rf_cruise! {
        // name:        (rf, vcruise)
        rf_cruise_test: (2028.25, 10.0),
    }

    // Test rw {{{3
    macro_rules! test_rw {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, lwl) = $value;
                    let v = 10.0; let d = 1000.0; let cs = 0.1234;

                    assert!(expected == to_place(Engine::rw(v, d, lwl, cs), 2));
                }
            )*
        }
    }
    test_rw! {
        // name:     (rw, lwl)
        rw_lwl_zero: (0.0, 0.0),
        rw_test:     (1234.0, 100.0),
    }

    // Test rw_max {{{3
    macro_rules! test_rw_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4;

                    assert!(expected == to_place(eng.rw_max(d, lwl, cs), 2));
                }
            )*
        }
    }
    test_rw_max! {
        // name:     (rw, vmax)
        rw_max_test: (37427.43, 20.0),
    }

    // Test rw_cruise {{{3
    macro_rules! test_rw_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4;

                    assert!(expected == to_place(eng.rw_cruise(d, lwl, cs), 2));
                }
            )*
        }
    }
    test_rw_cruise! {
        // name:        (rw, vcruise)
        rw_cruise_test: (2339.21, 10.0),
    }


    // Test pw {{{3
    macro_rules! test_pw {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, rw, rf) = $value;

                    assert!(expected == to_place(Engine::pw(rw, rf), 2));
                }
            )*
        }
    }
    test_pw! {
        // name:    (pw, rw, rf)
        pw_both_eq_0: (0.0, 0.0, 0.0),
        pw_test:    (0.29, 800.0, 2000.0),
    }

    // Test pw_max {{{3
    macro_rules! test_pw_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vmax) = $value;
                    let mut eng = Engine::default();
                    eng.vmax = vmax;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4; let ws = 3000.0;

                    assert!(expected == to_place(eng.pw_max(d, lwl, cs, ws), 2));
                }
            )*
        }
    }
    test_pw_max! {
        // name:     (pw, vmax)
        pw_max_test: (0.84, 20.0),
    }

    // Test pw_cruise {{{3
    macro_rules! test_pw_cruise {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.vcruise = vcruise;
                    let d = 5000.0; let lwl = 500.0; let cs = 0.4; let ws = 3000.0;

                    assert!(expected == to_place(eng.pw_cruise(d, lwl, cs, ws), 2));
                }
            )*
        }
    }
    test_pw_cruise! {
        // name:        (pw, vcruise)
        pw_cruise_test: (0.54, 10.0),
    }

    // Test bunker {{{3
    macro_rules! test_bunker {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, range, pct_coal, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.range = range;
                    eng.pct_coal = pct_coal;
                    eng.vcruise = vcruise;
                    eng.vmax = vcruise; // vmax must be >= vcruise or hp_cruise will fail

                    eng.boiler = BoilerType::Turbine;
                    eng.year = 1920;
                    let lwl = 500.0; let leff = 500.0;
                    let cs = 0.2563; let ws = 12000.0; let d = 1000.0;

                    assert!(expected == to_place(eng.bunker(d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_bunker! {
        // name:           (bunker, range, pct_coal, vcruise)
        bunker_pct_coal_0: (29.78, 1000, 1.0, 10.0),
        bunker_pct_coal_1: (22.70, 1000, 0.0, 10.0),
        bunker_range_0:    (5.0, 0, 0.5, 10.0),
        bunker_vcruise_0:  (0.0, 1000, 0.5, 0.0),
    }

    // Test bunker_max {{{3
    macro_rules! test_bunker_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, range, pct_coal, vcruise) = $value;
                    let mut eng = Engine::default();
                    eng.range = range;
                    eng.pct_coal = pct_coal;
                    eng.vcruise = vcruise;
                    eng.vmax = vcruise; // vmax must be >= vcruise or hp_cruise will fail

                    eng.boiler = BoilerType::Turbine;
                    eng.year = 1920;
                    let lwl = 500.0; let leff = 500.0;
                    let cs = 0.2563; let ws = 12000.0; let d = 1000.0;

                    println!("{}", to_place(eng.bunker_max(d, lwl, leff, cs, ws), 2));
                    assert!(expected == to_place(eng.bunker_max(d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_bunker_max! {
        // name:     (bunker, range, pct_coal, vcruise)
        bunker_max: (40.86, 1000, 0.0, 10.0),
    }

    // Test d_engine {{{3
    macro_rules! test_d_engine {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, year) = $value;
                    let mut eng = Engine::default();
                    eng.year = year;

                    eng.pct_coal = 0.5;
                    eng.vmax = 10.0;
                    eng.boiler = BoilerType::Turbine;
                    eng.fuel = FuelType::Oil;
                    let lwl = 500.0; let leff = 500.0;
                    let cs = 0.2563; let ws = 12000.0; let d = 1000.0;

                    println!("{}", to_place(eng.d_engine(d, lwl, leff, cs, ws), 2));
                    assert!(expected == to_place(eng.d_engine(d, lwl, leff, cs, ws), 2));
                }
            )*
        }
    }
    test_d_engine! {
        // name:     (d_engine, year)
        d_engine_early: (168.32, 1889),
        d_engine_late: (165.21, 1890),
    }
}

