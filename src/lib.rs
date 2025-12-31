pub const SHIP_FILE_EXT: &str = "ship";
pub const SS_SHIP_FILE_EXT: &str = "sship";

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

use crate::unit_types::Units::*;
use crate::unit_types::metric;
use crate::unit_types::UnitType::*;

use bitflags::{bitflags, bitflags_match};
use serde::{Serialize, Deserialize};

use std::error::Error;
use std::fmt;
use std::fs;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

#[cfg(test)] // Testing support {{{1
mod test_support {
    pub fn to_place(n: f64, digits: u32) -> f64 {
        let mult = 10_u32.pow(digits) as f64;
        (n * mult).round() / mult
    }
}


// Ship {{{1
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ship {
    /// Name of ship.
    pub name: String,
    /// Country of ship.
    pub country: String,
    /// Type of ship. (This is informative only and does not affect any
    /// calculations.)
    pub kind: String,
    /// Year ship laid down: The general technology level is determined by the
    /// date the ship is laid down. This affects weaponry, armor, engines,
    /// speed, fuel consumption, strength, cost, roominess.
    pub year: u32,

    pub trim: u8,

    /// Hull configuration.
    pub hull: Hull,
    /// Armor configuration.
    pub armor: Armor,
    /// Engine configuration.
    pub engine: Engine,
    /// Gun batteries.
    pub batteries: Vec<Battery>,
    /// Torpedo mounts.
    pub torps: Vec<Torpedoes>,
    /// Mines.
    pub mines: Mines,
    /// ASW gear.
    pub asw: Vec<ASW>,
    /// Miscellaneous weights.
    pub wgts: MiscWgts,

    /// Custom notes
    pub notes: Vec<String>,
}

// Ship API {{{1
impl Ship {
    pub fn name(&self) -> String { self.name.clone() }
    pub fn country(&self) -> String { self.country.clone() }
    pub fn kind(&self) -> String { self.kind.clone() }
    pub fn year(&self) -> String { self.year.to_string() }

    pub fn new(
        name: String,
        country: String,
        kind: String,
        year: String,
    ) -> Ship {
        Ship {
            name: name.clone(),
            country: country.clone(),
            kind: kind.clone(),
            year: match year.parse() { Ok(n) => n, Err(_) => 0, },

            trim: 50,

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

            notes: Vec::new(),
        }
    }
}

impl Default for Ship { // {{{1
    fn default() -> Ship {
        Ship {
            name: "".into(),
            country: "".into(),
            kind: "".into(),
            year: 0,

            trim: 50,

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

            notes: Vec::new(),
        }
    }
}

// Ship Implementation {{{1
impl Ship {
    /// Pounds in a long ton.
    const POUND2TON: f64 = 2240.0;

    // year_adj {{{2
    /// Year adjustment factor for various calculations.
    ///
    pub fn year_adj(year: u32) -> f64 {
             if year <= 1890 { 1.0 - (1890 - year) as f64 / 66.666664 }
        else if year <= 1950 { 1.0 }
        else                 { 0.0 }
    }

    // deck_space {{{2
    /// Relative measure of hull space based on waterplane area, freeboard and
    /// displacement adjusted for above water torpedoes.
    ///
    pub fn deck_space(&self) -> f64 {
        let mut space = 0.0;
        for w in self.torps.iter() {
            space += w.deck_space(self.hull.b); 
        }

        space / self.hull.wp()
    }

    // hull_space {{{2
    /// Proportional measure of weights of engines, guns, magazines,
    /// miscellaneous weights, ships stores, torpedo bulkheads and hull mounted
    /// torpedoes to displacement to estimate the minimum length of the
    /// "vitalspace" needed to contain these relative to a norm of 65% of water
    /// length.
    ///
    pub fn hull_space(&self) -> f64 {
        let mut space = 0.0;
        for w in self.torps.iter() {
            space += w.hull_space(); 
        }
        space / (self.hull.d() * Hull::FT3_PER_TON_SEA)
    }

    // wgt_bunker {{{2
    /// Convenience function to get bunkerage weight from the engine.
    ///
    fn wgt_bunker(&self) -> f64 {
        self.engine.bunker(
            self.hull.d(),
            self.hull.lwl(),
            self.hull.leff(),
            self.hull.cs(),
            self.hull.ws()
        )
    }

    // wgt_load {{{2
    /// Weight of bunkerage, magazine and stores.
    ///
    fn wgt_load(&self) -> f64 {
        self.hull.d() * 0.02 + self.wgt_bunker() + self.wgt_mag()
    }

    // d_lite {{{2
    /// Light Displacement (t): Displacement without bunkerage, magazine or
    /// stores.
    ///
    pub fn d_lite(&self) -> f64 {
        self.hull.d() - self.wgt_load()
    }

    // d_std {{{2
    /// Standard Displacement (t): Standardized displacement per the Washington
    /// and London Naval Treaties. Does not include bunkerage or reserve
    /// feedwater.
    ///
    pub fn d_std(&self) -> f64 {
        self.hull.d() - self.wgt_bunker()
    }

    // d_max {{{2
    /// Maximum Displacement (t): Displacement including full bunker, magazines,
    /// feedwater and stores.
    ///
    pub fn d_max(&self) -> f64 {
        self.hull.d() + 0.8 * self.wgt_bunker()
    }

    // t_max {{{2
    /// Draft at maximum displacement.
    ///
    pub fn t_max(&self) -> f64 {
        self.hull.t_calc(self.d_max())
    }

    // cb_max {{{2
    /// Block coeficcient at maximum displacement.
    ///
    pub fn cb_max(&self) -> f64 {
        self.hull.cb_calc(self.d_max(), self.t_max())
    }

    // crew_max {{{2
    /// Estimated maximum crew size based on displacement.
    ///
    pub fn crew_max(&self) -> u32 {
        (self.hull.d().powf(0.75) * 0.65) as u32
    }

    // crew_min {{{2
    /// Estimated minimum crew size based on displacement.
    ///
    pub fn crew_min(&self) -> u32 {
        (self.crew_max() as f64 * 0.7692) as u32
    }

    // convert {{{2
    /// Load a ship from a SpringSharp 3 file and output a sharpie ship
    ///
    pub fn convert(p: String) -> Result<Ship, Box<dyn Error>> {
        let mut ship = Ship::default();

        let f = File::open(p)?;
        let reader = BufReader::new(f);
        let mut lines = reader.lines().map(|l| l.unwrap());

        let line = lines.next().unwrap();
        if line.contains("SpringSharp Version 3.0") {
            ()
        } else if line.contains("SpringSharp") {
            Err("SpringSharp file too old")?;
        } else {
            Err("Unknown file format")?;
        }

        ship.name    = lines.next().unwrap();
        ship.country = lines.next().unwrap();
        ship.kind    = lines.next().unwrap();

        ship.hull.units     = lines.next().unwrap().into();
        for b in ship.batteries.iter_mut() { b.units = lines.next().unwrap().into(); }
        ship.torps[0].units = lines.next().unwrap().into();
        ship.armor.units    = lines.next().unwrap().into();

        ship.year = lines.next().unwrap().parse()?;

        ship.wgts.vital = lines.next().unwrap().parse()?;

        ship.hull.set_lwl(lines.next().unwrap().parse()?);
        ship.hull.b          = lines.next().unwrap().parse()?;
        ship.hull.t          = lines.next().unwrap().parse()?;
        ship.hull.stern_type = lines.next().unwrap().into();
        ship.hull.set_cb(lines.next().unwrap().parse()?);

        ship.hull.qd_aft         = lines.next().unwrap().parse()?;
        ship.hull.stern_overhang = lines.next().unwrap().parse()?;
        ship.hull.qd_len         = lines.next().unwrap().parse()?;
        ship.hull.qd_len /= 100.0; // convert from % to decimal
        ship.hull.qd_fwd         = lines.next().unwrap().parse()?;
        ship.hull.ad_aft         = lines.next().unwrap().parse()?;
        ship.hull.fd_len         = lines.next().unwrap().parse()?;
        ship.hull.fd_len /= 100.0; // convert from % to decimal
        ship.hull.ad_fwd         = lines.next().unwrap().parse()?;
        ship.hull.fd_aft         = lines.next().unwrap().parse()?;
        ship.hull.fc_len         = lines.next().unwrap().parse()?;
        ship.hull.fc_len /= 100.0; // convert from % to decimal
        ship.hull.fd_fwd         = lines.next().unwrap().parse()?;
        ship.hull.fc_aft         = lines.next().unwrap().parse()?;
        ship.hull.fc_fwd         = lines.next().unwrap().parse()?;
        ship.hull.bow_angle      = lines.next().unwrap().parse()?;

        for b in ship.batteries.iter_mut() {
            b.num             = lines.next().unwrap().parse()?;
            b.cal             = lines.next().unwrap().parse()?;
            b.kind            = lines.next().unwrap().into();
            b.groups[0].above = lines.next().unwrap().parse()?;
            b.groups[0].below = lines.next().unwrap().parse()?;

            // Have to remove the commas from the string or it fails
            // to convert to a float
            b.set_shell_wgt( lines.next().unwrap().replace(",", "").parse()? );
        }

        ship.batteries[0].shells                 = lines.next().unwrap().parse()?;
        ship.batteries[0].mount_num              = lines.next().unwrap().parse()?;
        ship.batteries[0].mount_kind             = lines.next().unwrap().into();
        ship.batteries[0].groups[0].distribution = lines.next().unwrap().into();

        ship.batteries[1].mount_num              = lines.next().unwrap().parse()?;
        ship.batteries[1].mount_kind             = lines.next().unwrap().into();
        ship.batteries[1].groups[0].distribution = lines.next().unwrap().into();

        ship.batteries[2].mount_num              = lines.next().unwrap().parse()?;
        ship.batteries[2].mount_kind             = lines.next().unwrap().into();
        ship.batteries[2].groups[0].distribution = lines.next().unwrap().into();

        ship.batteries[3].mount_num              = lines.next().unwrap().parse()?;
        ship.batteries[3].mount_kind             = lines.next().unwrap().into();
        ship.batteries[3].groups[0].distribution = lines.next().unwrap().into();

        ship.batteries[4].mount_num              = lines.next().unwrap().parse()?;
        ship.batteries[4].mount_kind             = lines.next().unwrap().into();
        ship.batteries[4].groups[0].distribution = lines.next().unwrap().into();

        ship.torps[0].num  = lines.next().unwrap().parse()?;
        ship.torps[1].num  = lines.next().unwrap().parse()?;
        ship.torps[0].diam = lines.next().unwrap().parse()?;

        ship.armor.main.thick = lines.next().unwrap().parse()?;
        ship.armor.main.len   = lines.next().unwrap().parse()?;
        ship.armor.main.hgt   = lines.next().unwrap().parse()?;

        ship.armor.end.thick = lines.next().unwrap().parse()?;
        ship.armor.end.len   = lines.next().unwrap().parse()?;
        ship.armor.end.hgt   = lines.next().unwrap().parse()?;

        ship.armor.upper.thick = lines.next().unwrap().parse()?;
        ship.armor.upper.len   = lines.next().unwrap().parse()?;
        ship.armor.upper.hgt   = lines.next().unwrap().parse()?;

        ship.armor.bulkhead.thick = lines.next().unwrap().parse()?;
        ship.armor.bulkhead.len   = lines.next().unwrap().parse()?;
        ship.armor.bulkhead.hgt   = lines.next().unwrap().parse()?;

        for b in ship.batteries.iter_mut() {
            b.armor_face = lines.next().unwrap().parse()?;
            b.armor_back = lines.next().unwrap().parse()?;
            b.armor_barb = lines.next().unwrap().parse()?;
        }

        ship.armor.deck.md      = lines.next().unwrap().parse()?;
        ship.armor.ct_fwd.thick = lines.next().unwrap().parse()?;
        ship.engine.vmax        = lines.next().unwrap().parse()?;
        ship.engine.vcruise     = lines.next().unwrap().parse()?;
        ship.engine.range       = lines.next().unwrap().parse()?;
        ship.engine.shafts      = lines.next().unwrap().parse()?;
        ship.engine.pct_coal    = lines.next().unwrap().parse()?;
        ship.engine.pct_coal /= 100.0; // convert from % to decimal

        ship.engine.fuel = FuelType::empty();
        match lines.next().unwrap().as_str() { "True" => ship.engine.fuel.toggle(FuelType::Coal), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.fuel.toggle(FuelType::Oil), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.fuel.toggle(FuelType::Diesel), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.fuel.toggle(FuelType::Gasoline), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.fuel.toggle(FuelType::Battery), _ => (), };

        ship.engine.boiler = BoilerType::empty();
        match lines.next().unwrap().as_str() { "True" => ship.engine.boiler.toggle(BoilerType::Simple), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.boiler.toggle(BoilerType::Complex), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.boiler.toggle(BoilerType::Turbine), _ => (), };

        ship.engine.drive = DriveType::empty();
        match lines.next().unwrap().as_str() { "True" => ship.engine.drive.toggle(DriveType::Direct), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.drive.toggle(DriveType::Geared), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.drive.toggle(DriveType::Electric), _ => (), };
        match lines.next().unwrap().as_str() { "True" => ship.engine.drive.toggle(DriveType::Hydraulic), _ => (), };

        ship.trim        = lines.next().unwrap().parse()?;
        ship.hull.bb     = lines.next().unwrap().parse()?;
        ship.engine.year = lines.next().unwrap().parse()?;

        for b in ship.batteries.iter_mut() { b.year = lines.next().unwrap().parse()?; }

        ship.hull.bow_type = lines.next().unwrap().into();
        let ram_len        = lines.next().unwrap().parse()?;
        ship.hull.bow_type = match ship.hull.bow_type {
            BowType::Ram(_) => BowType::Ram(ram_len),
            _ => ship.hull.bow_type,
        };
            
        ship.torps[1].units = lines.next().unwrap().into();
        ship.mines.units    = lines.next().unwrap().into();
        ship.asw[0].units   = lines.next().unwrap().into();
        ship.asw[1].units   = lines.next().unwrap().into();

        for b in ship.batteries.iter_mut() { b.len = lines.next().unwrap().parse()?; }

        ship.batteries[1].shells = lines.next().unwrap().parse()?;
        ship.batteries[2].shells = lines.next().unwrap().parse()?;
        ship.batteries[3].shells = lines.next().unwrap().parse()?;
        ship.batteries[4].shells = lines.next().unwrap().parse()?;

        for b in ship.batteries.iter_mut() { b.groups[1].distribution  = lines.next().unwrap().into(); }
        for b in ship.batteries.iter_mut() { b.groups[1].above         = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[1].two_mounts_up = match lines.next().unwrap().as_str() { "True" => true, _ => false, }; }
        for b in ship.batteries.iter_mut() { b.groups[1].on            = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[1].below         = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[1].lower_deck    = match lines.next().unwrap().as_str() { "True" => true, _ => false, }; }

        ship.torps[0].mounts     = lines.next().unwrap().parse()?;
        ship.torps[1].mounts     = lines.next().unwrap().parse()?;
        ship.torps[1].diam       = lines.next().unwrap().parse()?;
        ship.torps[0].len        = lines.next().unwrap().parse()?;
        ship.torps[1].len        = lines.next().unwrap().parse()?;
        ship.torps[0].mount_kind = lines.next().unwrap().into();
        ship.torps[1].mount_kind = lines.next().unwrap().into();

        ship.mines.num        = lines.next().unwrap().parse()?;
        ship.mines.reload     = lines.next().unwrap().parse()?;
        ship.mines.wgt        = lines.next().unwrap().parse()?;
        ship.mines.mount_kind = lines.next().unwrap().into();

        ship.asw[0].num    = lines.next().unwrap().parse()?;
        ship.asw[1].num    = lines.next().unwrap().parse()?;
        ship.asw[0].reload = lines.next().unwrap().parse()?;
        ship.asw[1].reload = lines.next().unwrap().parse()?;
        ship.asw[0].wgt    = lines.next().unwrap().parse()?;
        ship.asw[1].wgt    = lines.next().unwrap().parse()?;
        ship.asw[0].kind   = lines.next().unwrap().into();
        ship.asw[1].kind   = lines.next().unwrap().into();

        ship.wgts.hull  = lines.next().unwrap().parse()?;
        ship.wgts.on    = lines.next().unwrap().parse()?;
        ship.wgts.above = lines.next().unwrap().parse()?;

        ship.armor.incline               = lines.next().unwrap().parse()?;
        ship.armor.bulge.thick           = lines.next().unwrap().parse()?;
        ship.armor.bulge.len             = lines.next().unwrap().parse()?;
        ship.armor.bulge.hgt             = lines.next().unwrap().parse()?;
        ship.armor.strengthened_bulkhead = match lines.next().unwrap().parse()? { 0 => false, 1 | _ => true, };
        ship.armor.beam_between          = lines.next().unwrap().parse()?;
        ship.armor.deck.fc               = lines.next().unwrap().parse()?;
        ship.armor.deck.qd               = lines.next().unwrap().parse()?;
        ship.armor.deck.kind             = lines.next().unwrap().into();
        ship.armor.ct_aft.thick          = lines.next().unwrap().parse()?;

        for b in ship.batteries.iter_mut() { b.groups[0].above  = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[0].below  = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[1].above  = lines.next().unwrap().parse()?; }
        // Ignore extra reads of ship.batteries.groups[1].on, because, duplicate data in the file makes sense
        for _ in ship.batteries.iter_mut() { lines.next(); }
        for b in ship.batteries.iter_mut() { b.groups[1].below  = lines.next().unwrap().parse()?; }
        for b in ship.batteries.iter_mut() { b.groups[0].layout = lines.next().unwrap().into(); }
        for b in ship.batteries.iter_mut() { b.groups[1].layout = lines.next().unwrap().into(); }

        ship.wgts.void = lines.next().unwrap().parse()?;

        // Superfluous ship.batteries[4].layout
        for _ in 1..34 { lines.next(); }

        for line in lines.by_ref() { ship.notes.push(line); }

        // SpringSharp does not store the number of mounts in Group 0 that
        // are on the deck so we have to calculate it from the other numbers
        for b in ship.batteries.iter_mut() {
            b.groups[0].on = b.mount_num -
                b.groups[0].above - b.groups[0].below -
                b.groups[1].above - b.groups[1].on - b.groups[1].below;
        }

        // SpringSharp uses hull year for torpedo, mine and ASW year
        for t in ship.torps.iter_mut() { t.year = ship.year; }
        ship.mines.year = ship.year;
        for a in ship.asw.iter_mut() { a.year = ship.year; }

        Ok(ship)
    }

