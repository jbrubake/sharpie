// Hull drawing {{{1
/// Render a hull side profile as an SVG image.
///
/// This is a port of `FreeboardControl.drawFreeboard` from SpringSharp
/// 3b3 (SpringSharp3b3.cs lines 1954-2252).  The canvas is 494x156, the
/// same size the original program uses when saving the picture, and all
/// geometry is scaled from feet to pixels exactly as the original
/// `feet2pixels` does: pixels = feet / (LOA / 458).
///
/// The WinForms focus-dependent dimension arrows and hint text are not
/// ported; neither is the designer fallback for zero-length hulls.
//
use std::fmt::Write as _;

use crate::hull::{BowType, Hull, SternType};
use crate::units::Units;

// Constants {{{1
/// Canvas width in pixels.
const CANVAS_W: i32 = 494;
/// Canvas height in pixels.
const CANVAS_H: i32 = 156;
/// Reference length the original scales from: LOA of 458 ft fills the canvas.
const REF_LOA: f64 = 458.0;
/// Y coordinate of the waterline.
const WATERLINE: i32 = 100;
/// X coordinate where the hull starts when there is no stern overhang.
const X_MARGIN: i32 = 20;

// Colors
const WHITE:      &str = "white";
const FIREBRICK:  &str = "#b22222";
const STEEL_BLUE: &str = "#4682b4";
const LIGHT_GRAY: &str = "#d3d3d3";
const DARK_GRAY:  &str = "#a9a9a9";

// Point {{{1
type Pt = (i32, i32);

fn pts_str(pts: &[Pt]) -> String {
    pts.iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ")
}

// Profile {{{1
/// Geometry of one hull profile, in canvas pixels.
struct Profile {
    /// Above-water outline, 11 points, closed.
    deck: [Pt; 11],
    /// Underwater outline, 7 points, closed.
    bottom: [Pt; 7],
    /// X of the quarterdeck forward boundary (dashed line).
    qd_x: i32,
    /// X of the forecastle aft boundary (dashed line).
    fc_x: i32,
    /// X of the bow at the waterline (scale bar anchor).
    bow_x: i32,
}

