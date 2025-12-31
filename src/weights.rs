use serde::{Serialize, Deserialize};

// MiscWgts {{{1
/// Miscellaneous weights throughout the ship.
///
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MiscWgts {
    /// Extra weight in the vital spaces.
    pub vital: u32,
    /// Extra weight in the hull.
    pub hull: u32,
    /// Extra weight on the deck.
    pub on: u32,
    /// Extra weight above the deck.
    pub above: u32,
    /// Extra displacement given to void space.
    pub void: u32,
}

impl MiscWgts { // {{{2
    // wgt {{{3
    /// Total of miscellaneous weights.
    ///
    pub fn wgt(&self) -> u32 {
        self.vital + self.hull + self.on + self.above + self.void
    }
}

// Testing {{{2
//
#[cfg(test)]
mod misc_wgts {
    use super::*;

    // wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, vital, hull, on, above, void) = $value;
                    let misc_wgts = MiscWgts {
                        vital: vital,
                        hull: hull,
                        on: on,
                        above: above,
                        void: void,
                    };

                    assert!(expected == misc_wgts.wgt());
                }
            )*
        }
    }
    test_wgt! {
        // name: (wgt, vital, hull, on, above, void)
        wgt_sum: (11_111, 1, 10, 100, 1_000, 10_000),
    }

}