    // load {{{2
    /// Load ship from a file.
    ///
    pub fn load(p: String) -> Result<Ship, Box<dyn Error>> {
        let s = fs::read_to_string(p)?;
        let ship = serde_json::from_str(&s)?;

        Ok(ship)
    }

    // save {{{2
    /// Save ship to a file.
    ///
    pub fn save(&self, p: String) -> Result<(), Box<dyn Error>> {

        let s = serde_json::to_string(&self)?;
        fs::write(p, s)?;

        Ok(())
    }

    // ship_type {{{2
    fn ship_type(&self) -> String {
        let mut s: Vec<String> = Vec::new();

        let main = self.batteries[0].clone();
        let sec = self.batteries[1].clone();
        let ter = self.batteries[2].clone();

        if main.mount_kind == MountType::OpenBarbette ||
            sec.mount_kind == MountType::OpenBarbette
        { s.push("Barbette Ship".into()); }

        if main.groups[0].distribution == GunDistributionType::CenterlineFD ||
            main.groups[0].distribution == GunDistributionType::SidesEndsFD
        { s.push("Central Citadel Ship".into()); }

        let main_broad = main.mount_kind == MountType::Broadside;
        let sec_broad  = sec.mount_kind == MountType::Broadside;
        let ter_broad  = ter.mount_kind == MountType::Broadside;

        let main_below = (main.groups[0].below + main.groups[1].below) > 0;
        let sec_below  = (sec.groups[0].below + main.groups[1].below) > 0;
        let ter_below  = (ter.groups[0].below + main.groups[1].below) > 0;

        let main_broad_below = main_broad && main_below;
        let sec_broad_below  = sec_broad  && sec_below;
        let ter_broad_below  = ter_broad  && ter_below;

        let main_no_back = main.armor_face > 0.0;
        let sec_no_back  = sec.armor_face > 0.0;
        let ter_no_back  = ter.armor_face > 0.0;

        let main_broad_no_back = main_broad && main_no_back;
        let sec_broad_no_back  = sec_broad && sec_no_back;
        let ter_broad_no_back  = ter_broad && ter_no_back;

        let has_belt = (
            self.armor.main.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
            self.armor.end.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
            self.armor.upper.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b)
        ) > 0.0;

        if main_broad || sec_broad || ter_broad {
            if has_belt {
                if main_broad_no_back || sec_broad_no_back || ter_broad_no_back {
                    s.push("Armoured Casemate Ship".into());
                } else if self.hull.fc_len + self.hull.fd_len < 0.5 {
                    if main_broad_below || sec_broad_below || ter_broad_below {
                        s.push("Armoured Frigate (Broadside Ironclad)".into());
                    } else {
                        s.push("Armoured Corvette (Broadside Ironclad)".into());
                    }
                } else if main_broad_below || sec_broad_below || ter_broad_below {
                    s.push("Armoured Frigate (Central Battery Ironclad)".into());
                } else {
                    s.push("Armoured Corvette (Central Battery Ironclad)".into());
                }
            } else if main_broad_below || sec_broad_below || ter_broad_below {
                s.push("Frigate (Unarmoured)".into());
            } else {
                s.push("Corvette (Unarmoured)".into());
            }
        }

