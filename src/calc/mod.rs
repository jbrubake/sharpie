pub mod armor;
pub mod asw;
pub mod battery;
pub mod engine;
pub mod freeboard;
pub mod hull;
pub mod hull_draw;
pub mod mines;
pub mod ship;
pub mod torpedoes;
pub mod units;
pub mod weights;
pub mod macros;

/// Earliest valid year for a ship.
pub const YEAR_MIN: u32 = 1850;
/// Latest valid year for a ship.
pub const YEAR_MAX: u32 = 1950;

/// File extension for sharpie files.
pub const SHIP_FILE_EXT: &str = "ship";
/// File extension for SpringSharp files.
pub const SS_SHIP_FILE_EXT: &str = "sship";

pub use armor::*;
pub use asw::*;
pub use battery::*;
pub use engine::*;
pub use freeboard::*;
pub use hull::*;
pub use mines::*;
pub use ship::*;
pub use torpedoes::*;
pub use units::*;
pub use weights::*;

// Testing support {{{1
#[cfg(test)]
pub(crate) mod test_support {
    use crate::calc::Ship;
    use crate::calc::Hull;
    use crate::calc::Engine;
    use crate::calc::engine::{BoilerType, DriveType, FuelType};
    use crate::calc::units::{Measurement, Units, UnitType::*};

    // Round a float to a given number of digits
    //
    // This makes it much easier to test results that
    // are floats.
    pub fn to_place(n: f64, digits: u32) -> f64 {
        let mult = 10_u32.pow(digits) as f64;

        (n * mult).round() / mult
    }

    pub fn test_ship() -> Ship {
        let mut hull = Hull::default();

        hull.set_lwl(100.0, Units::Imperial);
        hull.set_d(1000.0);

        hull.b  = Measurement::new(50.0, LengthLong, Units::Imperial);
        hull.bb = Measurement::new(hull.b.imp(), LengthLong, Units::Imperial);
        hull.t  = Measurement::new(10.0, LengthLong, Units::Imperial);

        hull.freeboard.fc_len = 0.2;
        hull.freeboard.fc_fwd = Measurement::new(10.0, LengthLong, Units::Imperial);
        hull.freeboard.fc_aft = hull.freeboard.fc_fwd;

        hull.freeboard.fd_len = 0.3;
        hull.freeboard.fd_fwd = hull.freeboard.fc_fwd;
        hull.freeboard.fd_aft = hull.freeboard.fc_fwd;

        hull.freeboard.ad_fwd = hull.freeboard.fc_fwd;
        hull.freeboard.ad_aft = hull.freeboard.fc_fwd;

        hull.freeboard.qd_len = 0.15;
        hull.freeboard.qd_fwd = hull.freeboard.fc_fwd;
        hull.freeboard.qd_aft = hull.freeboard.fc_fwd;

        let mut engine = Engine::default();
        engine.set_shafts(2, &mut hull);
        engine.year     = 1920;
        engine.fuel     = FuelType::Oil;
        engine.boiler   = BoilerType::Turbine;
        engine.drive    = DriveType::Geared;
        engine.vmax     = 30.0;
        engine.vcruise  = 20.0;
        engine.range    = 10000;

        let mut ship = Ship::default();
        ship.hull  = hull;
        ship.engine = engine;
        ship
    }
}