// profile {{{1
/// Compute the hull profile geometry for a hull.
///
/// Returns `None` when the hull has no length, mirroring the original's
/// behaviour of drawing nothing but the frame in that case.
///
fn profile(hull: &Hull) -> Option<Profile> {
    let loa = hull.loa().imp();
    if loa <= 0.0 { return None; }

    // feet2pixels: pixels = feet / (loa / 458), truncated toward zero.
    let px = |feet: f64| (feet / (loa / REF_LOA)) as i32;

    let lwl            = hull.lwl().imp();
    let t              = hull.t.imp();
    let stern_overhang = hull.stern_overhang.imp();
    let ram            = hull.bow_type.ram_len().imp();

    let fc_len = hull.fc_len;
    let fc_fwd = hull.fc_fwd.imp();
    let fc_aft = hull.fc_aft.imp();

    let fd_len = hull.fd_len;
    let fd_fwd = hull.fd_fwd.imp();
    let fd_aft = hull.fd_aft.imp();

    let ad_fwd = hull.ad_fwd.imp();
    let ad_aft = hull.ad_aft.imp();

    let qd_len = hull.qd_len;
    let qd_fwd = hull.qd_fwd.imp();
    let qd_aft = hull.qd_aft.imp();

    // Above-water outline:
    //   0:    stern top
    //   1:    bow at waterline
    //   2:    stem head
    //   3:    forecastle aft
    //   4:    foredeck fwd
    //   5:    foredeck aft
    //   6:    aftdeck fwd
    //   7:    aftdeck aft
    //   8:    quarterdeck fwd
    //   9-10: stern closure
    let x0 = X_MARGIN + if stern_overhang > 0.0 { px(stern_overhang) } else { 0 };

    let mut deck = [(0, 0); 11];
    deck[0] = (x0, WATERLINE);
    deck[1] = (x0 + px(lwl), WATERLINE);

    // Stem head position depends on the sign of the bow rake.
    let tan_bow = hull.bow_angle.to_radians().tan();
    deck[2].0 = if hull.bow_angle < 0.0 {
        deck[1].0 + px((lwl * (1.0 - fc_len) - lwl).max(fc_fwd * tan_bow))
    } else {
        deck[1].0 + px(fc_fwd * tan_bow)
    };
    deck[2].1 = deck[1].1 - px(fc_fwd);

    deck[3] = (x0 + px(lwl * (1.0 - fc_len)), WATERLINE - px(fc_aft));
    deck[4] = (deck[3].0, WATERLINE - px(fd_fwd));
    deck[5] = (
        x0 + px(lwl * (1.0 - fd_len - fc_len)),
        WATERLINE - px(fd_aft),
    );
    deck[6] = (deck[5].0, WATERLINE - px(ad_fwd));
    deck[7] = (x0 + px(lwl * qd_len), WATERLINE - px(ad_aft));
    deck[8] = (deck[7].0, WATERLINE - px(qd_fwd));

    match hull.stern_type {
        SternType::Cruiser if stern_overhang > 0.0 => {
            deck[9] = (x0, WATERLINE - px(qd_aft));
            deck[10] = (x0 - px(stern_overhang), WATERLINE - px(qd_aft * 0.33));
        }
        _ => {
            deck[9] = (x0 - px(stern_overhang), WATERLINE - px(qd_aft));
            deck[10] = deck[9];
        }
    }

    // Underwater outline:
    //   0:   stern top
    //   1:   bow at waterline
    //   2-4: bow underwater variants
    //   5:   keel fwd
    //   6:   keel aft/stern bottom
    let mut bottom = [deck[0], deck[1], (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)];

    match hull.bow_type {
        // Overhanging stem: the underwater bow starts aft of the waterline.
        BowType::Normal if hull.bow_angle > 0.0 => {
            let cut = px((lwl * 0.85).min(t * tan_bow));
            bottom[2] = (deck[1].0 - cut, deck[1].1 + px(t));
            bottom[3] = bottom[2];
            bottom[4] = bottom[2];
        }
        BowType::Normal | BowType::BulbStraight => {
            bottom[2] = (deck[1].0, deck[1].1 + px(t));
            bottom[3] = bottom[2];
            bottom[4] = bottom[2];
        }
        BowType::Ram(_) => {
            bottom[2] = (deck[1].0, deck[1].1);
            bottom[3] = (deck[1].0 + px(ram), deck[1].1 + px(t * 0.667));
            bottom[4] = (deck[1].0, deck[1].1 + px(t));
        }
        BowType::BulbForward(_) => {
            bottom[2] = (deck[1].0, deck[1].1 + px(t * 0.333));
            bottom[3] = (deck[1].0 + px(ram), deck[1].1 + px(t * 0.667));
            bottom[4] = (deck[1].0, deck[1].1 + px(t));
        }
    }

    bottom[5] = (x0 + px(lwl * 0.15), bottom[4].1);

    bottom[6].1 = match hull.stern_type {
        SternType::Cruiser | SternType::TransomSm => WATERLINE + px(t * 0.33),
        SternType::TransomLg | SternType::Round => WATERLINE + px(t * 0.5),
    };
    bottom[6].0 = match hull.stern_type {
        SternType::Cruiser => x0 + px(stern_overhang).abs(),
        _ => x0 + px(stern_overhang).max(0),
    };

    Some(Profile {
        qd_x: deck[7].0,
        fc_x: deck[3].0,
        bow_x: deck[1].0,
        deck,
        bottom,
    })
}

// escape_xml {{{1
/// Escape text for inclusion in XML content.
///
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// text {{{1
/// Format a `<text>` element.
///
/// `x`, `y_top` are the top-left corner as in GDI `DrawString`; the SVG
/// baseline is derived from the font size.  Newlines become `<tspan>`s.
///
fn text(x: i32, y_top: i32, pt: f64, body: &str, anchor: Option<&str>) -> String {
    let px_size = pt * 4.0 / 3.0;
    let line_h = (px_size * 1.15).round() as i32;
    let baseline = y_top + (px_size * 0.8).round() as i32;
    let anchor_attr = anchor.map(|a| format!(" text-anchor=\"{a}\"")).unwrap_or_default();

    let mut lines = body.split('\n');
    let first = lines.next().unwrap_or_default();
    let mut s = format!(
        "  <text x=\"{x}\" y=\"{baseline}\" font-family=\"Tahoma, sans-serif\" \
         font-size=\"{px_size:.1}px\" fill=\"{WHITE}\"{anchor_attr}>"
    );
    let _ = write!(s, "{first}");
    for line in lines {
        let _ = write!(s, "<tspan x=\"{x}\" dy=\"{line_h}\">{line}</tspan>");
    }
    let _ = writeln!(s, "</text>");
    s
}