        s.join("\n")
    }

    // report {{{2
    /// Print report.
    ///
    pub fn report(&self) -> String {
        let mut report: Vec<String> = Vec::new();

        // Header {{{3
        report.push(format!("{}, {} {} laid down {}{}",
            self.name,
            self.country,
            self.kind,
            self.year,
            if self.year != self.engine.year {
                format!(" (Engine {})", self.engine.year)
            } else { "".into() }
        ));
        if self.ship_type() != "" {
            report.push(format!("{}", self.ship_type()));
        }

        // Warnings {{{3
        if self.hull.cb() <= 0.0 || self.hull.cb() > 1.0
            { report.push("DESIGN FAILURE: Displacement impossible with given dimensions".to_string()); }
        if self.hull.d() < (self.wgt_broad() / 4.0)
            { report.push("DESIGN FAILURE: Gun weight too much for hull".to_string()); }
        if self.wgt_armor() > self.hull.d()
            { report.push("DESIGN FAILURE: Armour weight too much for hull".to_string()); }
        if self.str_comp() < 0.5
            { report.push("DESIGN FAILURE: Overall load weight too much for hull".to_string()); }
        if self.metacenter() < 0.0
            { report.push("DESIGN FAILURE: Ship will capsize".to_string()); }

        report.push("".to_string());

    use format_num::format_num;
        report.push("Displacement:".to_string()); // {{{3
        report.push(format!("    {} t light; {} t standard; {} t normal; {} t full load",
            format_num!(",.0", self.d_lite()),
            format_num!(",.0", self.d_std()),
            format_num!(",.0", self.hull.d()),
            format_num!(",.0", self.d_max())
        ));
        report.push("".to_string());

        report.push("Dimensions: Length (overall / waterline) x beam x draught (normal/deep)".to_string()); // {{{3
        report.push(format!("    ({:.2} ft / {:.2} ft) x {:.2} ft {}x ({:.2} / {:.2} ft)",
            self.hull.loa(),
            self.hull.lwl(),
            self.hull.b,
            if self.hull.bb > self.hull.b { format!("(Bulges {:.2} ft) ", self.hull.bb) } else { "".into() },
            self.hull.t,
            self.t_max()
        ));
        report.push(format!("    ({:.2} m / {:.2} m) x {:.2} m {}x ({:.2} / {:.2} m)",
            metric(self.hull.loa(), LengthLong, self.hull.units),
            metric(self.hull.lwl(), LengthLong, self.hull.units),
            metric(self.hull.b, LengthLong, self.hull.units),
            if self.hull.bb > self.hull.b { format!("(Bulges {:.2} m) ", metric(self.hull.bb, LengthLong, self.hull.units)) } else { "".into() },
            metric(self.hull.t, LengthLong, self.hull.units),
            metric(self.t_max(), LengthLong, self.hull.units)
        ));
        report.push("".to_string());

        report.push("Armament:".to_string()); // {{{3
        for (i, b) in self.batteries.iter().enumerate() {
            let main_gun = i == 0;

            if b.num == 0 { continue; }
            report.push(format!("    {} - {:.2}\" / {:.1} mm {:.1} cal gun{} - {:.2}lbs / {:.2}kg shells, {:.0} per gun",
                b.num,
                b.cal,
                metric(b.cal, LengthSmall, b.units),
                b.len,
                match b.num { 1 => "", _ => "s", },
                b.shell_wgt(),
                metric(b.shell_wgt(), Weight, b.units),
                b.shells
            ));
            report.push(format!("        {} gun{} in {} mount{}, {} Model",
                b.kind,
                match b.num { 1 => "", _ => "s", },
                b.mount_kind,
                match b.num { 1 => "", _ => "s", },
                b.year
            ));

            for (i, sb) in b.groups.iter().enumerate() {
                let sb_super = match i {
                    0 => sb.above < (b.mount_num - b.groups[1].above),
                    // XXX: SpringSharp BUG. Correct line is the below commented line:
                    // 1 => sb.above < (b.mount_num - b.groups[0].above),
                    _ => sb.above < (2 * sb.num_mounts() - sb.above),
                };

                if sb.num_mounts() == 0 { continue; }
                report.push(format!("        {} x {} mount{} on {}",
                    sb.num_mounts(),
                    sb.layout,
                    match sb.num_mounts() { 1 => "", _ => "s", },
                    sb.distribution.desc(sb.num_mounts(), self.hull.fc_len + self.hull.fd_len)
                ));
                if sb.above > 0 {
                    report.push(format!("        {} {}raised mount{}{}",
                        sb.above,
                        match sb.two_mounts_up { true => "double ", false => "", },
                        if sb.above > 1 { "s" } else if sb.distribution.super_aft() && main_gun { " aft" } else { "" },
                        if sb_super {
                            match sb.distribution {
                                GunDistributionType::CenterlineEven |
                                GunDistributionType::CenterlineFD |
                                GunDistributionType::CenterlineAD |
                                GunDistributionType::SidesEven |
                                GunDistributionType::SidesFD |
                                GunDistributionType::SidesAD => "",

                                _ => match b.mount_kind {
                                    MountType::Broadside => "",
                                    MountType::ColesTurret => "",

                                    _ => " - superfiring",
                                    },
                            }
                        } else {
                            ""
                        }
                    ));
                }

                if sb.below > 0 {
                    report.push(format!("        {} hull mount{} {}- Limited use in {}",
                        sb.below,
                        if sb.above > 1 { "s" } else if sb.distribution.super_aft() && main_gun { " aft" } else { "" },
                        if b.mount_kind == MountType::Broadside {
                            (match sb.lower_deck { true => "on gundeck", false => "on upperdeck", }).into()
                        } else {
                            format!("in {}casemate{}",
                                if sb.lower_deck { "lower " } else { "" },
                                match sb.below { 1 => "", _ => "s", }
                            )
                        },
                        if b.free(self.hull.clone()) < 12.0 ||
                            (b.free(self.hull.clone()) < 19.0 && sb.lower_deck)
                        {
                            "any sea"
                        } else if b.free(self.hull.clone()) < 16.0 ||
                            (b.free(self.hull.clone()) < 24.0 && sb.lower_deck)
                        {
                            "all but light seas"
                        } else {
                            "heavy seas"
                        }
                    ));
                }
            }
        }
        report.push(format!("    Weight of broadside {:.0} lbs / {:.0} kg",
            self.wgt_broad(),
            metric(self.wgt_broad(), Weight, Imperial)
        ));

        // Weapons {{{3
        for (i, torp) in self.torps.iter().enumerate() {
            if torp.num == 0 { continue; }

            report.push(format!("{} Torpedoes",
                match i { 0 => "Main", 1 => "2nd", _ => "Other", }
            ));
            report.push(format!("{} - {:.1}\" / {:.0} mm, {:.2} ft / {:.2} m torpedo{} {:.3} t total",
                torp.num,
                torp.diam,
                metric(torp.diam, LengthSmall, torp.units),
                torp.len,
                metric(torp.len, LengthLong, torp.units),
                match torp.num {
                    1 => " -".to_string(),
                    _ => format!("es - {:.3} t each,", torp.wgt_weaps() / torp.num as f64),
                },
                torp.wgt_weaps()
            ));
            report.push(format!("    {}",
                torp.mount_kind.desc(torp.num, torp.mounts)
            ));
        }

        if self.mines.num != 0 {
            report.push("Mines".to_string());
            report.push(format!("{} - {:.2} lbs / {:.2} kg mines{} - {:.3} t total",
                self.mines.num,
                self.mines.wgt,
                metric(self.mines.wgt, Weight, self.mines.units),
                if self.mines.reload > 0 {
                    format!(" + {} reloads", self.mines.reload)
                } else { "".into() },
                self.mines.wgt_weaps()
            ));
            report.push(format!("    {}",
                self.mines.mount_kind.desc()
            ));
        }

        for (i, asw) in self.asw.iter().enumerate() {
            if asw.num == 0 { continue; }

            report.push(format!("{} DC/AS Mortars",
                match i { 0 => "Main", 1 => "2nd", _ => "Other", }
            ));
            report.push(format!("{} - {:.2} lbs / {:.2} kg {}{} - {:.3} t total",
                asw.num,
                asw.wgt,
                metric(asw.wgt, Weight, asw.units),
                asw.kind.inline_desc(),
                if asw.reload > 0 {
                    format!(" + {} reloads", self.mines.reload)
                } else { "".into() },
                asw.wgt_weaps()
            ));
            if asw.kind.desc() != "" {
                report.push(format!("    {}",
                    asw.kind.desc()
                ));
            }
        }

        // Armor {{{3
        report.push("".to_string());
        report.push("Armour:".to_string());

        if self.armor.main.thick + self.armor.end.thick + self.armor.upper.thick + self.armor.bulkhead.thick > 0.0 {
            report.push(" - Belts:    Width (max)    Length (avg)    Height (avg)".to_string());
            if self.armor.main.thick > 0.0 {
                report.push(format!("    Main:    {:.2}\" / {:.0} mm    {:.2} ft / {:.2} m    {:.2} ft / {:.2} m",
                    self.armor.main.thick,
                    metric(self.armor.main.thick, LengthSmall, self.armor.units),
                    self.armor.main.len,
                    metric(self.armor.main.len, LengthLong, self.armor.units),
                    self.armor.main.hgt,
                    metric(self.armor.main.hgt, LengthLong, self.armor.units),
                ));
            }

            if self.armor.end.thick > 0.0 {
                report.push(format!("    Ends:    {:.2}\" / {:.0} mm    {:.2} ft / {:.2} m    {:.2} ft / {:.2} m",
                    self.armor.end.thick,
                    metric(self.armor.end.thick, LengthSmall, self.armor.units),
                    self.armor.end.len,
                    metric(self.armor.end.len, LengthLong, self.armor.units),
                    self.armor.end.hgt,
                    metric(self.armor.end.hgt, LengthLong, self.armor.units),
                ));
                if self.armor.main.len + self.armor.end.len < self.hull.lwl() {
                    report.push(format!("    {:.2} ft / {:.2} m Unarmoured ends",
                        self.hull.lwl() - self.armor.main.len - self.armor.end.len,
                        metric(self.hull.lwl() - self.armor.main.len - self.armor.end.len, LengthLong, self.armor.units)
                    ));
                }
            } else if self.armor.main.len < self.hull.lwl() {
                report.push("    Ends:    Unarmoured".to_string());
            }

            if self.armor.upper.thick > 0.0 {
                report.push(format!("    Upper:    {:.2}\" / {:.0} mm    {:.2} ft / {:.2} m    {:.2} ft / {:.2} m",
                    self.armor.upper.thick,
                    metric(self.armor.upper.thick, LengthSmall, self.armor.units),
                    self.armor.upper.len,
                    metric(self.armor.upper.len, LengthLong, self.armor.units),
                    self.armor.upper.hgt,
                    metric(self.armor.upper.hgt, LengthLong, self.armor.units),
                ));
            }

            if self.armor.main.thick > 0.0 {
                report.push(format!("    Main Belt covers {:.0} % of normal length",
                    self.armor.belt_coverage(self.hull.lwl())*100.0
                ));
                if self.armor.belt_coverage(self.hull.lwl()) < self.hull_room() {
                    report.push("    Main belt does not fully cover magazines and engineering spaces".to_string());
                }
            }

            if self.armor.incline != 0.0 {
                report.push(format!("    Main Belt inclined {:.2} degrees (positive = in)",
                    self.armor.incline
                ));
            }
            report.push("".to_string());

            if self.armor.bulkhead.thick > 0.0 {
                report.push(format!("- Torpedo Bulkhead - {} bulkheads:",
                    if self.armor.strengthened_bulkhead { "Strengthened structural" }
                    else { "Additional damage containing" }
                ));
                report.push(format!("        {:.2}\" / {:.0} mm    {:.2} ft / {:.2} m    {:.2} ft / {:.2} m",
                    self.armor.bulkhead.thick,
                    metric(self.armor.bulkhead.thick, LengthSmall, self.armor.units),
                    self.armor.bulkhead.len,
                    metric(self.armor.bulkhead.len, LengthLong, self.armor.units),
                    self.armor.bulkhead.hgt,
                    metric(self.armor.bulkhead.hgt, LengthLong, self.armor.units),
                ));
                report.push(format!("    Beam between torpedo bulkheads {:.2} ft / {:.2} m",
                    self.armor.beam_between,
                    metric(self.armor.beam_between, LengthLong, self.armor.units)
                ));
            }
            report.push("".to_string());

            if self.armor.bulge.thick > 0.0 || self.wgts.void > 0 {
                report.push(format!("- Hull {}:",
                    if self.hull.b == self.hull.bb { "void" }
                    else { "Bulges" }
                ));
                report.push(format!("        {:.2}\" / {:.0} mm    {:.2} ft / {:.2} m    {:.2} ft / {:.2} m",
                    self.armor.bulge.thick,
                    metric(self.armor.bulge.thick, LengthSmall, self.armor.units),
                    self.armor.bulge.len,
                    metric(self.armor.bulge.len, LengthLong, self.armor.units),
                    self.armor.bulge.hgt,
                    metric(self.armor.bulge.hgt, LengthLong, self.armor.units),
                ));
                }

            report.push("".to_string());
        }

        if self.wgt_gun_armor() > 0.0 {
            report.push("- Gun armour:    Face (max)    Other gunhouse (avg)    Barbette/hoist (max)".to_string());

            for (i, b) in self.batteries.iter().enumerate() {
                if b.armor_face == 0.0 &&
                b.armor_back == 0.0 &&
                b.armor_barb == 0.0 { continue; }
                report.push(format!("    {}:    {}        {}            {}",
                    match i { 0 => "Main", 1 => "2nd", 2 => "3rd", 3 => "4th", 4 => "5th", _ => "Other", },
                    if b.armor_face == 0.0 { "-".into() } else if b.armor_face >= 10.0 { format!("{:.1}\" / {:.0} mm", b.armor_face, metric(b.armor_face, LengthSmall, b.units)) } else { format!("{:.2}\" / {:.0} mm", b.armor_face, metric(b.armor_face, LengthSmall, b.units)) },
                    if b.armor_back == 0.0 { "-".into() } else if b.armor_back >= 10.0 { format!("{:.1}\" / {:.0} mm", b.armor_back, metric(b.armor_back, LengthSmall, b.units)) } else { format!("{:.2}\" / {:.0} mm", b.armor_back, metric(b.armor_back, LengthSmall, b.units)) },
                    if b.armor_barb == 0.0 { "-".into() } else if b.armor_barb >= 10.0 { format!("{:.1}\" / {:.0} mm", b.armor_barb, metric(b.armor_barb, LengthSmall, b.units)) } else { format!("{:.2}\" / {:.0} mm", b.armor_barb, metric(b.armor_barb, LengthSmall, b.units)) },
                
                ));
            }
            report.push("".to_string());
        }

        if self.armor.deck.fc + self.armor.deck.md + self.armor.deck.qd > 0.0 {
            report.push(format!("- {}:",
                self.armor.deck.kind
            ));
            // TODO: Change spelling to Fore
            report.push(format!("    For and Aft decks: {:.2}\" / {:.0} mm",
                self.armor.deck.md,
                metric(self.armor.deck.md, LengthSmall, self.armor.units)
            ));
            // TODO: Change spelling to Quarterdeck
            report.push(format!("    Forecastle: {:.2}\" / {:.0} mm    Quarter deck: {:.2}\" / {:.0} mm",
                self.armor.deck.fc,
                metric(self.armor.deck.fc, LengthSmall, self.armor.units),
                self.armor.deck.qd,
                metric(self.armor.deck.qd, LengthSmall, self.armor.units)
            ));
            report.push("".to_string());
        }

        if self.armor.ct_fwd.thick + self.armor.ct_aft.thick > 0.0 {
            // TODO: Remove stray space before comma
            report.push(format!("- Conning towers: Forward {:.2}\" / {:.0} mm, Aft {:.2}\" / {:.0} mm",
                self.armor.ct_fwd.thick,
                metric(self.armor.ct_fwd.thick, LengthSmall, self.armor.units),
                self.armor.ct_aft.thick,
                metric(self.armor.ct_aft.thick, LengthSmall, self.armor.units)
            ));
            report.push("".to_string());
        }

        report.push("Machinery:".to_string()); // {{{3
        if self.engine.vmax != 0.0 {
            report.push(format!("    {}, {},",
                self.engine.fuel,
                self.engine.boiler
            ));
            report.push(format!("    {}, {} shaft{}, {:.0} {} / {:.0} Kw = {:.2} kts",
                self.engine.drive,
                self.engine.shafts,
                match self.engine.shafts { 1 => "", _ => "s", },
                self.engine.hp_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()),
                self.engine.boiler.hp_type(),
                metric(self.engine.hp_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()), Power, Imperial),
                self.engine.vmax
            ));
            report.push(format!("    Range {}nm at {:.2} kts",
                self.engine.range,
                self.engine.vcruise
            ));
            report.push(format!("    Bunker at max displacement = {:.0} tons{}",
                self.engine.bunker_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()),
                if self.engine.pct_coal > 0.0 { format!(" ({:.0}% coal)", self.engine.pct_coal * 100.0) } else { "".into() }
            ));
        } else {
            report.push("    Immobile floating battery".to_string());
        }
        report.push("".to_string());

        report.push("Complement:".to_string()); // {{{3
        report.push(format!("    {} - {}",
            self.crew_min(),
            self.crew_max()
        ));
        report.push("".to_string());

        report.push("Cost:".to_string()); // {{{3
        report.push(format!("    £{:.3} million / ${:.3} million",
            self.cost_lb(),
            self.cost_dollar()
        ));
        report.push("".to_string());

        report.push("Distribution of weights at normal displacement:".to_string()); // {{{3
        report.push(format!("    Armament: {:.0} tons, {:.1} %",
            (self.wgt_guns() + self.wgt_gun_mounts() + self.wgt_weaps()),
            Ship::percent_calc(self.hull.d(), self.wgt_guns() + self.wgt_gun_mounts()) +
            Ship::percent_calc(self.hull.d(), self.wgt_weaps())
        ));

        if self.wgt_guns() > 0.0 {
            report.push(format!("    - Guns: {:.0} tons, {:.1} %",
                (self.wgt_guns() + self.wgt_gun_mounts()),
                Ship::percent_calc(self.hull.d(), self.wgt_guns() + self.wgt_gun_mounts())
            ));
        }

        if self.torps[0].wgt() + self.torps[1].wgt() + self.mines.wgt() + self.asw[0].wgt() + self.asw[1].wgt > 0.0 {
            report.push(format!("    - Weapons: {:.0} tons, {:.1} %",
                (self.torps[0].wgt() + self.torps[1].wgt() + self.mines.wgt() + self.asw[0].wgt() + self.asw[1].wgt()),
                Ship::percent_calc(self.hull.d(), self.torps[0].wgt() + self.torps[1].wgt() + self.mines.wgt() + self.asw[0].wgt() + self.asw[1].wgt())
            ));
        }

        if self.wgt_armor() > 0.0 {
            report.push(format!("    Armour: {:.0} tons, {:.1} %",
                self.wgt_armor(),
                Ship::percent_calc(self.hull.d(), self.wgt_armor())
            ));

            if self.armor.main.thick + self.armor.end.thick + self.armor.upper.thick > 0.0 {
                report.push(format!("    - Belts: {:.0} tons, {:.1} %",
                    (self.armor.main.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
                    self.armor.end.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
                    self.armor.upper.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b)),
                    Ship::percent_calc(self.hull.d(), 
                        self.armor.main.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
                        self.armor.end.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) +
                        self.armor.upper.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b))
                ));
            }

            if self.armor.bulkhead.thick > 0.0 {
                report.push(format!("    - Torpedo bulkhead: {:.0} tons, {:.1} %",
                    (self.armor.bulkhead.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b)),
                    Ship::percent_calc(self.hull.d(), self.armor.bulkhead.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b))
                ));
            }

            if self.armor.bulge.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) > 0.0 {
                report.push(format!("    - {}: {:.0} tons, {:.1} %",
                    if self.hull.b == self.hull.bb { "Void" } else { "Bulges" },
                    self.armor.bulge.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b),
                    Ship::percent_calc(self.hull.d(), self.armor.bulge.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b))
                ));
            }

            if self.wgt_gun_armor() > 0.0 {
                report.push(format!("    - Armament: {:.0} tons, {:.1} %",
                    self.wgt_gun_armor(),
                    Ship::percent_calc(self.hull.d(), self.wgt_gun_armor())
                ));
            }

            if self.armor.deck.fc + self.armor.deck.md + self.armor.deck.qd > 0.0 {
                report.push(format!("    - Armour Deck: {:.0} tons, {:.1} %",
                    (self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), 0.0)),
                    Ship::percent_calc(self.hull.d(), self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), 0.0))
                ));
                    // TODO: (self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), self.wgt_engine())),
                    // TODO: Ship::percent_calc(self.hull.d(), self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), self.wgt_engine())));
            }

            if self.armor.ct_fwd.thick + self.armor.ct_aft.thick > 0.0 {
                report.push(format!("    - Conning Tower{}: {:.0} tons, {:.1} %",
                    if self.armor.ct_fwd.thick > 0.0 && self.armor.ct_aft.thick > 0.0 {
                        "s"
                    } else { "" },
                    (self.armor.ct_fwd.wgt(self.hull.d()) + self.armor.ct_aft.wgt(self.hull.d())),
                    Ship::percent_calc(self.hull.d(), self.armor.ct_fwd.wgt(self.hull.d()) + self.armor.ct_aft.wgt(self.hull.d()))
                ));
            }
        }

        report.push(format!("    Machinery: {:.0} tons, {:.1} %",
            self.wgt_engine(),
            Ship::percent_calc(self.hull.d(), self.wgt_engine())
        ));
        report.push(format!("    Hull, fittings & equipment: {:.0} tons, {:.1} %",
            self.wgt_hull(),
            Ship::percent_calc(self.hull.d(), self.wgt_hull())
        ));
        report.push(format!("    Fuel, ammunition & stores: {:.0} tons, {:.1} %",
            self.wgt_load(),
            Ship::percent_calc(self.hull.d(), self.wgt_load())
        ));

        if self.wgts.wgt() > 0 {
            report.push(format!("    Miscellaneous weights: {:.0} tons, {:.1} %",
                self.wgts.wgt(),
                Ship::percent_calc(self.hull.d(), self.wgts.wgt().into())
            ));
            if self.wgts.vital > 0 { report.push(format!("    - Hull below water: {:.0} tons", self.wgts.vital
            )); }
            if self.wgts.void > 0 {
                report.push(format!("    - {} void weights: {:.0} tons",
                    if self.hull.bb > self.hull.b { "Bulge" } else { "Hull" },
                    self.wgts.void
                ));
            }
            if self.wgts.hull > 0 { report.push(format!("    - Hull above water: {:.0} tons", self.wgts.hull
            )); }
            if self.wgts.on > 0 { report.push(format!("    - On freeboard deck: {:.0} tons", self.wgts.on
            )); }
            if self.wgts.above > 0 { report.push(format!("    - Above deck: {:.0} tons", self.wgts.above
            )); }
        }

        report.push("".to_string());

        report.push("Overall survivability and seakeeping ability:".to_string()); // {{{3
        report.push("    Survivability (Non-critical penetrating hits needed to sink ship):".to_string());
        report.push(format!("    {:.0} lbs / {:.0} Kg = {:.1} x {:.1} \" / {:.0} mm shells or {:.1} torpedoes",
            self.flotation(),
            metric(self.flotation(), Weight, Imperial),
            self.damage_shell_num(),
            self.damage_shell_size(),
            metric(self.damage_shell_size(), LengthSmall, Imperial),
            self.damage_torp_num()
        ));
        report.push(format!("    Stability (Unstable if below 1.00): {:.2}",
            self.stability_adj()
        ));
        report.push(format!("    Metacentric height {:.1} ft / {:.1} m",
            self.metacenter(),
            metric(self.metacenter(), LengthLong, Imperial)
        ));
        report.push(format!("    Roll period: {:.1} seconds",
            self.roll_period()
        ));
        report.push(format!("    Steadiness    - As gun platform (Average = 50 %): {:.0} %",
            self.steadiness()
        ));
        report.push(format!("        - Recoil effect (Restricted arc if above 1.00): {:.2}",
            self.recoil()
        ));
        report.push(format!("    Seaboat quality (Average = 1.00): {:.2}",
            self.seakeeping()
        ));
        report.push("".to_string());

        report.push("Hull form characteristics:".to_string()); // {{{3
        report.push(format!("    Hull has {},",
            self.hull.freeboard_desc()
        ));
        report.push(format!("    {} and {}",
            self.hull.bow_type,
            self.hull.stern_type
        ));
        report.push(format!("    Block coefficient (normal/deep): {:.3} / {:.3}",
            self.hull.cb(), self.cb_max()
        ));
        report.push(format!("    Length to Beam Ratio: {:.2} : 1",
            self.hull.len2beam()
        ));
        report.push(format!("    'Natural speed' for length: {:.2} kts",
            self.hull.vn()
        ));
        report.push(format!("    Power going to wave formation at top speed: {:.0} %",
            self.engine.pw_max(self.hull.d(), self.hull.lwl(), self.hull.cs(), self.hull.ws()) * 100.0
        ));
        report.push(format!("    Trim (Max stability = 0, Max steadiness = 100): {}",
            self.trim
        ));
        report.push(format!("    Bow angle (Positive = bow angles forward): {:.2} degrees",
            self.hull.bow_angle
        ));
        report.push(format!("    Stern overhang: {:.2} ft / {:.2} m",
            self.hull.stern_overhang,
            metric(self.hull.stern_overhang, LengthLong, self.hull.units)
        ));
        report.push(format!("    Freeboard (% = length of deck as a percentage of waterline length):"
        ));
        report.push("            Fore end, Aft end".to_string());
        report.push("".to_string());
        report.push(format!("    - Forecastle:    {:.2} %, {:.2} ft / {:.2} m, {:.2} ft / {:.2} m",
            self.hull.fc_len*100.0,   self.hull.fc_fwd, metric(self.hull.fc_fwd, LengthLong, self.hull.units), self.hull.fc_aft, metric(self.hull.fc_aft, LengthLong, self.hull.units)
        ));
        report.push(format!("    - Forward deck:    {:.2} %, {:.2} ft / {:.2} m, {:.2} ft / {:.2} m",
            self.hull.fd_len*100.0,   self.hull.fd_fwd, metric(self.hull.fd_fwd, LengthLong, self.hull.units), self.hull.fd_aft, metric(self.hull.fd_aft, LengthLong, self.hull.units)
        ));
        report.push(format!("    - Aft deck:    {:.2} %, {:.2} ft / {:.2} m, {:.2} ft / {:.2} m",
            self.hull.ad_len()*100.0, self.hull.ad_fwd, metric(self.hull.ad_fwd, LengthLong, self.hull.units), self.hull.ad_aft, metric(self.hull.ad_aft, LengthLong, self.hull.units)
        ));
        report.push(format!("    - Quarter deck:    {:.2} %, {:.2} ft / {:.2} m, {:.2} ft / {:.2} m",
            self.hull.qd_len*100.0,   self.hull.qd_fwd, metric(self.hull.qd_fwd, LengthLong, self.hull.units), self.hull.qd_aft, metric(self.hull.qd_aft, LengthLong, self.hull.units)
        ));
        report.push(format!("    - Average freeboard:        {:.2} ft / {:.2} m",
            self.hull.freeboard(), metric(self.hull.freeboard(), LengthLong, self.hull.units)
        
        ));
        if self.hull.is_wet_fwd() {
            report.push("    Ship tends to be wet forward".to_string());
        }
        report.push("".to_string());

        report.push("Ship space, strength and comments:".to_string()); // {{{3
        report.push(format!("    Space    - Hull below water (magazines/engines, low = better): {:.1} %",
            self.hull_room() * 100.0
        ));
        report.push(format!("        - Above water (accommodation/working, high = better): {:.1} %",
            self.deck_room() * 100.0
        ));
        report.push(format!("    Waterplane Area: {:.0} Square feet or {:.0} Square metres",
            self.hull.wp(),
            metric(self.hull.wp(), Area, Imperial)
        ));
        report.push(format!("    Displacement factor (Displacement / loading): {:.0} %",
            self.d_factor() * 100.0
        ));
        report.push(format!("    Structure weight / hull surface area: {:.0} lbs/sq ft or {:.0} Kg/sq metre",
            self.wgt_struct(),
            metric(self.wgt_struct(), WeightPerArea, Imperial)

            
        ));
        report.push("Hull strength (Relative):".to_string());
        report.push(format!("        - Cross-sectional: {:.2}",
            self.str_cross()
        ));
        report.push(format!("        - Longitudinal: {:.2}",
            self.str_long()
        ));
        report.push(format!("        - Overall: {:.2}",
            self.str_comp()
        ));
        report.push(format!("    {} machinery, storage, compartmentation space",
            self.hull_room_quality()
        ));
        report.push(format!("    {} accommodation and workspace room",
            self.deck_room_quality()
        ));
        for s in self.seakeeping_desc() {
            report.push(format!("    {}", s
            ));
        }
        report.push("".to_string());

        // Custom Notes {{{3
        for s in self.notes.iter() {
            report.push(format!("{}", s));
        }

        report.join("\n")
    }

    // Print internal values {{{2
    pub fn internals(&self) -> String {
        let mut s: Vec<String> = Vec::new();

        s.push("Internal values".to_string());
        s.push("===============".to_string());
        s.push("".to_string());
        s.push("Gun Batteries".to_string());
        s.push("------------".to_string());
        s.push(format!("wgt_guns = {}", self.wgt_guns()));
        s.push(format!("wgt_gun_mounts = {}", self.wgt_gun_mounts()));
        s.push(format!("wgt_mag = {}", self.wgt_mag()));
        s.push(format!("wgt_gun_armor = {}", self.wgt_gun_armor()));
        s.push(format!("wgt_borne = {}", self.wgt_borne()));
        s.push(format!("super_factor = {}", self.gun_super_factor()));
        s.push(format!("gun_wtf = {}", self.gun_wtf()));
        s.push("".to_string());

        for (i, b) in self.batteries.iter().enumerate() {
            s.push(format!("battery[{}]", i));
            s.push("-----------".to_string());
            b.internals(self.hull.clone(), self.wgt_broad());
            s.push("".to_string());
        }

        s.push(format!("Cs = {}", self.hull.cs()));
        s.push(format!("Cm = {}", Hull::cm(self.hull.cb())));
        s.push(format!("Cp = {}", Hull::cp(self.hull.cb())));
        s.push(format!("Cwp = {}", self.hull.cwp()));
        s.push(format!("WP = {}", self.hull.wp()));
        s.push(format!("WS = {}", self.hull.ws()));
        s.push(format!("Ts = {}", self.hull.ts()));
        s.push("".to_string());
        s.push(format!("Stem length = {}", self.hull.stem_len()));
        if let BowType::Ram(len) = self.hull.bow_type { s.push(format!("Ram length = {}", len)); }
        s.push(format!("Freeboard dist = {}", self.hull.freeboard_dist()));
        s.push(format!("Leff = {}", self.hull.leff()));
        s.push("".to_string());
        s.push(format!("Rf max = {}", self.engine.rf_max(self.hull.ws())));
        s.push(format!("Rf cruise = {}", self.engine.rf_cruise(self.hull.ws())));
        s.push(format!("Rw max = {}", self.engine.rw_max(self.hull.d(), self.hull.lwl(), self.hull.cs())));
        s.push(format!("Rw cruise = {}", self.engine.rw_cruise(self.hull.d(), self.hull.lwl(), self.hull.cs())));
        s.push(format!("Pw max = {}", self.engine.pw_max(self.hull.d(), self.hull.lwl(), self.hull.cs(), self.hull.ws())));
        s.push(format!("Pw cruise = {}", self.engine.pw_cruise(self.hull.d(), self.hull.lwl(), self.hull.cs(), self.hull.ws())));
        s.push("".to_string());
        s.push(format!("hp max = {}", self.engine.hp_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws())));
        s.push(format!("hp cruise = {}", self.engine.hp_cruise(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws())));
        s.push("".to_string());

        s.push(format!("wgt_load = {}", self.wgt_load()));
        s.push(format!("wgt_hull = {}", self.wgt_hull()));
        s.push(format!("wgt_hull_plus = {}", self.wgt_hull_plus()));
        s.push(format!("wgt_misc = {}", self.wgts.wgt()));
        s.push(format!("wgt_armor = {}", self.wgt_armor()));
        s.push("".to_string());

        s.push(format!("main belt = {}", self.armor.main.wgt(self.hull.d(), self.hull.cwp(), self.hull.b)));
        s.push(format!("upper belt = {}", self.armor.upper.wgt(self.hull.d(), self.hull.cwp(), self.hull.b)));
        s.push(format!("end belt = {}", self.armor.end.wgt(self.hull.d(), self.hull.cwp(), self.hull.b)));
        s.push(format!("deck = {}", self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), 0.0)));
        s.push("".to_string());

        s.push(format!("wgt_engine = {}", self.wgt_engine()));
        s.push(format!("d_engine = {}", self.engine.d_engine(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws())));
        s.push(format!("d_factor = {}", self.d_factor()));
        s.push(format!("bunker (normal) = {}", self.engine.bunker(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws())));
        s.push(format!("bunker_factor = {}", self.engine.boiler.bunker_factor(self.engine.year)));
        s.push("".to_string());

        s.push(format!("stability = {}", self.stability()));
        s.push(format!("seaboat = {}", self.seaboat()));
        s.push("".to_string());

        s.push(format!("{:?}", self.engine.fuel));
        s.push(format!("{:?}", self.engine.boiler));
        s.push(format!("{:?}", self.engine.drive));
        s.push(format!("num_engines = {}", self.engine.num_engines()));

        s.push("".to_string());

        s.push(format!("gun_concentration = {}", self.gun_concentration()));
        s.push(format!("str_cross = {}", self.str_cross()));
        s.push(format!("str_long = {}", self.str_long()));
        s.push(format!("str_comp = {}", self.str_comp()));
        s.push(format!("flotation = {}", self.flotation()));

        s.join("\n")
    }
}

