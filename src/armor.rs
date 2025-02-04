use serde::{Serialize, Deserialize};

use std::f64::consts::PI;

const INCH: f64 = 0.0185; 

// Armor {{{1
#[derive(Serialize, Deserialize, Debug)]
pub struct Armor {
    pub main: Belt,
    pub end: Belt,
    pub upper: Belt,
    pub incline: f64,
    pub bulge: Belt,
    pub bulkhead: Belt,
    pub beam_between: f64,
    pub deck: Deck,
    pub ct_fwd: CT,
    pub ct_aft: CT,
}

impl Default for Armor { // {{{1
    fn default() -> Self {
        Armor {
            main: Belt::new(BeltType::Main),
            end: Belt::new(BeltType::End),
            upper: Belt::new(BeltType::Upper),
            incline: 0.0,
            bulge: Belt::new(BeltType::Bulge),
            bulkhead: Belt::new(BeltType::Bulkhead),
            beam_between: 0.0,
            deck: Deck::default(),
            ct_fwd: CT::default(),
            ct_aft: CT::default(),
        }
    }
}

impl Armor { // {{{1
    // belt_coverage {{{2
    pub fn belt_coverage(&self, lwl: f64) -> f64 {
        self.main.len / (lwl * 0.65)
    }

    // max_hgt {{{2
    pub fn max_belt_hgt(&self, t: f64, dist: f64) -> f64 {
        (t + dist) * (1.0 / (self.incline * PI / 180.0).abs().cos()) + 0.02
    }

    // new {{{2
    pub fn new() -> Armor {
        Default::default()
    }
}


// Belt {{{1
#[derive(Serialize, Deserialize, Debug)]
pub struct Belt {
    pub thick: f64,
    pub len: f64,
    pub hgt: f64,
        kind: BeltType, // Belt kind cannot be changed after creation
}

impl Belt { // {{{1
    // wgt {{{2
    pub fn wgt(&self, lwl: f64, cwp: f64, b: f64) -> f64 {
        let adj = match self.kind {
            BeltType::Main     => 1.0,
            BeltType::Upper    => 1.0,
            BeltType::End      => 0.0,
            BeltType::Bulge    => 0.0,
            BeltType::Bulkhead => 0.0,
        };

        (self.len + adj * ((lwl - self.len)/lwl).powf(1.0 - cwp) * b) * self.hgt * self.thick * INCH * 2.0
    }

    // new {{{2
    pub fn new(kind: BeltType) -> Belt {
        Belt {
            thick: 0.0,
            len: 0.0,
            hgt: 0.0,
            kind,
        }
    }
}

#[cfg(test)] // Belt {{{1
mod belt {
    use super::*;

    // Test stem_len {{{2
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let lwl = 500.0;
                    let cwp = 0.5;
                    let b = 10.0;

                    let (expected, thick, len, hgt, kind) = $value;
                    let mut belt = Belt::new(kind);
                    belt.thick = thick; belt.len = len; belt.hgt = hgt;

                    println!("{}",belt.wgt(lwl, cwp, b));
                    assert!(expected == belt.wgt(lwl, cwp, b));
                }
            )*
        }
    }
    test_wgt! {
        // name: (wgt, thick, len, hgt, kind)
        zero: (0.0, 0.0, 0.0, 0.0, BeltType::Main),
        main: (40.30938060669968, 1.0, 100.0, 10.0, BeltType::Main),
        end: (37.0, 1.0, 100.0, 10.0, BeltType::End),
        upper: (40.30938060669968, 1.0, 100.0, 10.0, BeltType::Upper),
        bulge: (37.0, 1.0, 100.0, 10.0, BeltType::Bulge),
        bulkhead: (37.0, 1.0, 100.0, 10.0, BeltType::Bulkhead),
    }
}

// BeltType {{{1
#[derive(Serialize, Deserialize, Debug)]
pub enum BeltType {
    Main,
    End,
    Upper,
    Bulge,
    Bulkhead,
}

// CT {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CT {
    pub thick: f64,
}

impl CT {
    // wgt {{{2
    pub fn wgt(&self, d: f64) -> f64 {
        10.0 * (d / 10_000.0).powf(2.0/3.0) * self.thick
    }
    // new {{{2
    pub fn new() -> CT {
        Default::default()
    }
}

#[cfg(test)] // CT {{{1
mod ct {
    use super::*;

    // Test wgt {{{2
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let d = 1000.0;
                    let (expected, thick) = $value;
                    let mut ct = CT::default();
                    ct.thick = thick;

                    assert!(expected == ct.wgt(d));
                }
            )*
        }
    }
    test_wgt! {
        // name: (wgt, thick)
        zero: (0.0, 0.0),
        test: (2.154434690031884, 1.0),
    }
}

// Deck {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Deck {
    pub kind: u32,
    pub fc: u32,
    pub fd: u32,
    pub ad: u32,
    pub qd: u32,
}

impl Deck {
    // wgt {{{2
    pub fn wgt(&self, lwl: f64, b: f64, fc_len: f64, qd_len: f64, cwp: f64) -> f64 {
        let fc = self.fc as f64;
        let fd = self.fd as f64;
        let ad = self.ad as f64;
        let qd = self.qd as f64;

        let wgt = 1.0; // lookup(deck_armor_type, deck_armor_types[type], deck_armor_types[weight])
        let wgt = wgt * (fd + ad);
        let wgt = wgt + (fc_len * 2.0).powf(1.0 - cwp.powf(2.0)) * b * lwl * fc_len * 0.5 * fc;
        let wgt = wgt + qd_len.powf(1.0 - cwp) * b * lwl * qd_len / 4.0;
        let wgt = wgt + ((qd_len.powf(1.0 - cwp) + (qd_len*2.0).powf(1.0 - cwp)) * b * lwl * qd_len / 4.0) * qd;
        wgt * INCH
    }

    // new {{{2
    pub fn new() -> Deck {
        Default::default()
    }
}
