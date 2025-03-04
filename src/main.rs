use sharpie::Ship;
use sharpie::{BowType, SternType};
use sharpie::{FuelType, BoilerType, DriveType};

use std::env;

fn main() -> Result<(),std::io::Error> {
    let args: Vec<String> = env::args().collect();

    // Simple command line for testing:
    //
    // --load FILE : load a ship FILE
    // FILE : save ship to FILE (edit the ship here in the source)
    //
    let ship = match args[1].as_str() {
        "--load" => Ship::load(args[2].clone()).unwrap(),
        &_ => {
            let mut ship = Ship::default();

            ship.name = String::from("NAME");
            ship.country = String::from("COUNTRY");
            ship.kind = String::from("KIND");
            ship.year = 1920;

            // Configure Hull {{{
            ship.hull.set_d(5000.0);
            ship.hull.set_lwl(500.0);
            ship.hull.b = 50.0;
            ship.hull.bb = 50.0;
            ship.hull.t = 10.0;

            ship.hull.fc_len = 0.25;
            ship.hull.fc_fwd = 10.0;
            ship.hull.fc_aft = 1.0;

            ship.hull.fd_len = 0.25;
            ship.hull.fd_fwd = 10.0;
            ship.hull.fd_aft = 1.0;

            ship.hull.ad_fwd = 10.0;
            ship.hull.ad_aft = 1.0;

            ship.hull.qd_len = 0.25;
            ship.hull.qd_fwd = 10.0;
            ship.hull.qd_aft = 1.0;

            ship.hull.bow_type = BowType::Ram(3.0);
            ship.hull.stern_type = SternType::TransomLg;
            // }}}

            // Configure Engine {{{
            ship.engine.year = 1920;
            ship.engine.fuel = FuelType::Coal;
            ship.engine.boiler = BoilerType::Turbine;
            ship.engine.drive = DriveType::Direct;
            ship.engine.vmax = 20.0;
            ship.engine.vcruise = 10.0;
            ship.engine.range = 1000;
            ship.engine.shafts = 1;
            // }}}

            // Configure Armor {{{
            ship.armor.incline = 10.0;

            ship.armor.main.thick = 1.0;
            ship.armor.main.len = 100.0;
            ship.armor.main.hgt = 1.0;

            ship.armor.end.thick = 1.0;
            ship.armor.end.len = 100.0;
            ship.armor.end.hgt = 1.0;

            ship.armor.upper.thick = 1.0;
            ship.armor.upper.len = 100.0;
            ship.armor.upper.hgt = 1.0;

            ship.armor.bulge.thick = 1.0;
            ship.armor.bulge.len = 50.0;
            ship.armor.bulge.hgt = 5.0;

            ship.armor.bulkhead.thick = 1.0;
            ship.armor.bulkhead.len = 50.0;
            ship.armor.bulkhead.hgt = 5.0;

            ship.armor.ct_fwd.thick = 1.0;
            ship.armor.ct_aft.thick = 1.0;
            // }}}

            let _ = ship.save(args[1].clone());

            ship
        },
    };

    ship.report();

    #[cfg(debug_assertions)]
    {
        println!("");
        println!("Internal values");
        println!("---------------");
        ship.internals();
    }

    Ok(())
}

