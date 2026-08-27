use clap::{Parser, Subcommand};
use rfd::FileDialog;
use sharpie::{hull_draw, SHIP_FILE_EXT, SS_SHIP_FILE_EXT, Ship};

use std::error::Error;

slint::include_modules!();

// Command line parsing {{{1
//
#[derive(Parser)]
#[command(version = concat!(
    env!("CARGO_PKG_VERSION"), "\n",
    "Copyright (C) 2024 Jeremy Brubaker\n",
    "License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\n",
    "This is free software: you are free to change and redistribute it.\n",
    "There is NO WARRANTY, to the extent permitted by law.\n",
    "\n",
    "Written by Jeremy Brubaker.\n",
))]
#[command(about = "SpringSharp 3b3 clone", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[cfg(debug_assertions)]
    #[arg(short, long)]
    #[arg(help = "Show internal values")]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Load {
        file: String,

        #[arg(short, long, num_args = 0..=1)]
        #[arg(help = "Write hull profile image (default name: <file stem>-hull.svg)")]
        image: Option<Option<String>>,
    },

    Convert {
        #[arg(help = "SpringSharp 3 file to convert")]
        from: String,

        #[arg(short, long)]
        #[arg(help = "Filename to save conversion to")]
        to: Option<String>,

        #[arg(short, long)]
        #[arg(help = "Show ship report after conversion")]
        report: bool,

        #[arg(short, long, num_args = 0..=1)]
        #[arg(help = "Write hull profile image (default name: <file stem>-hull.svg)")]
        image: Option<Option<String>>,
    },
}

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
fn run_gui() -> Result<(), Box<dyn Error>> {
    let ui = MainWindow::new().unwrap();

    ui.on_load_ship   ({ let h = ui.as_weak(); move || { load_ship(h.unwrap()); }});
    ui.on_convert_ship({ let h = ui.as_weak(); move || { convert_ship(h.unwrap()); }});

    match ui.run() {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

// Run the CLI {{{1
//
/// Derive the hull image filename from an input filename.
///
/// An explicit output name wins; otherwise use the input file's stem plus
/// "-hull.svg".
///
fn image_path(file: &str, out: Option<String>) -> String {
    out.unwrap_or_else(|| {
        let path = std::path::Path::new(file);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("hull");
        format!("{stem}-hull.svg")
    })
}

/// Write the hull side-profile SVG of a ship.
///
fn write_image(ship: &Ship, path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::write(path, hull_draw::hull_svg(&ship.hull, &ship.name))?;
    println!("wrote {path}");

    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Some(Commands::Load { file, image }) => {
            // Compute the image name before moving the input filename.
            //
            let image = image.map(|out| image_path(&file, out));

            match Ship::load(file) {
                Ok(ship) => {
                    println!("{}", ship.report());
                    #[cfg(debug_assertions)]
                    if cli.debug { eprintln!("{}", ship.internals()); }

                    match image {
                        Some(path) => write_image(&ship, &path),
                        None       => Ok(()),
                    }
                }

                Err(error) => Err(error),
            }
        }

        Some(Commands::Convert { from, to, report, image }) => {
            // Compute the image name before moving the input filename.
            //
            let image = image.map(|out| image_path(&from, out));

            match Ship::convert(from) {
                Ok(ship) => {
                    if report { println!("{}", ship.report()); }
                    #[cfg(debug_assertions)]
                    if cli.debug { eprintln!("{}", ship.internals()); }

                    if let Some(to) = to { ship.save(to)?; }

                    if let Some(path) = image { write_image(&ship, &path)?; }

                    Ok(())
                }

                Err(error) => Err(error),
            }
        }

        // No subcommand means launch the GUI
        None => run_gui(),
    }
}

// Main {{{1
//
fn main() -> Result<(), Box<dyn Error>> {
    run(Cli::parse())
}

// Testing {{{1
//
// cli {{{2
#[cfg(test)]
mod cli {
    use super::*;
    use clap::Parser;

    // assert_matches! {{{3
    //
    // assert_matches! would require "nightly"
    //
    // Local replacement for std::assert_matches (unstable as of Rust 1.93).
    macro_rules! assert_matches {
        ($expression:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
            match $expression {
                $( $pattern )|+ $( if $guard )? => {}
                _ => panic!(
                    "assertion failed: value did not match pattern `{}`",
                    stringify!($( $pattern )|+ $( if $guard )?)
                ),
            }
        };
    }

    // Test cli_parse_ok {{{3
    macro_rules! test_cli_parse_ok {
        ($($name:ident: ($args:expr, $($pattern:tt)+),)*) => {
            $(
                #[test]
                fn $name() {
                    let cli = Cli::try_parse_from($args).unwrap();
                    assert_matches!(cli.command, $($pattern)+);
                }
            )*
        }
    }

    test_cli_parse_ok! {
        // name: (args, pattern)
        cli_no_subcommand:
            (["sharpie"],
             None),
        cli_load:
            (["sharpie", "load", "ship.ship"],
             Some(Commands::Load { ref file, image: None }) if file == "ship.ship"),
        cli_load_image_bare:
            (["sharpie", "load", "ship.ship", "--image"],
             Some(Commands::Load { image: Some(None), .. })),
        cli_load_image_value:
            (["sharpie", "load", "ship.ship", "-i", "out.svg"],
             Some(Commands::Load { ref file, image: Some(ref image) })
                 if file == "ship.ship" && *image == Some("out.svg".to_owned())),
        cli_convert_minimal:
            (["sharpie", "convert", "in.sship"],
             Some(Commands::Convert { ref from, to: None, report: false, image: None })
                 if from == "in.sship"),
        cli_convert_to_long:
            (["sharpie", "convert", "in.sship", "--to", "out.ship"],
             Some(Commands::Convert { ref from, to: Some(ref to), report: false, image: None })
                 if from == "in.sship" && to == "out.ship"),
        cli_convert_to_short:
            (["sharpie", "convert", "in.sship", "-t", "out.ship"],
             Some(Commands::Convert { to: Some(ref to), .. }) if to == "out.ship"),
        cli_convert_report_long:
            (["sharpie", "convert", "in.sship", "--report"],
            Some(Commands::Convert { ref from, report: true, .. }) if from == "in.sship"),
        cli_convert_report_short:
            (["sharpie", "convert", "in.sship", "-r"],
            Some(Commands::Convert { report: true, .. })),
        cli_convert_image_bare:
            (["sharpie", "convert", "in.sship", "--image"],
             Some(Commands::Convert { image: Some(None), .. })),
        cli_convert_image_value:
            (["sharpie", "convert", "in.sship", "-i", "out.svg"],
             Some(Commands::Convert { image: Some(ref image), .. })
                 if *image == Some("out.svg".to_owned())),
    }

    // Test cli_parse_err {{{3
    macro_rules! test_cli_parse_err {
        ($($name:ident: ($args:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    assert!(Cli::try_parse_from($args).is_err());
                }
            )*
        }
    }

    test_cli_parse_err! {
        // name:              (args)
        cli_err_load:        (["sharpie", "load"]),
        cli_err_convert:     (["sharpie", "convert"]),
        cli_err_bogus:       (["sharpie", "bogus"]),
    }

    // Test cli_debug {{{3
    #[cfg(debug_assertions)]
    #[test]
    fn cli_debug() {
        let cli = Cli::try_parse_from(["sharpie", "--debug", "load", "ship.ship"]).unwrap();
        assert!(cli.debug);
    }
}
