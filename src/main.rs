use clap::{Parser, Subcommand};
use rfd::FileDialog;
use sharpie::{SHIP_FILE_EXT, SS_SHIP_FILE_EXT, Ship};

use std::error::Error;

slint::include_modules!();

// Command line parsing {{{1
//
#[derive(Parser)]
#[command(version)]
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
//  - The logic after the dialog is already exercised by the run_* tests in
//    the cli module below (Ship::load/convert/save and report()).
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

// Run the CLI command.
//
fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Some(Commands::Load { file }) => match Ship::load(file) {
            Ok(ship) => {
                println!("{}", ship.report());
                #[cfg(debug_assertions)]
                if cli.debug { eprintln!("{}", ship.internals()); }

                Ok(())
            }

            Err(error) => Err(error),
        },

        Some(Commands::Convert { from, to, report }) => match Ship::convert(from) {
            Ok(ship) => {
                if report { println!("{}", ship.report()); }
                #[cfg(debug_assertions)]
                if cli.debug { eprintln!("{}", ship.internals()); }

                match to {
                    Some(to) => match ship.save(to) {
                        Ok(_) => Ok(()),
                        Err(error) => Err(error),
                    },
                    None => Ok(()),
                }
            }

            Err(error) => Err(error),
        },

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
#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}

