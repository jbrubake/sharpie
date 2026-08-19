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
// cli {{{2
#[cfg(test)]
mod cli {
    use super::*;
    use clap::CommandFactory;
    use clap::Parser;
    use sharpie::units::Measurement;
    use sharpie::units::UnitType::*;

    // Test verify_cli {{{3
    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    // Test parse_ok {{{3
    macro_rules! test_parse_ok {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (args, expected) = $value;

                    let cli = Cli::try_parse_from(args).expect("expected successful parse");
                    assert!(expected(cli));
                }
            )*
        }
    }

    test_parse_ok! {
        // name:             (args, verify)
        parse_load: (
            ["sharpie", "load", "ship.ship"],
            |cli: Cli| matches!(cli.command,
                Some(Commands::Load { ref file }) if file.as_str() == "ship.ship"),
        ),
        parse_convert: (
            ["sharpie", "convert", "ship.sship"],
            |cli: Cli| matches!(cli.command,
                Some(Commands::Convert { ref from, to: None, report: false })
                    if from.as_str() == "ship.sship"),
        ),
        parse_convert_to: (
            ["sharpie", "convert", "ship.sship", "--to", "out.ship"],
            |cli: Cli| matches!(cli.command,
                Some(Commands::Convert { ref from, to: Some(ref to), report: false })
                    if from.as_str() == "ship.sship" && to.as_str() == "out.ship"),
        ),
        parse_convert_to_short: (
            ["sharpie", "convert", "ship.sship", "-t", "out.ship"],
            |cli: Cli| matches!(cli.command,
                Some(Commands::Convert { to: Some(ref to), .. }) if to.as_str() == "out.ship"),
        ),
        parse_convert_report: (
            ["sharpie", "convert", "ship.sship", "--report"],
            |cli: Cli| matches!(cli.command,
                Some(Commands::Convert { ref from, report: true, .. })
                    if from.as_str() == "ship.sship"),
        ),
        parse_convert_report_short: (
            ["sharpie", "convert", "ship.sship", "-r"],
            |cli: Cli| matches!(cli.command, Some(Commands::Convert { report: true, .. })),
        ),
        parse_none: (
            ["sharpie"],
            |cli: Cli| cli.command.is_none(),
        ),
    }

    // Test parse_err {{{3
    macro_rules! test_parse_err {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let args = $value;

                    assert!(Cli::try_parse_from(args).is_err());
                }
            )*
        }
    }

    test_parse_err! {
        // name:                (args)
        parse_load_missing:    ["sharpie", "load"],
        parse_convert_missing: ["sharpie", "convert"],
        parse_bad_command:     ["sharpie", "bogus"],
    }

    // Test debug {{{3
    #[cfg(debug_assertions)]
    #[test]
    fn parse_debug() {
        let cli = Cli::try_parse_from(["sharpie", "--debug", "load", "ship.ship"])
            .expect("expected successful parse");
        assert!(cli.debug);
    }

    // Test run {{{3
    use sharpie::hull::{BowType, SternType};
    use sharpie::units::Units;

    static TMP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn temp_path(ext: &str) -> String {
        let n = TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::env::temp_dir()
            .join(format!("sharpie_test_{}_{}.{}", std::process::id(), n, ext))
            .to_string_lossy()
            .into_owned()
    }

    fn make_ship() -> sharpie::Ship {
        let mut ship = sharpie::Ship::default();

        ship.name = "Test Ship".into();
        ship.country = "Test Country".into();
        ship.kind = "Test Kind".into();
        ship.year = 1890;

        ship.hull.set_lwl(500.0, Units::Imperial);
        ship.hull.b  = Measurement::new(50.0, LengthLong, Units::Imperial);
        ship.hull.bb = Measurement::new(ship.hull.b.imp(), LengthLong, Units::Imperial);
        ship.hull.t  = Measurement::new(10.0, LengthLong, Units::Imperial);
        ship.hull.bow_angle = 0.0;
        ship.hull.stern_overhang = 0.0;

        ship.hull.fc_len = 0.20;
        ship.hull.fc_fwd = Measurement::new(10.0, LengthLong, Units::Imperial);
        ship.hull.fc_aft = Measurement::new(10.0, LengthLong, Units::Imperial);

        ship.hull.fd_len = 0.30;
        ship.hull.fd_fwd = ship.hull.fc_fwd;
        ship.hull.fd_aft = ship.hull.fc_fwd;

        ship.hull.ad_fwd = ship.hull.fc_fwd;
        ship.hull.ad_aft = ship.hull.fc_fwd;

        ship.hull.qd_len = 0.15;
        ship.hull.qd_fwd = ship.hull.fc_fwd;
        ship.hull.qd_aft = ship.hull.fc_fwd;

        ship.hull.bow_type = BowType::Normal;
        ship.hull.stern_type = SternType::Cruiser;

        ship
    }

    // Test run_load {{{3
    #[test]
    fn run_load() {
        let path = temp_path("ship");
        make_ship().save(path.clone()).unwrap();

        let cli = Cli::try_parse_from(["sharpie", "load", &path]).unwrap();
        assert!(run(cli).is_ok());

        std::fs::remove_file(path).unwrap();
    }

    // Test run_load_missing {{{3
    #[test]
    fn run_load_missing() {
        let path = temp_path("ship");

        let cli = Cli::try_parse_from(["sharpie", "load", &path]).unwrap();
        assert!(run(cli).is_err());
    }

    // Test run_convert {{{3
    #[test]
    fn run_convert() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/ships/orn.sship");

        let cli = Cli::try_parse_from(["sharpie", "convert", path]).unwrap();
        assert!(run(cli).is_ok());
    }

    // Test run_convert_to {{{3
    #[test]
    fn run_convert_to() {
        let from = concat!(env!("CARGO_MANIFEST_DIR"), "/ships/orn.sship");
        let to = temp_path("ship");

        let cli = Cli::try_parse_from(["sharpie", "convert", from, "--to", &to]).unwrap();
        assert!(run(cli).is_ok());

        assert!(sharpie::Ship::load(to.clone()).is_ok());

        std::fs::remove_file(to).unwrap();
    }

    // Test run_convert_bad {{{3
    #[test]
    fn run_convert_bad() {
        let from = temp_path("sship");

        let cli = Cli::try_parse_from(["sharpie", "convert", &from]).unwrap();
        assert!(run(cli).is_err());
    }
}
