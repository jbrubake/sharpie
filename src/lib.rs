mod hull;
mod armor;
mod engine;
mod weapons;
mod weights;

use hull::Hull;
use armor::Armor;
use engine::Engine;
use weapons::{Battery, Torpedoes, Mines, ASW};
use weights::MiscWgts;

use bitflags::bitflags;
use num_format::{Locale, ToFormattedString};
use serde::{Serialize, Deserialize};

use std::error::Error;
use std::fs;

const POUND2TON: f64 = 2240.0;

// Ship {{{1
#[derive(Serialize, Deserialize, Debug)]
pub struct Ship {
    pub name: String,
    pub country: String,
    pub kind: String,
    pub year: u32,

    pub hull: Hull,
    pub armor: Armor,
    pub engine: Engine,
    pub batteries: Vec<Battery>,
    pub torps: Vec<Torpedoes>,
    pub mines: Mines,
    pub asw: Vec<ASW>,
    pub wgts: MiscWgts,
}

impl Default for Ship { // {{{1
    fn default() -> Ship {
        Ship {
            name: String::from("NAME"),
            country: String::from("COUNTRY"),
            kind: String::from("TYPE"),
            year: 1920,

            hull: Hull::default(),
            wgts: MiscWgts::default(),
            engine: Engine::default(),
            armor: Armor::default(),
            torps: vec![Torpedoes::default(), Torpedoes::default()],
            mines: Mines::default(),
            asw: vec![ASW::default(), ASW::default()],
            batteries: vec![
                Battery::default(),
                Battery::default(),
                Battery::default(),
                Battery::default(),
                Battery::default(),
            ],
        }
    }
}

// Ship Implementation {{{1
impl Ship {
    // year_adj {{{2
    pub fn year_adj(year: u32) -> f64 {
             if year <= 1890 { 1.0 - (1890 - year) as f64 / 66.666664 }
        else if year <= 1950 { 1.0 }
        else                 { 0.0 }
    }

    // deck_space {{{2
    pub fn deck_space(&self) -> f64 {
        let mut space = 0.0;
        for w in self.torps.iter() {
            space += w.deck_space(self.hull.b); 
        }

        space / self.hull.wp()
    }

    // hull_space {{{2
    pub fn hull_space(&self) -> f64 {
        let mut space = 0.0;
        for w in self.torps.iter() {
            space += w.hull_space(); 
        }
        // FIXME: Why does FT3_PER_TON_SEA need multipled out of D?
        space / self.hull.d() * Hull::FT3_PER_TON_SEA
    }
    // wgt_load {{{2
    pub fn wgt_load(&self) -> f64 {
        self.hull.d() * 0.02 + self.engine.bunker(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()) // + wgt_mag
    }

    // d_lite {{{2
    pub fn d_lite(&self) -> f64 {
        self.hull.d() - self.wgt_load()
    }

    // d_std {{{2
    pub fn d_std(&self) -> f64 {
        self.hull.d() - self.engine.bunker(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()) // + wgt_mag
    }

    // d_max {{{2
    pub fn d_max(&self) -> f64 {
        self.hull.d() + 0.8 * self.engine.bunker(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws())
    }

    // t_max {{{2
    pub fn t_max(&self) -> f64 {
        self.hull.t_calc(self.d_max())
    }

    // cb_max {{{2
    pub fn cb_max(&self) -> f64 {
        self.hull.cb_calc(self.d_max())
    }

    // crew_max {{{2
    /// Calculate maximum crew size
    ///
    pub fn crew_max(&self) -> u32 {
        (self.hull.d().powf(0.75) * 0.56) as u32
    }

    // crew_min {{{2
    /// Calculate minimum crew size
    ///
    pub fn crew_min(&self) -> u32 {
        (self.crew_max() as f64 * 0.7692) as u32
    }

    // new {{{2
    /// Create a new ship
    ///
    pub fn new() -> Ship {
        Default::default()
    }

    // load {{{2
    /// Load ship from a file
    ///
    pub fn load(p: String) -> Result<Ship, Box<dyn Error>> {
        let s = fs::read_to_string(p)?;
        let ship = serde_json::from_str(&s)?;

        Ok(ship)
    }

    // save {{{2
    /// Save ship to a file
    ///
    pub fn save(&self, p: String) -> Result<(), Box<dyn Error>> {

        let s = serde_json::to_string(&self).unwrap();
        fs::write(p, s)?;

        Ok(())
    }

