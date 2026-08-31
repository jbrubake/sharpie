// Macros {{{1
//
// choice_enum {{{2
/// Generate boilerplate for enums representing a list of user choices, e.g., BowType and GunType.
///
/// Expands to:
///     ALL: every variant, in .sship index order
///     label(), index(), from_index(): presentation-layer helpers
///     fmt::Display: report prose
///     From<&str>, From<String>: parse a decimal index string
///
/// The variants must be listed in SpringSharp (.sship) index order, which is not always
/// declaration order. Each row carries the menu label followed by an optional report
/// prose string; when the prose is omitted, fmt::Display falls back to the label.
/// Parsing accepts any non-negative decimal integer; unknown or out-of-range values
/// yield the default variant.
///
macro_rules! choice_enum {
    ($name:ident { $( $variant:ident $( ( $init:expr ) )? => ( $label:expr, $display:expr ) ),+ $(,)? }) => {
        choice_enum!(@impl $name {
            $( $variant $( ( $init ) )? => ( $label, $display ) ),+
        });
    };

    ($name:ident { $( $variant:ident $( ( $init:expr ) )? => ( $label:expr ) ),+ $(,)? }) => {
        choice_enum!(@impl $name {
            $( $variant $( ( $init ) )? => ( $label, $label ) ),+
        });
    };

    (@impl $name:ident { $( $variant:ident $( ( $init:expr ) )? => ( $label:expr, $display:expr ) ),+ $(,)? }) => {
        impl $name {
            /// Every variant, in .sship index order.
            pub const ALL: &'static [$name] = &[ $( $name::$variant $( ( $init ) )? ),+ ];

            /// Convert variant into "menu" label.
            pub fn label(&self) -> &'static str {
                match self {
                    $( $name::$variant { .. } => $label ),+
                }
            }

            /// Get index into choice_enum!::ALL.
            pub fn index(&self) -> usize {
                Self::ALL
                    .iter()
                    .position(|v| std::mem::discriminant(v) == std::mem::discriminant(self))
                    .unwrap_or(0)
            }

            /// Convert index into variant.
            pub fn from_index(index: usize) -> Self {
                Self::ALL.get(index).cloned().unwrap_or_default()
            }
        }

        /// Convert from string index in SpringSharp files.
        impl From<&str> for $name {
            fn from(index: &str) -> Self {
                index.parse::<usize>()
                    .ok()
                    .and_then(|i| Self::ALL.get(i).cloned())
                    .unwrap_or_default()
            }
        }

        /// Convert from string index in SpringSharp files.
        impl From<String> for $name {
            fn from(index: String) -> Self {
                index.as_str().into()
            }
        }

        /// Convert to string used in reports.
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}",
                    match self {
                        $( $name::$variant { .. } => $display ),+
                    })
            }
        }
    };
}

// num {{{2
/// Format a number with commas and the specified number of
/// significant digits, 0 by default.
///
// This is a macro instead of a function to avoid having to cast
// floats to ints or ints to floats
pub use format_num::format_num;
#[macro_export]
macro_rules! num {
    ($val:expr) => { num!($val, 0) };
    ($val:expr, $digits: expr) => {
        $crate::format_num!(&*format!(",.{}", $digits), $val)
    };
}

// pct {{{2
/// Treat a number as a percent and format a number with commas and the specified number of
/// significant digits, 0 by default. A trailing '%' is deliberately ommitted.
///
// This is a macro instead of a function to avoid having to cast
// floats to ints or ints to floats
#[macro_export]
macro_rules! pct {
    ($val:expr)                => { pct!($val, 0) };
    ($val:expr, $digits: expr) => { $crate::num!($val as f64 * 100.0, $digits)
    };
}

// addto {{{2
/// Pass arguments to format!() and push to a Vec<String>.
///
#[macro_export]
macro_rules! addto {
    ($r:ident,$($tts:tt)*) => {
        $r.push(format!($($tts)*))
    };
    ($r:ident) => {
        $r.push("".to_string())
    };
}

// addif {{{2
/// Return a formatted string if the condition is true.
/// Otherwise return an empty string.
///
#[macro_export]
macro_rules! addif {
    ($cond:expr, $($tts:tt)*) => {
        if $cond {
            format!($($tts)*)
        } else {
            "".into()
        }
    }
}

// Imports {{{1
//
pub mod calc;
pub use calc::*;

// Re-export the calc modules at the crate root so that paths like
// `crate::units::X` continue to work for external and internal callers.
pub use calc::armor;
pub use calc::engine;
pub use calc::hull;
pub use calc::hull_draw;
pub use calc::ship;
pub use calc::units;
pub use calc::weapons;
pub use calc::weights;