// hull_svg {{{1
/// Render the side profile of `hull` as a standalone SVG document.
///
/// `name` is drawn on the picture, as the original does with the ship
/// name box contents.
///
pub fn hull_svg(hull: &Hull, name: &str) -> String {
    let mut svg = String::with_capacity(4096);

    // Header, gradient and background.
    //
    // The original fills the hull with a PathGradientBrush built from an
    // ellipse at (-20,-40) sized 540x190, light grey fading to dark grey.
    // That maps directly onto a unit-radius radialGradient transformed
    // into an ellipse.
    let _ = writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {CANVAS_W} {CANVAS_H}\" \
         width=\"{CANVAS_W}\" height=\"{CANVAS_H}\">"
    );
    let _ = write!(
        svg,
        "  <defs>\n    <radialGradient id=\"hullGrad\" gradientUnits=\"userSpaceOnUse\" \
         cx=\"0\" cy=\"0\" r=\"1\" gradientTransform=\"translate(250,55) scale(270,95)\">\n\
         \x20     <stop offset=\"0\" stop-color=\"{LIGHT_GRAY}\"/>\n\
         \x20     <stop offset=\"1\" stop-color=\"{DARK_GRAY}\"/>\n\
         \x20   </radialGradient>\n  </defs>\n"
    );
    let _ = writeln!(
        svg,
        "  <rect x=\"0\" y=\"0\" width=\"{CANVAS_W}\" height=\"{CANVAS_H}\" fill=\"{STEEL_BLUE}\"/>"
    );

    if let Some(p) = profile(hull) {
        // Dashed deck boundary lines, drawn before the hull so the
        // polygons cover them, as in the original.
        for x in [p.qd_x, p.fc_x] {
            let _ = writeln!(
                svg,
                "  <line x1=\"{x}\" y1=\"50\" x2=\"{x}\" y2=\"140\" stroke=\"{WHITE}\" stroke-dasharray=\"4 4\"/>"
            );
        }

        // Hull and underwater polygons.
        let _ = writeln!(
            svg,
            "  <polygon points=\"{}\" fill=\"url(#hullGrad)\" stroke=\"{WHITE}\" stroke-width=\"1\"/>",
            pts_str(&p.deck)
        );
        let _ = writeln!(
            svg,
            "  <polygon points=\"{}\" fill=\"{FIREBRICK}\" stroke=\"{WHITE}\" stroke-width=\"1\"/>",
            pts_str(&p.bottom)
        );

        // Scale bar: 50 feet or 10 metres depending on units.
        let (bar_ft, label) = match hull.units {
            Units::Imperial => (50.0, "50 feet"),
            Units::Metric => (32.8084, "10 metres"),
        };
        let bar_start = p.bow_x - (bar_ft / (hull.loa().imp() / REF_LOA)) as i32;
        let _ = writeln!(
            svg,
            "  <line x1=\"{}\" y1=\"23\" x2=\"{bar_start}\" y2=\"23\" stroke=\"{WHITE}\"/>",
            p.bow_x
        );
        let _ = write!(
            svg,
            "{}",
            text(bar_start - 2, 11, 7.0, label, Some("end"))
        );

        // Vitalspace marker and caption.
        let _ = writeln!(
            svg,
            "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"none\" stroke=\"{WHITE}\"/>",
            p.deck[5].0, p.deck[0].1
        );
        let _ = write!(
            svg,
            "{}",
            text(p.deck[7].0 + 5, 130, 8.0, "Vitalspace (machinery, magazines, main guns)", None)
        );

        // Deck labels.
        let _ = write!(svg, "{}", text(11, p.deck[8].1 - 30, 8.0, "Quarter\ndeck", None));
        let _ = write!(svg, "{}", text(430, p.deck[2].1 - 18, 8.0, "Forecastle", None));
        let _ = write!(
            svg,
            "{}",
            text(p.deck[5].0, p.deck[5].1 - 30, 8.0, "Forward\ndeck", None)
        );
        let _ = write!(
            svg,
            "{}",
            text(p.deck[6].0 - 40, p.deck[6].1 - 30, 8.0, "Aft\ndeck", None)
        );
    }

    // Ship name and frame, always drawn.
    let _ = write!(svg, "{}", text(12, 12, 10.0, &escape_xml(name), None));
    let _ = writeln!(
        svg,
        "  <rect x=\"10\" y=\"10\" width=\"474\" height=\"134\" fill=\"none\" stroke=\"{WHITE}\"/>"
    );
    let _ = writeln!(
        svg,
        "  <rect x=\"448\" y=\"135\" width=\"36\" height=\"9\" fill=\"none\" stroke=\"{WHITE}\"/>"
    );
    let _ = writeln!(svg, "</svg>");

    svg
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use crate::calc::test_support::*;
    use super::*;

    // Fixed test hull {{{2
    /// Build a hull with hand-checkable round numbers.
    ///
    /// LWL = 400 ft, stern overhang 40 ft so LOA = 440 ft (sharpie adds
    /// the overhang to LOA), draft 20 ft.
    ///
    fn flat_hull() -> Hull {
        let m = crate::units::Measurement::new;
        let u = crate::units::Units::Imperial;
        use crate::units::UnitType::LengthLong;

        let mut h = Hull::default();
        h.set_lwl(400.0, u);
        h.t = m(20.0, LengthLong, u);
        h.bow_type = BowType::Normal;
        h.bow_angle = 0.0;
        h.stern_type = SternType::Cruiser;
        h.stern_overhang = m(40.0, LengthLong, u);

        h.fc_len = 0.25;
        h.fc_fwd = m(16.0, LengthLong, u);
        h.fc_aft = m(12.0, LengthLong, u);

        h.fd_len = 0.25;
        h.fd_fwd = m(10.0, LengthLong, u);
        h.fd_aft = m(8.0, LengthLong, u);

        h.ad_fwd = m(8.0, LengthLong, u);
        h.ad_aft = m(8.0, LengthLong, u);

        h.qd_len = 0.15;
        h.qd_fwd = m(8.0, LengthLong, u);
        h.qd_aft = m(8.0, LengthLong, u);

        h
    }

    // Test profile deck points {{{2
    #[test]
    fn profile_deck_points() {
        let p = profile(&flat_hull()).unwrap();

        assert_eq!(p.deck[0], (61, 100)); // stern top: 20 + px(40)
        assert_eq!(p.deck[1], (477, 100)); // bow: + px(400)
        assert_eq!(p.deck[2], (477, 84)); // level stem, fc fwd 16 ft
        assert_eq!(p.deck[3], (373, 88)); // fc aft end, 12 ft up
        assert_eq!(p.deck[4], (373, 90)); // foredeck fwd, 10 ft up
        assert_eq!(p.deck[5], (269, 92)); // foredeck aft, 8 ft up
        assert_eq!(p.deck[6], (269, 92)); // aftdeck fwd
        assert_eq!(p.deck[7], (123, 92)); // aftdeck aft
        assert_eq!(p.deck[8], (123, 92)); // quarterdeck fwd
        assert_eq!(p.deck[9], (61, 92)); // cruiser stern closure
        assert_eq!(p.deck[10], (20, 98)); // overhang tip at 1/3 qd height
    }

    // Test profile bottom points {{{2
    #[test]
    fn profile_bottom_points() {
        let p = profile(&flat_hull()).unwrap();

        assert_eq!(p.bottom[0], (61, 100));
        assert_eq!(p.bottom[1], (477, 100));
        assert_eq!(p.bottom[2], (477, 120)); // vertical stem, draft 20 ft
        assert_eq!(p.bottom[4], (477, 120));
        assert_eq!(p.bottom[5], (123, 120)); // keel fwd at 15% length
        assert_eq!(p.bottom[6], (102, 106)); // cruiser stern, 1/3 draft
    }

    // Test raked bow {{{2
    #[test]
    fn profile_raked_bow() {
        let mut h = flat_hull();
        h.bow_angle = 30.0;

        let p = profile(&h).unwrap();

        // LOA grows to 400 + 16*tan(30) + 40 = 449.24 ft.
        // Underwater stem cut back by min(85% lwl, t*tan(30)) = 11.55 ft.
        assert_eq!(p.bottom[2].0, 456);
        // Stem head leans forward by fc_fwd*tan(30) = 9.24 ft.
        assert_eq!(p.deck[2].0, 476);
    }

    // Test ram bow {{{2
    #[test]
    fn profile_ram_bow() {
        let m = crate::units::Measurement::new;
        let mut h = flat_hull();
        h.bow_type = BowType::Ram(m(
            10.0,
            crate::units::UnitType::LengthLong,
            crate::units::Units::Imperial,
        ));

        let p = profile(&h).unwrap();

        // LOA grows to 400 + 10 + 40 = 450 ft.
        assert_eq!(p.bottom[2], (467, 100)); // ram hinges at the waterline
        assert_eq!(p.bottom[3], (477, 113)); // ram tip at 2/3 draft
        assert_eq!(p.bottom[4], (467, 120));
    }

    // Test bulbous forward bow {{{2
    #[test]
    fn profile_bulb_forward_bow() {
        let m = crate::units::Measurement::new;
        let mut h = flat_hull();
        h.bow_type = BowType::BulbForward(m(
            10.0,
            crate::units::UnitType::LengthLong,
            crate::units::Units::Imperial,
        ));

        let p = profile(&h).unwrap();

        // LOA grows to 400 + 10 + 40 = 450 ft.
        assert_eq!(p.bottom[2], (467, 106)); // bulb starts at 1/3 draft
        assert_eq!(p.bottom[3], (477, 113)); // bulb tip at 2/3 draft
        assert_eq!(p.bottom[4], (467, 120));
    }

    // Test transom stern {{{2
    #[test]
    fn profile_transom_stern() {
        let mut h = flat_hull();
        h.stern_type = SternType::TransomLg;

        let p = profile(&h).unwrap();

        // No curved overhang point; stern closes vertically.
        assert_eq!(p.deck[9], (20, 92));
        assert_eq!(p.deck[10], (20, 92));
        // Large transom runs the bottom out at half draft.
        assert_eq!(p.bottom[6], (102, 110));
    }

    // Test zero-length hull {{{2
    #[test]
    fn profile_zero_length_is_none() {
        assert!(profile(&Hull::default()).is_none());
    }

    // Test SVG output {{{2
    #[test]
    fn svg_contains_expected_parts() {
        let svg = hull_svg(&flat_hull(), "A&T <Test>");

        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("radialGradient"));
        assert!(svg.contains("#4682b4"));
        assert!(svg.contains("points=\"61,100 477,100 477,84 373,88 373,90 269,92 269,92 123,92 123,92 61,92 20,98\""));
        assert!(svg.contains("points=\"61,100 477,100 477,120 477,120 477,120 123,120 102,106\""));
        assert!(svg.contains("#b22222"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains(">50 feet<"));
        assert!(svg.contains("A&amp;T &lt;Test&gt;"));
        assert!(svg.ends_with("</svg>\n"));
    }

    // Test metric scale bar {{{2
    #[test]
    fn svg_metric_scale_bar() {
        let mut h = flat_hull();
        h.units = crate::units::Units::Metric;

        let svg = hull_svg(&h, "x");

        assert!(svg.contains(">10 metres<"));
    }

    // Test empty hull still renders a frame {{{2
    #[test]
    fn svg_zero_length_renders_frame_only() {
        let svg = hull_svg(&Hull::default(), "empty");

        assert!(!svg.contains("<polygon"));
        assert!(svg.contains(">empty<"));
    }

    // Test test_ship renders {{{2
    #[test]
    fn svg_test_ship_smoke() {
        let ship = test_ship();
        let svg = hull_svg(&ship.hull, &ship.name);

        assert!(svg.matches("<polygon").count() == 2);
    }
}