    // report {{{2
    /// Print report
    ///
    pub fn report(&self) {
        println!("{}, {} {} laid down {}", self.name, self.country, self.kind, self.year);
        println!("");

        println!("Displacment:"); // {{{3
        println!("\t{} t light; {} t standard; {} t normal; {} t full load",
            (self.d_lite() as u64).to_formatted_string(&Locale::en),
            (self.d_std() as u64).to_formatted_string(&Locale::en),
            (self.hull.d() as u64).to_formatted_string(&Locale::en),
            (self.d_max() as u64).to_formatted_string(&Locale::en));
        println!("");

        println!("Dimensions: Length (overall / waterline) × beam × draft (normal/deep)"); // {{{3
        println!("\t({:.2} ft / {:.2} ft) × {:.2} ft × ({:.2} / {:.2} ft)",
            self.hull.loa(),
            self.hull.lwl(),
            self.hull.b,
            self.hull.t,
            self.t_max());
        println!("");

        println!("Armor:"); // {{{3
        println!(" - Belts:\tWidth (max)\tLength (avg)\tHeight (avg)");
        println!("\tMain:\t{:.2}\"\t{:.2} ft\t{:.2} ft",
            self.armor.main.thick,
            self.armor.main.len,
            self.armor.main.hgt);
        println!("\tEnds:\t{:.2}\"\t{:.2} ft\t{:.2} ft",
            self.armor.end.thick,
            self.armor.end.len,
            self.armor.end.hgt);
        println!("\tUpper:\t{:.2}\"\t{:.2} ft\t{:.2} ft",
            self.armor.upper.thick,
            self.armor.upper.len,
            self.armor.upper.hgt);
        println!("");
        println!("- Torpedo Bulkhead - DESCRIPTION");
        println!("\t\t{:.2}\"\t{:.2} ft\t{:.2} ft",
            self.armor.bulkhead.thick,
            self.armor.bulkhead.len,
            self.armor.bulkhead.hgt);
        println!("\tBeam between torpedo bulkheads {} ft",
            self.armor.beam_between);
        println!("");
        println!("- Conning towers: Forward {}\", Aft {}\"",
            self.armor.ct_fwd.thick,
            self.armor.ct_aft.thick);
        println!("");

        println!("Machinery:"); // {{{3
        println!("\t{}, {}, {} = {:.2} kts",
            "DRIVE",
            "n SHAFTS",
            "HP",
            self.engine.vmax);
        println!("\tRange {} nm at {:.2} kts",
            self.engine.range.to_formatted_string(&Locale::en),
            self.engine.vcruise);
        println!("\tBunker at max displacement = {} tons", self.engine.bunker_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()));
        println!("");

        println!("Complement:"); // {{{3
        println!("\t{} - {}",
            self.crew_min(),
            self.crew_max());
        println!("");

        println!("Cost:"); // {{{3
        println!("\t£{:.3} million / ${:.3} million",
            123_456.0/1_000_000.0, 1_234_456.0/1_000_000.0);
        println!("");

        println!("Distribution of weights at normal displacement:"); // {{{3
        println!("");

        println!("Overall survivability and seakeeping ability:"); // {{{3
        println!("");

        println!("Hull form characteristics:"); // {{{3
        println!("\tBlock coefficient (normal/deep): {:.2} / {:.2}",
            self.hull.cb(), self.cb_max());
        println!("\tLength to Beam Ratio: {:.2} : 1",
            self.hull.len2beam());
        println!("\t'Natural speed' for length: {:.2} kts",
            self.hull.vn());
        println!("\tBow angle (Positive = bow angles forward): {:.2} degrees",
            self.hull.bow_angle);
        println!("\tStern overhang: {:.2} ft",
            self.hull.stern_overhang);
        println!("\tFreeboard % = length of deck as a percentage of waterline length");
        println!("\t\t\tFore end, Aft end");
        println!("\
            \t- Forecastle:\t{:.2}%, {:.2} ft, {:.2} ft\n\
            \t- Forward Deck:\t{:.2}%, {:.2} ft, {:.2} ft\n\
            \t- Aft deck:\t{:.2}%, {:.2} ft, {:.2} ft\n\
            \t- Quarter deck:\t{:.2}%, {:.2} ft, {:.2} ft\n\
            \t- Average freeboard:\t\t{:.2} ft",
            self.hull.fc_len*100.0,   self.hull.fc_fwd, self.hull.fc_aft,
            self.hull.fd_len*100.0,   self.hull.fd_fwd, self.hull.fd_aft,
            self.hull.ad_len()*100.0, self.hull.ad_fwd, self.hull.ad_aft,
            self.hull.qd_len*100.0,   self.hull.qd_fwd, self.hull.qd_aft,
            self.hull.freeboard());
        println!("");

