use serde::{Deserialize, Serialize};

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
mod tests {
    use super::*;

    // wgt {{{3
    macro_rules! test_wgt {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, misc_wgts) = $value;

                    assert_eq!(expected, misc_wgts.wgt());
                }
            )*
        }
    }
    test_wgt! {
        // name: (wgt, MiscWgts)
        wgt_default: (0, MiscWgts::default()),
        wgt_sum:     (11_111, MiscWgts {
            vital: 1,
            hull: 10,
            on: 100,
            above: 1_000,
            void: 10_000,
        }),
        wgt_vital:   (1, MiscWgts { vital: 1, ..Default::default() }),
        wgt_hull:    (1, MiscWgts { hull:  1, ..Default::default() }),
        wgt_on:      (1, MiscWgts { on:    1, ..Default::default() }),
        wgt_above:   (1, MiscWgts { above: 1, ..Default::default() }),
        wgt_void:    (1, MiscWgts { void:  1, ..Default::default() }),
    }
}
