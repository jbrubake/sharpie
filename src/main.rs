use clap::{Parser, Subcommand};
use rfd::FileDialog;
use sharpie::{Ship, SHIP_FILE_EXT, SS_SHIP_FILE_EXT};

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

    #[arg(short, long)]
    #[arg(help = "Show internal values")]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Load {
        file: String
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
/// Convert a Springsharp 3b3 file to sharpie format and show the ship report.
///
fn convert_ship(binding: MainWindow) {
    let file = FileDialog::new()
        .set_title("Springsharp file to convert")
        .add_filter(SS_SHIP_FILE_EXT, &[SS_SHIP_FILE_EXT,])
        .add_filter("all", &["*",])
        .pick_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    let ship = Ship::convert(file);

    match ship {
        Ok(ship) => {
            binding.set_report_str(ship.report().into());
            save_ship(ship);
        },

        // TODO: Show errors in the GUI
        Err(error) => eprintln!("{}", error),
    };
}

/// Load a sharpie ship file and show the ship report.
///
fn load_ship(binding: MainWindow) {
    let file = FileDialog::new()
        .set_title("Sharpie file to load")
        .add_filter(SHIP_FILE_EXT, &[SHIP_FILE_EXT,])
        .add_filter("all", &["*",])
        .pick_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    let ship = Ship::load(file);

    match ship {
        Ok(ship) => binding.set_report_str(ship.report().into()),
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
        .add_filter(SHIP_FILE_EXT, &[SHIP_FILE_EXT,])
        .add_filter("all", &["*",])
        .save_file()
        .unwrap_or_default()
        .into_os_string()
        .into_string()
        .unwrap();

    let _ = ship.save(file);
}

slint::include_modules!();

// Main {{{1
//
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut report_txt = None;
    let mut internals = None;

    let result = match cli.command {
        Some(Commands::Load { file }) => {
            match Ship::load(file) {
                Ok(ship) => {
                    report_txt = Some(ship.report());
                    if cli.debug {
                        internals = Some(ship.internals());
                    }
                    Ok(())
                },
                Err(err) => Err(err),
            }
        },

        Some(Commands::Convert { from, to, report }) => {
            match Ship::convert(from) {
                Ok(ship) => {
                    if report {
                        report_txt = Some(ship.report());
                    }
                    if cli.debug {
                        internals = Some(ship.internals());
                    }
                    match to {
                        Some(to) => ship.save(to),
                        None => Ok(()),
                    }
                },
                Err(err) => Err(err),
            }
        },

        None => {
            let ui = MainWindow::new().unwrap();

            ui.on_load_ship({ let handle = ui.as_weak(); move || { load_ship(handle.unwrap()); }});
            ui.on_convert_ship({ let handle = ui.as_weak(); move || { convert_ship(handle.unwrap()); }});

            let _ = ui.run();

            Ok(())
        }
    };

    match report_txt {
        Some(txt) => println!("{}", txt),
        None => (),
    }

    match internals {
        Some(txt) => eprintln!("{}", txt),
        None => (),
    }

    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            println!("{}", error);

            Err(error)
        },
    }
}

// Testing {{{1
//
#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}