        println!("Ship space, strength and comments:"); // {{{3
        println!("\tWaterplane Area: {} Square feet",
            (self.hull.wp() as u64).to_formatted_string(&Locale::en));
    }

    // Print internal values {{{2
    #[cfg(debug_assertions)]
    pub fn internals(&self) {
        println!("Cs = {}", self.hull.cs());
        println!("Cm = {}", Hull::cm(self.hull.cb()));
        println!("Cp = {}", Hull::cp(self.hull.cb()));
        println!("Cwp = {}", self.hull.cwp());
        println!("WP = {}", self.hull.wp());
        println!("WS = {}", self.hull.ws());
        println!("Ts = {}", self.hull.ts());
        println!("");
        println!("Stem length = {}", self.hull.stem_len());
        if let BowType::Ram(len) = self.hull.bow_type { println!("Ram length = {}", len); }
        println!("Freeboard dist = {}", self.hull.dist());
        println!("Leff = {}", self.hull.leff());
        println!("");
        println!("Rf max = {}", self.engine.rf_max(self.hull.ws()));
        println!("Rf cruise = {}", self.engine.rf_cruise(self.hull.ws()));
        println!("Rw max = {}", self.engine.rw_max(self.hull.d(), self.hull.lwl(), self.hull.cs()));
        println!("Rw cruise = {}", self.engine.rw_cruise(self.hull.d(), self.hull.lwl(), self.hull.cs()));
        println!("Pw max = {}", self.engine.pw_max(self.hull.d(), self.hull.lwl(), self.hull.cs(), self.hull.ws()));
        println!("Pw cruise = {}", self.engine.pw_cruise(self.hull.d(), self.hull.lwl(), self.hull.cs(), self.hull.ws()));
        println!("");
        println!("hp max = {}", self.engine.hp_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()));
        println!("hp cruise = {}", self.engine.hp_cruise(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()));
        println!("");

        println!("{:?}", self.engine.fuel);
        println!("{:?}", self.engine.boiler);
        println!("{:?}", self.engine.drive);
        println!("num_engines = {}", self.engine.num_engines());
    }
}
#[cfg(test)] // Ship {{{1
mod ship {
    use super::*;

    // // Test wgt_load {{{2
    // macro_rules! test_wgt_load {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_wgt_load! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // // Test d_lite {{{2
    // macro_rules! test_d_lite {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_d_lite! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // // Test d_std {{{2
    // macro_rules! test_d_std {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_d_std! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // // Test d_max {{{2
    // macro_rules! test_d_max {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_d_max! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // // Test t_max {{{2
    // macro_rules! test_t_max {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_t_max! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // // Test cb_max {{{2
    // macro_rules! test_cb_max {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let mut ship = Ship::default();

    //                 let (expected, d) = $value;
    //                 ship.hull.set_d(d);
    //                 assert_eq!(expected, ship.crew_max());
    //             }
    //         )*
    //     }
    // }

    // test_cb_max! {
    //     // name:  (crew, d)
    //     crew_max_d_eq_zero: (0, 0.0),
    // }

    // Test crew_max {{{2
    macro_rules! test_crew_max {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut ship = Ship::default();

                    let (expected, d) = $value;
                    ship.hull.set_d(d);
                    assert_eq!(expected, ship.crew_max());
                }
            )*
        }
    }

    test_crew_max! {
        // name:  (crew, d)
        crew_max_d_eq_zero: (0, 0.0),
        crew_d_eq_1000: (99, 1000.0),
    }

    // Test crew_min {{{2
    macro_rules! test_crew_min {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut ship = Ship::default();

                    let (expected, d) = $value;
                    ship.hull.set_d(d);
                    assert_eq!(expected, ship.crew_min());
                }
            )*
        }
    }

    test_crew_min! {
        // name:  (crew, d)
        crew_min_d_eq_zero: (0, 0.0),
        crew_min_d_eq_1000: (76, 1000.0),
    }
}

// SternType {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum SternType {
    TransomSm,
    TransomLg,
    #[default]
    Cruiser,
    Round,
}

// BowType {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum BowType {
    Ram(f64), // Ram length
    BulbStraight,
    BulbForward,
    #[default]
    Normal,
}

