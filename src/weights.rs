use serde::{Serialize, Deserialize};

// MiscWgts {{{1
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MiscWgts {
    pub vital: u32,
    pub hull: u32,
    pub on: u32,
    pub above: u32,
    pub void: u32,
}

impl MiscWgts {
    // new {{{2
    pub fn new() -> MiscWgts {
        Default::default()
    }

    // wgt {{{2
    pub fn wgt(&self) -> u32 {
        self.vital + self.hull + self.on + self.above + self.void
    }
}


