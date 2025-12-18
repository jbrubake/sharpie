use clap::{Parser, Subcommand};
use sharpie::Ship;

use std::error::Error;

#[derive(Parser)]
#[command(version)]
#[command(about = "SpringSharp 3b3 clone", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    #[arg(help = "Show internal values")]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Load { file: String },
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut to_file = String::new();
    let mut show_report = true;

    let result = match cli.command {
        Commands::Load { file } => Ship::load(file),
        Commands::Convert { from, to, report } => {
            to_file = match to { Some(to) => to, _ => "".into(), };
            show_report = report;
            Ship::convert(from)
        },
    };

    match result {
        Ok(ship) => {
            if to_file != "" {
                let _ = ship.save(to_file);
            }

            if show_report { 
                ship.report();
            }

            if cli.debug {
                eprintln!("");
                eprintln!("Internal values");
                eprintln!("---------------");
                ship.internals();
            }

            Ok(())
        },
        Err(error) => {
            println!("{}", error);

            Err(error)
        },
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}