// Ship Performance {{{1
impl Ship {
    // room {{{2
    fn room(&self) -> f64 {
        (
            self.wgt_mag() +
            self.hull.d() * 0.02 +
            self.wgt_borne() * 6.4 +
            self.wgt_engine() * 3.0 +
            self.wgts.vital as f64 +
            self.wgts.hull as f64
        ) / (self.hull.d() * 0.94) / (1.0 - self.hull_space())
    }

    // hull_room {{{2
    pub fn hull_room(&self) -> f64 {
        self.room() * if self.armor.bulkhead.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) > 0.1 {
            self.hull.b / self.armor.beam_between
        } else { 1.0 }
    }

    // deck_room {{{2
    pub fn deck_room(&self) -> f64 {
        self.hull.wp() /
            Hull::FT3_PER_TON_SEA /
            15.0 * (1.0 - self.deck_space()) /
            self.crew_min() as f64 * self.hull.freeboard_dist()
    }

    // deck_room_quality {{{2
    /// Return a string describing the deck space
    ///
    pub fn deck_room_quality(&self) -> String {
        let sp = self.deck_room();

               if sp > 1.2 {
            "Excellent".into()
        } else if sp > 0.9 {
            "Adequate".into()
        } else if sp >= 0.5 {
            "Cramped".into()
        } else {
            "Poor".into()
        }
    }

    // hull_room_quality {{{2
    /// Return a string describing the hull space
    ///
    pub fn hull_room_quality(&self) -> String {
        let sp = self.hull_room();

               if sp < 5.0/6.0 {
            "Excellent".into()
        } else if sp < 1.1111112 {
            "Adequate".into()
        } else if sp <= 2.0 {
            "Cramped".into()
        } else {
            "Extremely poor".into()
        }
    }

    // cost_dollar {{{2
    /// Cost in $ million
    ///
    pub fn cost_dollar(&self) -> f64 {
        ((self.hull.d()-self.wgt_load())*0.00014+self.wgt_engine()*0.00056+(self.wgt_borne()*8.0)*0.00042)*
            if self.year as f64 +2.0>1914.0 {
                1.0+(self.year as f64 +1.5-1914.0)/5.5
            } else { 1.0 }
    }

    // cost_lb {{{2
    /// Cost in £ million
    ///
    pub fn cost_lb(&self) -> f64 {
        self.cost_dollar() / 4.0
    }

    // recoil {{{2
    pub fn recoil(&self) -> f64 {
        (
            (self.wgt_broad()/self.hull.d() * self.hull.freeboard_dist() * self.gun_super_factor() / self.hull.bb) *

            ( self.hull.d().powf(1.0 / 3.0) / self.hull.bb * 3.0 ).powf(2.0) * 7.0
        ) /
            if self.stability_adj() > 0.0 {
                self.stability_adj() * ((50.0 - self.steadiness()) / 150.0 + 1.0)
            } else { 1.0 }
    }

    // metacenter {{{2
    pub fn metacenter(&self) -> f64 {
        self.hull.b.powf(1.5) * (self.stability_adj() - 0.5) / 0.5 / 200.0
    }

    // seaboat {{{2
    fn seaboat(&self) -> f64 {
        let a = (self.hull.free_cap(self.cap_calc_broadside()) / (2.4 * self.hull.d().powf(0.2))).sqrt() *
            (
                (self.stability() * 5.0 * (self.hull.bb / self.hull.lwl())).powf(0.2) *
                (self.hull.free_cap(self.cap_calc_broadside()) / self.hull.lwl() * 20.0).sqrt() *
                (
                    self.hull.d() /
                        (
                            self.hull.d() +
                            self.armor.end.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) * 3.0 +
                            self.wgt_hull_plus() / 3.0 +
                            (
                                self.wgt_borne() +
                                self.wgt_gun_armor()
                            ) * self.super_factor_long()
                        )
                )
            ) * 8.0;

        let b = a * if (self.hull.t / self.hull.bb) < 0.3 {
                (self.hull.t / self.hull.bb / 0.3).sqrt()
            } else {
                1.0
            };

        let c = b *
            if (self.engine.rf_max(self.hull.ws()) / (self.engine.rf_max(self.hull.ws()) + self.engine.rw_max(self.hull.d(), self.hull.lwl(), self.hull.cs()))) < 0.55 &&
                self.engine.vmax > 0.0
            {
                (self.engine.rf_max(self.hull.ws()) / (self.engine.rf_max(self.hull.ws()) + self.engine.rw_max(self.hull.d(), self.hull.lwl(), self.hull.cs()))).powf(2.0)
            } else {
                0.3025
            };

        f64::min(c, 2.0)
    }

    // seakeeping {{{2
    pub fn seakeeping(&self) -> f64 {
        self.seaboat() * f64::min(self.steadiness(), 50.0) / 50.0
    }

    // tender_warn {{{2
    fn tender_warn(&self) -> bool {
        if self.metacenter() <= 0.995 {
            true
        } else {
            false
        }
    }

    // capsize_warn {{{2
    fn capsize_warn(&self) -> bool {
        if self.stability_adj() <= 0.0 {
            true
        } else {
            false
        }
    }

    // hull_strained {{{2
    fn hull_strained(&self) -> bool {
        if self.str_comp() >= 0.5 && self.str_comp() < 0.885 && (
            self.engine.vmax < 24.0 || self.hull.d() > 4000.0)
        {
            true
        } else {
            false
        }
    }

    // is_steady {{{2
    fn is_steady(&self) -> bool {
        if self.steadiness() >= 69.5 {
            true
        } else {
            false
        }
    }

    // is_unsteady {{{2
    fn is_unsteady(&self) -> bool {
        if self.steadiness() < 30.0 {
            true
        } else {
            false
        }
    }

    // type_sea {{{2
    fn type_sea(&self) -> SeaType {
               if self.seakeeping() < 0.7 {
            SeaType::BadSea
        } else if self.seakeeping() < 0.995 {
            SeaType::PoorSea
        } else if self.seakeeping() >= 1.5 {
            SeaType::FineSea
        } else if self.seakeeping() >= 1.2 {
            SeaType::GoodSea
        } else {
            SeaType::Error
        }
    }

    // seakeeping desc {{{2
    pub fn seakeeping_desc(&self) -> Vec<String> {
        let mut s: Vec<String> = Vec::new();
        
        if self.tender_warn() && self.capsize_warn() {
            s.push("Caution: Poor stability - excessive risk of capsizing".into());
        }

        if self.hull_strained() {
            s.push("Caution: Hull subject to strain in open-sea".into());
        }

        if self.is_steady() {
            s.push("Ship has slow easy roll, a good steady, gun platform".into());
        } else if self.is_unsteady() {
            s.push("Ship has quick, lively roll, not a steady gun platform".into());
        }

        let sea = match self.type_sea() {
            SeaType::BadSea  => "Caution: Lacks seaworthiness - very limited seakeeping ability".into(),
            SeaType::PoorSea => "Poor seaboat, wet and uncomfortable, reduced performance in heavy weather".into(),
            SeaType::GoodSea => "Good seaboat, rides out heavy weather easily".into(),
            SeaType::FineSea => format!("Excellent seaboat, comfortable, {}",
                    if self.wgt_guns() > 0.0 {
                        "can fire her guns in the heaviest weather"
                    } else {
                        "rides out heavy weather easily"
                    }).into(),
            SeaType::Error   => "Invalid SeaType".into(),
        };

        s.push(sea);

        s
    }

    // roll_period {{{2
    pub fn roll_period(&self) -> f64 {
        0.42 * self.hull.bb / self.metacenter().sqrt()
    }

    // steadiness {{{2
    pub fn steadiness(&self) -> f64 {
        f64::min(self.trim as f64 * self.seaboat(), 100.0)
    }


    // stability {{{2
    fn stability(&self) -> f64 {
        let a =
            (self.armor.ct_fwd.wgt(self.hull.d()) + self.armor.ct_aft.wgt(self.hull.d())) * 5.0 +
            (self.wgt_borne() + self.wgt_gun_armor()) * (2.0 * self.gun_super_factor() - 1.0) * 4.0 +
            self.wgts.hull as f64 * 2.0 +
            self.wgts.on as f64 * 3.0 +
            self.wgts.above as f64 * 4.0 +
            self.armor.upper.wgt(self.hull.d(), self.hull.cwp(), self.hull.b) * 2.0 +
            self.armor.main.wgt(self.hull.d(), self.hull.cwp(), self.hull.b) +
            self.armor.end.wgt(self.hull.d(), self.hull.cwp(), self.hull.b) +
            self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), 0.0) +
            // TODO: self.armor.deck.wgt(self.hull.clone(), self.wgt_mag(), self.wgt_engine()) +
            (self.wgt_hull_plus() + self.wgt_guns() + self.wgt_gun_mounts() - self.wgt_borne()) * 1.5 * self.hull.freeboard() / self.hull.t;

        let b = a +
            if self.deck_room() < 1.0 {
                (self.wgt_engine() + self.wgts.vital as f64 + self.wgts.void as f64) * (1.0 - self.deck_room().powf(2.0))
            } else { 0.0 };

        if b > 0.0 {
            ((self.hull.d() * (self.hull.bb / self.hull.t) / b) * 0.5).sqrt() *
            (8.76755 / self.hull.len2beam()).powf(0.25)
        } else {
            b
        }
    }

    // stability_adj {{{2
    pub fn stability_adj(&self) -> f64 {
        self.stability() * ((50.0 - self.trim as f64) / 150.0 + 1.0)
    }

    // d_factor {{{2
    pub fn d_factor(&self) -> f64 {
        f64::min(
            self.hull.d() /
            (
                self.engine.d_engine(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()) +
                    8.0 * self.wgt_borne() + self.wgt_armor() + self.wgts.wgt() as f64
            ),
            10.0
        )
    }

    // cap_calc_broadside {{{2
    pub fn cap_calc_broadside(&self) -> bool {
        for b in self.batteries.iter() {
            if ! b.broad_and_below() { return false; }
        }

        true
    }

    // flotation {{{2
    pub fn flotation(&self) -> f64 {
        let a = if self.cap_calc_broadside() {
                self.hull.free_cap(self.cap_calc_broadside())
            } else {
                self.hull.freeboard_dist()
            };

        let b = (a * self.hull.wp() / Hull::FT3_PER_TON_SEA + self.hull.d()) / 2.0;

        let c = b * self.stability_adj().powf(
            if self.stability_adj() > 1.0 { 0.5 } else { 4.0 }
            );

        let d = c * if self.str_comp() < 1.0 { self.str_comp() } else { 1.0 };

        let e = d / self.room().powf(if self.room() > 1.0 { 2.0 } else { 1.0 });

        f64::max(e * Self::year_adj(self.year), 0.0)
    }

    // str_cross {{{2
    pub fn str_cross(&self) -> f64 {
        let mut concentration: f64 = 1.0;

        if self.wgt_broad() > 0.0 {
            concentration = 1.0 + self.gun_concentration();
        }

        let mut str_cross = self.wgt_struct() / f64::sqrt(self.hull.bb * (self.hull.t + self.hull.freeboard_dist())) /
            ((self.hull.d() + ((self.wgt_broad() + self.wgt_borne() + self.wgt_gun_armor() + self.armor.ct_fwd.wgt(self.hull.d()) + self.armor.ct_aft.wgt(self.hull.d())) * (concentration * self.gun_super_factor()) + f64::max(self.engine.hp_max(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()), 0.0) / 100.0)) / self.hull.d()) * 0.6;

        if self.year < 1900 {
            str_cross *= 1.0 - (1900.0 - self.year as f64) / 100.0;
        }

        str_cross
    }

    // str_long {{{2
    pub fn str_long(&self) -> f64 {
        (
            self.wgt_hull_plus() + if self.armor.strengthened_bulkhead {
                    self.armor.bulkhead.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b)
                } else { 0.0 }
        ) /
            (
                (self.hull.lwl() / (self.hull.t + self.hull.free_cap(self.cap_calc_broadside()))).powf(2.0) *
                (
                    self.hull.d() +
                    self.armor.end.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b) *
                    3.0 + (
                        self.wgt_borne() +
                        self.wgt_gun_armor()
                        ) * self.super_factor_long() * 2.0
                )
            ) *
            850.0 * if self.year < 1900 { 1 - (1900 - self.year) / 100 } else { 1 } as f64
    }

    // str_comp {{{2
    pub fn str_comp(&self) -> f64 {
        if self.str_cross() > self.str_long() {
            self.str_long() * (self.str_cross() / self.str_long()).powf(0.25)
        } else {
            self.str_cross() * (self.str_long() / self.str_cross()).powf(0.1)
        }
    }

    // gun_concentration {{{2
    fn gun_concentration(&self) -> f64 {
        let mut concentration = 0.0;
        for b in self.batteries.iter() {
            concentration += b.concentration(self.wgt_broad());
        }
        concentration
    }

    // damage_shell_size {{{2
    pub fn damage_shell_size(&self) -> f64 {
        if self.batteries[0].cal > 0.0 {
            self.batteries[0].cal
        } else {
            6.0
        }
    }

    // damage_shell_num {{{2
    pub fn damage_shell_num(&self) -> f64 {
        self.flotation() / (
            self.damage_shell_size().powf(3.0) /
            2.0 * Self::year_adj(self.year) as f64
            )
    }

    // damage_shell_torp_num {{{2
    pub fn damage_torp_num(&self) -> f64 {
        (
            (
                (
                    (self.flotation() / 10_000.0).powf(1.0/3.0) +
                    (self.hull.bb / 75.0).powf(2.0) +
                    (
                        (self.armor.bulkhead.thick / 2.0 * self.armor.bulkhead.len / self.hull.lwl()) /
                        0.65 * self.armor.bulkhead.hgt / self.hull.t
                    ).powf(1.0/3.0) *
                    self.flotation() / 35_000.0 * self.hull.bb / 50.0
                ) / self.room() * self.hull.lwl() / (self.hull.lwl() + self.hull.bb)
            ) * if self.stability_adj() < 1.0 {
                    self.stability_adj().powf(4.0)
                } else {
                    1.0
                } * (1.0 - self.hull_space())
        ) * if self.torps[0].wgt_weaps() > 0.0 {
                1.313 / (self.torps[0].wgt_weaps() / self.torps[0].num as f64)
            } else {
                1.0
            }
    }

    // wgt_engine {{{2
    fn wgt_engine(&self) -> f64 {

        let p =
            if (self.hull.d() < 5000.0) && (self.hull.d() >= 600.0) && (self.d_factor() < 1.0)
            {
                1.0 - self.hull.d() / 5000.0
            } else if (self.hull.d() < 600.0) && (self.d_factor() < 1.0) {
                    0.88
                } else {
                    0.0
            };

        (self.engine.d_engine(self.hull.d(), self.hull.lwl(), self.hull.leff(), self.hull.cs(), self.hull.ws()) / 2.0) *
            self.d_factor().powf(p)
    }

    // wgt_struct {{{2
    pub fn wgt_struct(&self) -> f64 {
        (
            self.wgt_hull_plus() +
            if self.armor.strengthened_bulkhead {
                self.armor.bulkhead.wgt(self.hull.lwl(), self.hull.cwp(), self.hull.b)
            } else {
                0.0
            }
        ) * Self::POUND2TON / (
            self.hull.ws() +
            2.0 * self.hull.lwl() * self.hull.free_cap(self.cap_calc_broadside()) +
            self.hull.wp()
            )
    }

    // wgt_hull {{{2
    fn wgt_hull(&self) -> f64 {
        self.hull.d() -
            self.wgt_guns() -
            self.wgt_gun_mounts() -
            self.wgt_weaps() -
            self.wgt_armor() -
            self.wgt_engine() -
            self.wgt_load() -
            self.wgts.wgt() as f64
    }

    // wgt_hull_plus {{{2
    fn wgt_hull_plus(&self) -> f64 {
        self.wgt_hull() +
        self.wgt_guns() +
        self.wgt_gun_mounts() -
        self.wgt_borne()
    }

    // wgt_borne {{{2
    fn wgt_borne(&self) -> f64 {
        let mut wgt = 0.0;
        for b in self.batteries.iter() {
            wgt += b.gun_wgt() * b.mount_kind.wgt_adj();
        }
        wgt * 2.0
    }

    // wgt_weaps {{{2
    fn wgt_weaps(&self) -> f64 {
        let mut wgt = 0.0;
        for w in self.torps.iter() { wgt += w.wgt(); }
        for w in self.asw.iter()   { wgt += w.wgt(); }
        wgt += self.mines.wgt();

        wgt
    }

    // wgt_guns {{{2
    fn wgt_guns(&self) -> f64 {
        let mut wgt = 0.0;
        for b in self.batteries.iter() {
            wgt += b.gun_wgt();
        }
        wgt
    }

    // wgt_gun_mounts {{{2
    fn wgt_gun_mounts(&self) -> f64 {
        let mut wgt = 0.0;
        for b in self.batteries.iter() {
            wgt += b.mount_wgt();
        }
        wgt
    }

    // wgt_gun_armor {{{2
    fn wgt_gun_armor(&self) -> f64 {
        let mut wgt = 0.0;
        for b in self.batteries.iter() {
            wgt += b.armor_wgt(self.hull.clone());
        }
        wgt
    }

    // wgt_mag {{{2
    fn wgt_mag(&self) -> f64 {
        let mut wgt = 0.0;
        for b in self.batteries.iter() {
            wgt += b.mag_wgt();
        }
        wgt
    }

    // wgt_broad {{{2
    fn wgt_broad(&self) -> f64 {
        let mut broad = 0.0;
        for b in self.batteries.iter() {
            broad += b.broadside_wgt();
        }
        broad
    }

    // wgt_armor {{{2
    fn wgt_armor(&self) -> f64 {
        self.armor.wgt(self.hull.clone(), self.wgt_mag(), 0.0) + self.wgt_gun_armor()
        // TODO: self.armor.wgt(self.hull.clone(), self.wgt_mag(), self.wgt_engine()) + self.wgt_gun_armor()
    }

    // gun_wtf {{{2
    /// XXX: I have no idea wtf this is
    fn gun_wtf(&self) -> f64 {
        let mut wtf = 0.0;
        for b in self.batteries.iter() {
            if b.cal == 0.0 { continue; }
            wtf += (
                b.gun_wgt() +
                b.mount_wgt() +
                b.armor_wgt(self.hull.clone())
             ) *
                b.super_(self.hull.clone()) *
                b.mount_kind.wgt_adj();
        }
        wtf
    }

    // gun_super_factor {{{2
    fn gun_super_factor(&self) -> f64 {
        self.gun_wtf() / (self.wgt_gun_armor() + self.wgt_guns() + self.wgt_gun_mounts())
    }

    // super_factor_long {{{2
    pub fn super_factor_long(&self) -> f64 {
        let a = self.hull_room() *
            if (
                    self.batteries[0].groups[0].distribution == GunDistributionType::CenterlineEven ||
                    self.batteries[0].groups[0].distribution == GunDistributionType::SidesEven ||
                    self.batteries[0].groups[1].distribution == GunDistributionType::CenterlineEven ||
                    self.batteries[0].groups[1].distribution == GunDistributionType::SidesEven
                ) && (
                    self.batteries[0].mount_num == 3 ||
                    self.batteries[0].mount_num == 4
                )
            {
                self.gun_super_factor()
            } else {
                1.0
            };
        a *
            if (
                    self.batteries[0].groups[0].num_mounts() > 0 &&
                    self.batteries[0].groups[1].num_mounts() == 0 &&
                    self.batteries[0].groups[0].distribution.super_factor_long()
                ) || (
                    self.batteries[0].groups[1].num_mounts() > 0 &&
                    self.batteries[0].groups[0].num_mounts() == 0 &&
                    self.batteries[0].groups[1].distribution.super_factor_long()
                ) || (
                    self.batteries[0].groups[0].num_mounts() > 0 &&
                    self.batteries[0].groups[1].num_mounts() > 0 &&
                    (self.batteries[0].groups[0].distribution.g1_gun_position(self.hull.fd_len, self.hull.ad_len()) -
                     self.batteries[0].groups[1].distribution.g2_gun_position(self.hull.fd_len, self.hull.ad_len())).abs() < 0.2
                )
            {
                0.8 * self.gun_super_factor()
            } else {
                2.0 * self.gun_super_factor() - 1.0
            }
    }
    // percent_calc {{{2
    fn percent_calc(total: f64, portion: f64) -> f64 {
        if total > 0.0 {
            (portion / total) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)] // Ship {{{1
