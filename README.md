[![Rust](https://github.com/jbrubake/sharpie/actions/workflows/rust.yaml/badge.svg)](https://github.com/jbrubake/sharpie/actions/workflows/rust.yaml)

# Sharpie

A [springsharp](http://springsharp.com) remake.

When released **version 1** will be a clone of `Springsharp v3b3` with the
unimplemented features (such as *Engine Factor*) implemented.

New features will be added in **Version 2**.

# Usage

Build:

    cargo build

Load a ship FILE and print a report:

    cargo run -- load [FILE]

Convert a SpringSharp 3 file to `sharpie` format:

    cargo run -- convert [SpringSharp FILE] [OUTPUT FILE]

Tests:

    cargo test