// FuelType {{{1
bitflags! {
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct FuelType: u8 {
        const Coal     = 1 << 0;
        const Oil      = 1 << 1;
        const Diesel   = 1 << 2;
        const Gasoline = 1 << 3;
        const Battery  = 1 << 4;
    }
}

// BoilerType {{{1
bitflags! {
    #[derive(PartialEq, Serialize, Deserialize, Default, Debug)]
    pub struct BoilerType: u8 {
        const Simple  = 1 << 0;
        const Complex = 1 << 1;
        const Turbine = 1 << 2;
    }
}

// DriveType {{{1
bitflags! {
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct DriveType: u8 {
        const Direct    = 1 << 0;
        const Geared    = 1 << 1;
        const Electric  = 1 << 2;
        const Hydraulic = 1 << 3;
    }
}

// MineType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
pub enum MineType {
    #[default]
    SternRails,
    BowTubes,
    SternTubes,
    SideTubes,
}

// ASWType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
pub enum ASWType {
    #[default]
    SternRacks,
    Throwers,
    Hedgehogs,
    SquidMortars,
}

// TorpedoType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
pub enum TorpedoType {
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

// GunType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
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

impl GunType {
    // wgt_sm {{{2
    pub fn wgt_sm(&self) -> f64 {
        match *self {
            GunType::MuzzleLoading => 0.9,
            GunType::BreechLoading => 1.0,
            GunType::QuickFiring   => 1.35,
            GunType::AntiAir       => 1.44,
            GunType::DualPurpose   => 1.57,
            GunType::RapidFire     => 2.16,
            GunType::MachineGun    => 1.0,
        }
    }

    // wgt_lg {{{2
    pub fn wgt_lg(&self) -> f64 {
        match *self {
            GunType::MuzzleLoading => 0.98,
            GunType::BreechLoading => 1.0,
            GunType::QuickFiring   => 1.0,
            GunType::AntiAir       => 1.0,
            GunType::DualPurpose   => 1.1,
            GunType::RapidFire     => 1.5,
            GunType::MachineGun    => 1.0,
        }
    }

}

// MountType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
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

impl MountType {
    // wgt{{{2
    pub fn wgt(&self) -> f64 {
        match *self {
            MountType::Broadside      =>0.83,
            MountType::ColesTurret    =>3.5,
            MountType::OpenBarbette   =>3.33,
            MountType::ClosedBarbette =>3.5,
            MountType::DeckAndHoist   =>3.15,
            MountType::Deck           =>1.08,
            MountType::Casemate       =>1.08,
        }
    }
    // wgt_adj {{{2
    pub fn wgt_adj(&self) -> f64 {
        match *self {
            MountType::Broadside      =>0.5,
            MountType::ColesTurret    =>1.0,
            MountType::OpenBarbette   =>0.7,
            MountType::ClosedBarbette =>1.0,
            MountType::DeckAndHoist   =>1.0,
            MountType::Deck           =>0.5,
            MountType::Casemate       =>0.5,
        }
    }
}

// GunDistributionType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
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

impl GunDistributionType {
    // super_factor_long {{{2
    pub fn super_factor_long(&self) -> bool {
        match *self {
            GunDistributionType::CenterlineEven   => false,
            GunDistributionType::CenterlineEndsFD => false,
            GunDistributionType::CenterlineEndsAD => true,
            GunDistributionType::CenterlineFDFwd  => true,
            GunDistributionType::CenterlineFD     => true,
            GunDistributionType::CenterlineFDAft  => true,
            GunDistributionType::CenterlineADFwd  => true,
            GunDistributionType::CenterlineAD     => true,
            GunDistributionType::CenterlineADAft  => true,
            GunDistributionType::SidesEven        => false,
            GunDistributionType::SidesEndsFD      => false,
            GunDistributionType::SidesEndsAD      => false,
            GunDistributionType::SidesFDFwd       => true,
            GunDistributionType::SidesFD          => true,
            GunDistributionType::SidesFDAft       => true,
            GunDistributionType::SidesADFwd       => true,
            GunDistributionType::SidesAD          => true,
            GunDistributionType::SidesADAft       => true,
        }
    }

