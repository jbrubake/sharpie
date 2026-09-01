//! Binary-side Slint GUI: callbacks and window lifecycle.
//!
//! This module is declared from `main.rs` only and lives entirely in the
//! binary crate, where the `slint::include_modules!()` types are available.

use rfd::FileDialog;
use crate::calc::{SHIP_FILE_EXT, SS_SHIP_FILE_EXT, Ship};
use slint::ComponentHandle;

use std::error::Error;

use crate::MainWindow;

// Load and Convert {{{1
//
// NOTE: The GUI functions in this section (convert_ship, load_ship, save_ship,
// and run_gui) are intentionally NOT unit-tested:
//
//  - convert_ship, load_ship, and save_ship call rfd::FileDialog
//    (pick_file()/save_file()), which has no mock/test hook in rfd 0.16 and
//    blocks on a real native dialog; a test cannot inject a fake path.
//  - They also take a slint MainWindow, and slint 1.14.1 ships no headless
//    test backend (no backend-testing/TestingBackend feature), so
//    MainWindow::new() needs a real display and fails on headless CI.
//  - run_gui is a blocking slint event loop (ui.run()).
//  - The logic after the dialog is already exercised by the Ship tests in
//    lib.rs (Ship::load/convert/save round-trips).
//
/// Convert a SpringSharp 3b3 file to sharpie format and show the ship report.
///
fn convert_ship(ui: MainWindow) {
    let file = FileDialog::new()
        .set_title("SpringSharp file to convert")
        .add_filter(SS_SHIP_FILE_EXT, &[SS_SHIP_FILE_EXT])
        .add_filter("all", &["*"])
        .pick_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    match Ship::convert(file) {
        Ok(ship) => {
            ui.set_report_str(ship.report().into());
            save_ship(ship);
        }

        // TODO: Show errors in the GUI
        Err(error) => eprintln!("{}", error),
    };
}

/// Load a sharpie ship file and show the ship report.
///
fn load_ship(ui: MainWindow) {
    let file = FileDialog::new()
        .set_title("Sharpie file to load")
        .add_filter(SHIP_FILE_EXT, &[SHIP_FILE_EXT])
        .add_filter("all", &["*"])
        .pick_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    match Ship::load(file) {
        Ok(ship) => ui.set_report_str(ship.report().into()),

        // TODO: Show errors in the GUI
        Err(error) => eprintln!("{}", error),
    };
}

/// Save a ship to a file.
///
fn save_ship(ship: Ship) {
    let file = FileDialog::new()
        .set_title("Sharpie file to save")
        .set_file_name("SHIP.".to_owned() + SHIP_FILE_EXT)
        .add_filter(SHIP_FILE_EXT, &[SHIP_FILE_EXT])
        .add_filter("all", &["*"])
        .save_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    match ship.save(file) {
        Ok(_) => (),
        // TODO: Show errors in the GUI
        Err(error) => eprintln!("{}", error),
    };
}

// Run the GUI {{{1
//
pub fn run_gui() -> Result<(), Box<dyn Error>> {
    let ui = MainWindow::new().unwrap();

    ui.on_load_ship   ({ let h = ui.as_weak(); move || { load_ship(h.unwrap()); }});
    ui.on_convert_ship({ let h = ui.as_weak(); move || { convert_ship(h.unwrap()); }});

    match ui.run() {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