mod ship {
    use super::*;
    use crate::test_support::*;

    fn get_hull() -> Hull {

        let mut hull = Hull::default();

        hull.set_d(7000.0);
        hull.set_lwl(500.0);
        hull.b = 50.0;
        hull.bb = hull.b;
        hull.t = 10.0;
        hull.bow_angle = 0.0;
        hull.stern_overhang = 0.0;

        hull.fc_len = 0.20;
        hull.fc_fwd = 10.0;
        hull.fc_aft = 10.0;

        hull.fd_len = 0.30;
        hull.fd_fwd = hull.fc_len;
        hull.fd_aft = hull.fc_len;

        hull.ad_fwd = hull.fc_len;
        hull.ad_aft = hull.fc_len;

        hull.qd_len = 0.15;
        hull.qd_fwd = hull.fc_len;
        hull.qd_aft = hull.fc_len;

        hull.bow_type = BowType::Normal;
        hull.stern_type = SternType::Cruiser;

        hull
    }

    // Test year_adj {{{2
    macro_rules! test_year_adj {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, year) = $value;

                    assert_eq!(expected, to_place(Ship::year_adj(year), 5));
                }
            )*
        }
    }

    test_year_adj! {
        // name:    (year_adj, year)
        year_adj_1: (0.985, 1889),
        year_adj_2: (1.0, 1890),
        year_adj_3: (1.0, 1949),
        year_adj_4: (1.0, 1950),
        year_adj_5: (0.0, 1951),
    }

    // Test deck_space {{{2
    macro_rules! test_deck_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind) = $value;

                    let mut ship = Ship::default();
                    ship.hull = get_hull().clone();

                    ship.torps[0].year = 1920;
                    ship.torps[0].num = 3;
                    ship.torps[0].mounts = 2;
                    ship.torps[0].diam = 20.0;
                    ship.torps[0].len = 10.0;
                    ship.torps[0].mount_kind = kind;

                    ship.torps[1].num = 0;

                    assert_eq!(expected, to_place(ship.deck_space(), 4));
                }
            )*
        }
    }

    test_deck_space! {
        // name:    (deck_space, kind)
        deck_space_1: (0.002, TorpedoMountType::FixedTubes),
        deck_space_2: (0.0039, TorpedoMountType::DeckSideTubes),
        deck_space_3: (0.0415, TorpedoMountType::CenterTubes),
        deck_space_4: (0.0039, TorpedoMountType::DeckReloads),
        deck_space_5: (0.0, TorpedoMountType::BowTubes),
        deck_space_6: (0.0, TorpedoMountType::SternTubes),
        deck_space_7: (0.0, TorpedoMountType::BowAndSternTubes),
        deck_space_8: (0.0, TorpedoMountType::SubmergedSideTubes),
        deck_space_9: (0.0, TorpedoMountType::SubmergedReloads),
    }

    // Test hull_space {{{2
    macro_rules! test_hull_space {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, kind) = $value;

                    let mut ship = Ship::default();
                    ship.hull = get_hull().clone();

                    ship.torps[0].year = 1920;
                    ship.torps[0].num = 3;
                    ship.torps[0].mounts = 2;
                    ship.torps[0].diam = 20.0;
                    ship.torps[0].len = 10.0;
                    ship.torps[0].mount_kind = kind;

                    ship.torps[1].num = 0;

                    assert_eq!(expected, to_place(ship.hull_space(), 4));
                }
            )*
        }
    }

    test_hull_space! {
        // name:    (hull_space, kind)
        hull_space_1: (0.0, TorpedoMountType::FixedTubes),
        hull_space_2: (0.0, TorpedoMountType::DeckSideTubes),
        hull_space_3: (0.0, TorpedoMountType::CenterTubes),
        hull_space_4: (0.0, TorpedoMountType::DeckReloads),
        hull_space_5: (0.0064, TorpedoMountType::BowTubes),
        hull_space_6: (0.0064, TorpedoMountType::SternTubes),
        hull_space_7: (0.0064, TorpedoMountType::BowAndSternTubes),
        hull_space_8: (0.0064, TorpedoMountType::SubmergedSideTubes),
        hull_space_9: (0.0011, TorpedoMountType::SubmergedReloads),
    }


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
        // name:            (crew, d)
        crew_max_d_eq_zero: (0, 0.0),
        crew_max_d_eq_1000: (115, 1000.0),
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
        // name:            (crew, d)
        crew_min_d_eq_zero: (0, 0.0),
        crew_min_d_eq_1000: (88, 1000.0),
    }
}