    // position {{{2
    pub fn position(&self) -> f64 {
        match *self {
            // TODO: 1.0 => for group 2
            GunDistributionType::CenterlineEven   => 1.0,
            GunDistributionType::CenterlineEndsFD => 1.0,
            GunDistributionType::CenterlineEndsAD => 1.0,
            GunDistributionType::CenterlineFDFwd  => 0.0625,
            GunDistributionType::CenterlineFD     => 0.125,
            GunDistributionType::CenterlineFDAft  => 0.1875,
            GunDistributionType::CenterlineADFwd  => 0.0625,
            GunDistributionType::CenterlineAD     => 0.125,
            GunDistributionType::CenterlineADAft  => 0.1875,
            GunDistributionType::SidesEven        => 1.0,
            GunDistributionType::SidesEndsFD      => 1.0,
            GunDistributionType::SidesEndsAD      => 1.0,
            GunDistributionType::SidesFDFwd       => 0.0625,
            GunDistributionType::SidesFD          => 0.125,
            GunDistributionType::SidesFDAft       => 0.1875,
            GunDistributionType::SidesADFwd       => 0.0625,
            GunDistributionType::SidesAD          => 0.125,
            GunDistributionType::SidesADAft       => 0.1875,
        }
    }
}

// GunLayoutType {{{1
#[derive(Serialize, Deserialize, Default, Debug)]
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

impl GunLayoutType {
    // num_guns {{{2
    pub fn guns_per(&self) -> u32 {
        match *self {
            GunLayoutType::Single   => 1,
            GunLayoutType::Twin2Row => 2,
            GunLayoutType::Twin     => 2,
            GunLayoutType::TwoGun   => 2,
            GunLayoutType::Triple   => 3,
            GunLayoutType::ThreeGun => 3,
            GunLayoutType::Quad2Row => 4,
            GunLayoutType::Quad4Row => 4,
            GunLayoutType::Quad     => 4,
            GunLayoutType::FourGun  => 4,
            GunLayoutType::Quint    => 5,
            GunLayoutType::FiveGun  => 5,
            GunLayoutType::Sex2Row  => 6,
            GunLayoutType::Oct2Row  => 8,
            GunLayoutType::Dec2Row  => 10,
        }
    }

    // num1 {{{2
    pub fn num1(&self) -> f64 {
        match *self {
            GunLayoutType::Single   => 1.44,
            GunLayoutType::Twin2Row => 1.44,
            GunLayoutType::Quad4Row => 1.44,
            GunLayoutType::Twin     => 1.52,
            GunLayoutType::TwoGun   => 1.52,
            GunLayoutType::Quad2Row => 1.52,
            GunLayoutType::Triple   => 1.64,
            GunLayoutType::ThreeGun => 1.64,
            GunLayoutType::Sex2Row  => 1.64,
            GunLayoutType::Quad     => 1.8,
            GunLayoutType::FourGun  => 1.8,
            GunLayoutType::Oct2Row  => 1.8,
            GunLayoutType::Quint    => 2.0,
            GunLayoutType::FiveGun  => 2.0,
            GunLayoutType::Dec2Row  => 2.0,
        }
    }

    // num2 {{{2
    pub fn num2(&self) -> f64 {
        match *self {
            GunLayoutType::Single   => 0.609725,
            GunLayoutType::Twin2Row => 0.609725,
            GunLayoutType::Quad4Row => 0.609725,
            GunLayoutType::Twin     => 0.4205,
            GunLayoutType::TwoGun   => 0.4205,
            GunLayoutType::Quad2Row => 0.4205,
            GunLayoutType::Triple   => 0.29,
            GunLayoutType::ThreeGun => 0.29,
            GunLayoutType::Sex2Row  => 0.29,
            GunLayoutType::Quad     => 0.2,
            GunLayoutType::FourGun  => 0.2,
            GunLayoutType::Oct2Row  => 0.2,
            GunLayoutType::Quint    => 0.14,
            GunLayoutType::FiveGun  => 0.14,
            GunLayoutType::Dec2Row  => 0.14,
        }
    }

    // wgt_adj {{{2
    pub fn wgt_adj(&self) -> f64 {
        match *self {
            GunLayoutType::Single   => 1.0,
            GunLayoutType::Twin2Row => 1.0,
            GunLayoutType::Quad4Row => 1.0,
            GunLayoutType::Twin     => 0.75,
            GunLayoutType::TwoGun   => 1.0,
            GunLayoutType::Quad2Row => 1.0,
            GunLayoutType::Triple   => 0.75,
            GunLayoutType::ThreeGun => 1.0,
            GunLayoutType::Sex2Row  => 1.0,
            GunLayoutType::Quad     => 0.75,
            GunLayoutType::FourGun  => 1.0,
            GunLayoutType::Oct2Row  => 1.0,
            GunLayoutType::Quint    => 0.75,
            GunLayoutType::FiveGun  => 1.0,
            GunLayoutType::Dec2Row  => 1.0,
        }
    }
}


