# Sharpie

A [springsharp](http://springsharp.com) remake.

When released **version 1** will be a clone of `Springsharp v3b3` with the
unimplemented features (such as *Engine Factor*) implemented.

New features will be added in **Version 2**.

# Usage

Build:

    cargo build

Load a ship FILE and print a report:

    cargo run -- --load [FILE]

Create a ship, print a report and save to FILE:

    # Edit src/main.rs to adjust the ship parameters
    cargo run [FILE]

Tests:

    cargo test

