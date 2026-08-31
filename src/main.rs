use clap::Parser;

use std::error::Error;

slint::include_modules!();

mod cli;
mod gui;

fn main() -> Result<(), Box<dyn Error>> {
    cli::run(cli::Cli::parse())
}