// SeaType {{{1
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum SeaType {
    #[default]
    BadSea,
    PoorSea,
    FineSea,
    GoodSea,
    Error, // This is an...error if it shows up anywhere
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


// FuelType {{{1
bitflags! {
    #[derive(PartialEq, Serialize, Deserialize, Default, Debug, Clone)]
    pub struct FuelType: u8 {
        const Coal     = 1 << 0;
        const Oil      = 1 << 1;
        const Diesel   = 1 << 2;
        const Gasoline = 1 << 3;
        const Battery  = 1 << 4;
    }
}

impl fmt::Display for FuelType { // {{{2
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            bitflags_match!(*self, {
                Self::Coal        => "Coal fired boilers",
                Self::Oil         => "Oil fired boilers",
                Self::Coal |
                    Self::Oil     => "Coal and oil fired boilers",

                Self::Coal |
                    Self::Diesel  => "Coal fired boilers plus diesel motors",
                Self::Oil |
                    Self::Diesel  => "Oil fired boilers plus diesel motors",
                Self::Coal |
                    Self::Oil |
                    Self::Diesel  => "Coal and oil fired boilers plus diesel motors",

                Self::Diesel      => "Diesel internal combustion motors",
                Self::Diesel |
                    Self::Battery => "Diesel internal combustion engines plus batteries",

                Self::Gasoline    => "Gasoline internal combustion motors",
                Self::Gasoline |
                    Self::Battery => "Gasoline internal combustion motors plus batteries",

                Self::Battery     => "Battery powered",

                _                 => "ERROR: Revise fuels",
            })
        )
    }
}

impl FuelType { // {{{2
    // is_steam {{{2
    /// Return true if the fuel indicates a steam engine.
    ///
    pub fn is_steam(&self) -> bool {
        self.contains(Self::Coal) || self.contains(Self::Oil)
    }
}

#[cfg(test)] // FuelType {{{1
mod fuel_type {
    use super::*;

    // Test is_steam {{{2
    macro_rules! test_is_steam {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, fuel) = $value;

                    assert_eq!(expected, fuel.is_steam());
                }
            )*
        }
    }

    test_is_steam! {
        // name:           (is_steam, fuel)
        is_steam_coal:     (true, FuelType::Coal),
        is_steam_oil:      (true, FuelType::Oil),
        is_steam_diesel:   (false, FuelType::Diesel),
        is_steam_gas:      (false, FuelType::Gasoline),
        is_steam_battery:  (false, FuelType::Battery),
        is_steam_multiple: (true, FuelType::Coal | FuelType::Diesel),
    }
}

// BoilerType {{{1
bitflags! {
    #[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
    pub struct BoilerType: u8 {
        /// Simple, reciprocating engines.
        const Simple  = 1 << 0;
        /// Complex, reciprocating engines.
        const Complex = 1 << 1;
        /// Steam turbine engines.
        const Turbine = 1 << 2;
    }
}

impl fmt::Display for BoilerType { // {{{1
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            bitflags_match!(*self, {
                Self::Simple => "simple receiprocating steam engines",
                Self::Complex => "complex receiprocating steam engines",
                Self::Turbine => "steam turbines",

                Self::Simple |
                    Self::Complex => "reciprocating steam engines",

                Self::Simple |
                    Self::Turbine => "reciprocating cruising steam engines and steam turbines",

                Self::Simple |
                    Self::Complex | 
                    Self::Turbine => "ERROR: Too many types of steam engines",

                _ => "ERROR: No steam engines",
            })
        )
    }
}

// BoilerType Implementation {{{1
impl BoilerType {
    // hp_type {{{2
    pub fn hp_type(&self) -> String {
        match self.is_reciprocating() {
            true => "ihp".into(),
            false => "shp".into(),
        }
    }

    // num_engines {{{2
    /// Number of steam engines.
    ///
    /// Each bit set is one engine.
    ///
    pub fn num_engines(&self) -> u32 {
        u8::count_ones(self.bits())
    }

    // is_simple {{{2
    /// Return true if the boiler has simple reciprocating engines.
    ///
    pub fn is_simple(&self) -> bool {
        self.contains(Self::Simple)
    }

    // is_complex {{{2
    /// Return true if the boiler has complex reciprocating engines.
    ///
    pub fn is_complex(&self) -> bool {
        self.contains(Self::Complex)
    }

    // is_reciprocating {{{2
    /// Return true if the boiler has any type of reciprocating engines.
    ///
    pub fn is_reciprocating(&self) -> bool {
        self.is_simple() || self.is_complex()
    }

    // is_turbine {{{2
    /// Return true if the boiler has steam turbines.
    ///
    pub fn is_turbine(&self) -> bool {
        self.contains(Self::Turbine)
    }

    // d_engine_factor {{{2
    pub fn d_engine_factor(&self, year: u32, fuel: FuelType) -> f64 {
        let a = if self.is_simple() {
                    if year <= 1884 { 1.2 + (year - 1860) as f64 * 0.05 }
               else if year <= 1949 { 2.45 + (year - 1885) as f64 * 0.025 }
               else                 { 4.075 }
            } else { 0.0 };

        let b = if self.is_complex() {
                    if year <= 1905 { 1.2 + (year - 1860) as f64 * 0.05 }
               else if year <= 1910 { 3.5 + (year - 1906) as f64 }
               else if year <= 1949 { 7.5 + (year - 1910) as f64 * 0.025 }
               else                 { 8.5 }
            } else { 0.0 };

        let c = if self.is_turbine() || ! fuel.is_steam()
            {
                    if year <= 1897 { 1.2 + (year - 1860) as f64 * 0.05 }
               else if year <= 1902 { 1.0 + (year - 1898) as f64 * 0.5 }
               else if year <= 1909 { 4.0 + (year - 1903) as f64 }
               else if year <= 1949 { 11.0 + (year - 1910) as f64 * 0.2 }
               else                 { 19.0 }
            } else { 0.0 };

        a + b + c
    }

    // bunker_factor {{{2
    pub fn bunker_factor(&self, year: u32) -> f64 {
        if self.is_reciprocating() {
            1.0 - (1910 - year) as f64 / 70.0 
        } else if year < 1898 {
            1.0 - (1910 - year) as f64 / 70.0
        } else if year < 1920 {
            1.0 + (year - 1910) as f64 / 20.0
        } else if year < 1950 {
            1.5 + (year - 1920) as f64 / 60.0
        } else {
            2.0
        }
    }
}

#[cfg(test)] // BoilerType {{{1
mod boiler_type {
    use super::*;
    use crate::test_support::*;

    // Test d_engine_factor {{{2
    macro_rules! test_d_engine_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, boiler, fuel, year) = $value;

                    assert_eq!(expected, to_place(boiler.d_engine_factor(year, fuel), 3));
                }
            )*
        }
    }

    test_d_engine_factor! {
        // name:                  (d_engine_factor, boiler, fuel, year)
        // Test the years
        d_engine_factor_simple_1: (2.4, BoilerType::Simple, FuelType::Oil, 1884),
        d_engine_factor_simple_2: (4.05, BoilerType::Simple, FuelType::Oil, 1949),
        d_engine_factor_simple_3: (4.075, BoilerType::Simple, FuelType::Oil, 1950),

        d_engine_factor_complex_1: (3.45, BoilerType::Complex, FuelType::Oil, 1905),
        d_engine_factor_complex_2: (7.5, BoilerType::Complex, FuelType::Oil, 1910),
        d_engine_factor_complex_3: (8.475, BoilerType::Complex, FuelType::Oil, 1949),
        d_engine_factor_complex_4: (8.5, BoilerType::Complex, FuelType::Oil, 1950),

        d_engine_factor_other_1: (3.05, BoilerType::Turbine, FuelType::Oil, 1897),
        d_engine_factor_other_2: (3.0, BoilerType::Turbine, FuelType::Oil, 1902),
        d_engine_factor_other_3: (10.0, BoilerType::Turbine, FuelType::Oil, 1909),
        d_engine_factor_other_4: (18.8, BoilerType::Turbine, FuelType::Oil, 1949),
        d_engine_factor_other_5: (19.0, BoilerType::Turbine, FuelType::Oil, 1950),

        // Test ! fuel.is_steam()
        d_engine_factor_not_steam: (4.825, BoilerType::Simple, FuelType::Gasoline, 1900),

        // Test sum of the three checks
        d_engine_factor_recip:           (6.025, BoilerType::Simple | BoilerType::Complex, FuelType::Oil, 1900),
        d_engine_factor_simple_turbine:  (4.825, BoilerType::Simple | BoilerType::Turbine, FuelType::Oil, 1900),
        d_engine_factor_complex_turbine: (5.2, BoilerType::Complex | BoilerType::Turbine, FuelType::Oil, 1900),
    }
}

// DriveType {{{1
bitflags! {
    #[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
    pub struct DriveType: u8 {
        const Direct    = 1 << 0;
        const Geared    = 1 << 1;
        const Electric  = 1 << 2;
        const Hydraulic = 1 << 3;
    }
}

impl fmt::Display for DriveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            bitflags_match!(*self, {
                Self::Direct    => "Direct drive",
                Self::Geared    => "Geared drive",
                Self::Electric  => "Electric motors",
                Self::Hydraulic => "Hydraulic drive",

                Self::Geared |
                    Self::Electric => "Electric cruising motors plus geared drives",

                // TODO: DriveType {0}   => "ERROR: No drive to shaft",
                _               => "ERROR: Revise drives",
            })
        )
    }
}

// MineType {{{1
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum MineType {
    #[default]
    SternRails,
    BowTubes,
    SternTubes,
    SideTubes,
}

impl From<String> for MineType {
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

impl MineType {
    pub fn wgt_factor(&self) -> f64 {
        match self {
            Self::SternRails => 0.25,
            Self::BowTubes   => 1.0,
            Self::SternTubes => 1.0,
            Self::SideTubes  => 1.0,
        }
    }

    pub fn desc(&self) -> String {
        match self {
            Self::SternRails => "in Above water - Stern racks/rails",
            Self::BowTubes   => "in Below water - bow tubes",
            Self::SternTubes => "",
            Self::SideTubes  => "",
        }.into()
    }
}

#[cfg(test)] // MineType {{{1
mod mine_type {
    use super::*;

    // Test wgt_factor {{{2
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


// ASWType {{{1
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum ASWType {
    #[default]
    SternRacks,
    Throwers,
    Hedgehogs,
    SquidMortars,
}

impl From<String> for ASWType {
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

impl ASWType {
    pub fn mount_wgt_factor(&self) -> f64 {
        match self {
            Self::SternRacks   => 0.25,
            Self::Throwers     => 0.5,
            Self::Hedgehogs    => 0.5,
            Self::SquidMortars => 10.0,
        }
    }

    pub fn inline_desc(&self) -> String {
        match self {
            Self::SternRacks   => "Depth Charges",
            Self::Throwers     => "Depth Charges",
            Self::Hedgehogs    => "ahead throwing AS Mortars",
            Self::SquidMortars => "trainable AS Mortars",
        }.into()
    }

    pub fn desc(&self) -> String {
        match self {
            Self::SternRacks   => "in Stern depth charge racks",
            Self::Throwers     => "in Depth depth throwers",
            Self::Hedgehogs    => "",
            Self::SquidMortars => "",
        }.into()
    }
}

#[cfg(test)] // ASWType {{{1
mod asw_type {
    use super::*;

    // Test mount_wgt_factor {{{2
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

// TorpedoMountType {{{1
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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

impl From<String> for TorpedoMountType {
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

impl TorpedoMountType {
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

#[cfg(test)] // TorpedoMountType {{{1
mod torpedo_mount_type {
    use super::*;
    use crate::test_support::*;

    // Test wgt_factor {{{2
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

    // Test hull_space {{{2
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

    // Test deck_space {{{2
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

// GunType {{{1
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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

impl From<String> for GunType {
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
    // armor_face_wgt {{{2
    pub fn armor_face_wgt(&self) -> f64 {
        match self {
            Self::MuzzleLoading => 1.0,
            Self::BreechLoading => 1.0,
            Self::QuickFiring   => 1.0,
            Self::AntiAir       => 0.333,
            Self::DualPurpose   => 1.0,
            Self::RapidFire     => 1.0,
            Self::MachineGun    => 1.0,
        }
    }

    // armor_face_wgt_if_no_back {{{2
    pub fn armor_face_wgt_if_no_back(&self) -> f64 {
        match self {
            Self::MuzzleLoading => 1.0,
            Self::BreechLoading => 1.0,
            Self::QuickFiring   => 1.0,
            Self::AntiAir       => 1.0,
            Self::DualPurpose   => 1.0,
            Self::RapidFire     => 1.0,
            Self::MachineGun    => 0.333,
        }
    }

    // wgt_sm {{{2
    pub fn wgt_sm(&self) -> f64 {
        match self {
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
        match self {
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

#[cfg(test)] // GunType {{{1
mod gun_type {
    use super::*;

    // Test wgt_sm {{{2
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

    // Test wgt_lg {{{2
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
#[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
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

impl From<String> for MountType {
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
impl fmt::Display for MountType { // {{{1
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
impl MountType { // {{{1
    // armor_face_wgt {{{2
    pub fn armor_face_wgt(&self) -> f64 {
        use std::f64::consts::PI;
        match self {
            Self::Broadside      => 1.0,
            Self::ColesTurret    => PI / 2.0,
            Self::OpenBarbette   => 0.0,
            Self::ClosedBarbette => 0.5,
            Self::DeckAndHoist   => 0.5,
            Self::Deck           => 0.5,
            Self::Casemate       => 1.0,
        }
    }

    // armor_face_wgt_if_no_back {{{2
    pub fn armor_face_wgt_if_no_back(&self) -> f64 {
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

    // gunhouse_hgt_factor {{{2
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

    // armor_back_wgt {{{2
    pub fn armor_back_wgt(&self) -> f64 {
        match self {
            Self::Broadside      => 0.0,
            Self::ColesTurret    => 0.0,
            Self::OpenBarbette   => 0.0,
            Self::ClosedBarbette => 2.5,
            Self::DeckAndHoist   => 2.5,
            Self::Deck           => 2.5,
            Self::Casemate       => 0.0,
        }
    }

    // armor_back_wgt_factor {{{2
    pub fn armor_back_wgt_factor(&self) -> f64 {
        match self {
            Self::Broadside      => 0.75,
            Self::ColesTurret    => 1.0,
            Self::OpenBarbette   => 0.75,
            Self::ClosedBarbette => 0.75,
            Self::DeckAndHoist   => 0.75,
            Self::Deck           => 0.75,
            Self::Casemate       => 0.75,
        }
    }

    // armor_barb_wgt {{{2
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

    // wgt {{{2
    pub fn wgt(&self) -> f64 {
        match self {
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
        match self {
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

#[cfg(test)] // MountType {{{1
mod mount_type {
    use super::*;

    use std::f64::consts::PI;

    // Test armor_wgt_adj {{{2
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

    // Test armor_wgt {{{2
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

    // Test armor_barb_wgt {{{2
    macro_rules! test_armor_barb_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.armor_barb_wgt());
                }
            )*
        }
    }

    test_armor_barb_wgt! {
        // name:                    (factor, mount)
        barb_wgt_broad:       (0.0, MountType::Broadside),
        barb_wgt_coles:       (0.0, MountType::ColesTurret),
        barb_wgt_open_barb:   (0.6416, MountType::OpenBarbette),
        barb_wgt_closed_barb: (0.5, MountType::ClosedBarbette),
        barb_wgt_deckhoist:   (0.1, MountType::DeckAndHoist),
        barb_wgt_deck:        (0.0, MountType::Deck),
        barb_wgt_casemate:    (0.1, MountType::Casemate),
    }

    // Test armor_back_wgt {{{2
    macro_rules! test_armor_back_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.armor_back_wgt());
                }
            )*
        }
    }

    test_armor_back_wgt! {
        // name:                    (factor, mount)
        back_wgt_broad:       (0.0, MountType::Broadside),
        back_wgt_coles:       (0.0, MountType::ColesTurret),
        back_wgt_open_barb:   (0.0, MountType::OpenBarbette),
        back_wgt_closed_barb: (2.5, MountType::ClosedBarbette),
        back_wgt_deckhoist:   (2.5, MountType::DeckAndHoist),
        back_wgt_deck:        (2.5, MountType::Deck),
        back_wgt_casemate:    (0.0, MountType::Casemate),
    }

    // Test armor_back_wgt_factor {{{2
    macro_rules! test_armor_back_wgt_factor {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.armor_back_wgt_factor());
                }
            )*
        }
    }

    test_armor_back_wgt_factor! {
        // name:                    (factor, mount)
        back_wgt_factor_broad:       (0.75, MountType::Broadside),
        back_wgt_factor_coles:       (1.0, MountType::ColesTurret),
        back_wgt_factor_open_barb:   (0.75, MountType::OpenBarbette),
        back_wgt_factor_closed_barb: (0.75, MountType::ClosedBarbette),
        back_wgt_factor_deckhoist:   (0.75, MountType::DeckAndHoist),
        back_wgt_factor_deck:        (0.75, MountType::Deck),
        back_wgt_factor_casemate:    (0.75, MountType::Casemate),
    }

    // Test armor_face_wgt {{{2
    macro_rules! test_armor_face_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.armor_face_wgt());
                }
            )*
        }
    }

    test_armor_face_wgt! {
        // name:                    (factor, mount)
        face_wgt_broad:       (1.0, MountType::Broadside),
        face_wgt_coles:       (PI / 2.0, MountType::ColesTurret),
        face_wgt_open_barb:   (0.0, MountType::OpenBarbette),
        face_wgt_closed_barb: (0.5, MountType::ClosedBarbette),
        face_wgt_deckhoist:   (0.5, MountType::DeckAndHoist),
        face_wgt_deck:        (0.5, MountType::Deck),
        face_wgt_casemate:    (1.0, MountType::Casemate),
    }

    // Test armor_face_wgt_if_no_back {{{2
    macro_rules! test_armor_face_wgt_if_no_back {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, mount) = $value;

                    assert_eq!(expected, mount.armor_face_wgt_if_no_back());
                }
            )*
        }
    }

    test_armor_face_wgt_if_no_back! {
        // name:                    (factor, mount)
        face_wgt_if_no_back_broad:       (0.0, MountType::Broadside),
        face_wgt_if_no_back_coles:       (0.0, MountType::ColesTurret),
        face_wgt_if_no_back_open_barb:   (0.0, MountType::OpenBarbette),
        face_wgt_if_no_back_closed_barb: (1.0, MountType::ClosedBarbette),
        face_wgt_if_no_back_deckhoist:   (1.0, MountType::DeckAndHoist),
        face_wgt_if_no_back_deck:        (1.0, MountType::Deck),
        face_wgt_if_no_back_casemate:    (0.0, MountType::Casemate),
    }
}

// GunDistributionType {{{1
#[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
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

impl From<String> for GunDistributionType {
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

impl fmt::Display for GunDistributionType { // {{{1
// TODO: look at source (line 10809) for how to adjust this based on number of mounts
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

impl GunDistributionType { // {{{1
    // desc {{{2
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

    // super_aft {{{2
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

    // mounts_fwd {{{2
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

    // free {{{2
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

    // gun_position {{{2
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

    // g1_gun_position {{{2
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
    // g2_gun_position {{{2
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

    // super_factor_long {{{2
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

#[cfg(test)] // GunDistributionType {{{1
mod gun_dist_type {
    use super::*;
    use crate::test_support::*;

    // Test g1_gun_position {{{2
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

    // Test mounts_fwd {{{2
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

    // Test free {{{2
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
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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

impl From<String> for GunLayoutType {
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

impl fmt::Display for GunLayoutType { // {{{1
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

impl GunLayoutType { // {{{1
    // num_guns {{{2
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

    // diameter_calc_nums {{{2
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

    // wgt_adj {{{2
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

// DeckType {{{1
#[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
pub enum DeckType {
    #[default]
    MultipleArmored,
    SingleArmored,
    MultipleProtected,
    SingleProtected,
    BoxOverMachinery,
    BoxOverMagazine,
    BoxOverBoth,
}

impl From<String> for DeckType {
    fn from(index: String) -> Self {
        index.as_str().into()
    }
}

impl From<&str> for DeckType {
    fn from(index: &str) -> Self {
        match index {
            "1" => Self::SingleArmored,
            "2" => Self::MultipleProtected,
            "3" => Self::SingleProtected,
            "4" => Self::BoxOverMachinery,
            "5" => Self::BoxOverMagazine,
            "6" => Self::BoxOverBoth,
            "0" | _ => Self::MultipleArmored,
        }
    }
}

impl fmt::Display for DeckType { // {{{1
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::MultipleArmored   => "Armoured deck - multiple decks",
                Self::SingleArmored     => "Armoured deck - single deck",
                Self::MultipleProtected => "Protected deck - multiple decks",
                Self::SingleProtected   => "Protected deck - singls deck",
                Self::BoxOverMachinery  => "Box over machinery",
                Self::BoxOverMagazine   => "Box over magazines",
                Self::BoxOverBoth       => "Box over machiner & magazines",
            }
        )
    }
}

pub mod unit_types { // {{{1
    use serde::{Serialize, Deserialize};
    use std::fmt;

    // Units {{{2
    #[derive(PartialEq, Serialize, Deserialize, Default, Clone, Copy, Debug)]
    pub enum Units {
        #[default]
        Imperial,
        Metric
    }

    // Convert from String {{{3
    impl From<String> for Units { 
        fn from(index: String) -> Self {
            index.as_str().into()
        }
    }

    impl From<&str> for Units {
        fn from(index: &str) -> Self {
            match index {
                "1"     => Self::Metric,
                "0" | _ => Self::Imperial,
            }
        }
    }

    impl fmt::Display for Units { // {{{3
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}",
                match self {
                    Self::Imperial => "imperial",
                    Self::Metric   => "metric",
                }
            )
        }
    }

    pub enum UnitType { // {{{2
        LengthSmall,
        LengthLong,
        Area,
        Weight,
        Power, 
        WeightPerArea,
    }

    // Conversion constants {{{2
    const INCH2MM: f64         = 25.4;
    const FEET2METERS: f64     = 0.3048;
    const SQFEET2SQMETERS: f64 = 0.092903;
    const POUND2KG: f64        = 0.45359236;
    const HP2KW: f64           = 0.746;

    // Function {{{2
    //
    pub fn metric(imperial: f64, unit_type: UnitType, units: Units) -> f64 { // {{{3
        if units == Units::Metric { return imperial; }

        match unit_type {
            UnitType::LengthSmall => imperial * INCH2MM,
            UnitType::LengthLong => imperial * FEET2METERS,
            UnitType::Area => imperial * SQFEET2SQMETERS,
            UnitType::Weight => imperial * POUND2KG,
            UnitType::Power => imperial * HP2KW,
            UnitType::WeightPerArea => imperial / SQFEET2SQMETERS * POUND2KG,
        }
    }

}
