[![Rust](https://github.com/orionarts/sharpie/actions/workflows/test.yaml/badge.svg)](https://github.com/orionarts/sharpie/actions/workflows/test.yaml)

# Sharpie

A [SpringSharp](http://springsharp.com) remake.

**Version 1** is intended to be a bug-for-bug clone of `SpringSharp v3b3`
(excluding some difficult to reproduce bugs related to how `SpringSharp` stores
values). New features will be added in **Version 2**. Release history is kept
in [CHANGELOG.md](CHANGELOG.md).

# Usage

`sharpie` can convert `SpringSharp` files to its own format, load its own
`*.ship` files and generate reports for both. `sharpie` files can only be edited
by hand for now. Running `sharpie` without any arguments launches the GUI.

Load a ship FILE and print a report:

    sharpie load FILE

Write the hull side profile as an SVG while loading:

    sharpie load FILE --image              # writes <file stem>-hull.svg
    sharpie load FILE --image OUT.svg      # custom output name

Convert a `SpringSharp` file to `sharpie` format and save it:

    sharpie convert SPRINGSHARP_FILE --to OUTPUT_FILE

Convert a `SpringSharp` file to `sharpie` format without saving but show the
report:

    sharpie convert SPRINGSHARP_FILE --report

Convert a `SpringSharp` file to `sharpie` format and save it plus show the
report:

    sharpie convert SPRINGSHARP_FILE --to OUTPUT_FILE --report

Like `load`, `convert` accepts `--image [FILE]` to write the hull SVG.

# Building

`sharpie` is written in Rust. Build it with:

    cargo build --release

Run the test suite with:

    cargo test

A `Makefile` provides convenience targets: `make build`, `make gui` (build and
launch the GUI), `make preview` (preview the UI with `slint-viewer`),
`make live-preview`, `make docs` and `make view-docs`.

# Comparing Sharpie reports to SpringSharp reports

The report output by `sharpie` is supposed to be formatted exactly like a
`SpringSharp` report, except for differences in spacing. If you place the
`*.sship` file and a corresponding `*.report` file that contains the
`SpringSharp` report (i.e., `foo.sship` and `foo.report` in the same directory),
you can use the included `chkreport` script to compare `sharpie`'s report:

    ./chkreport path/to/SHIP.sship

This will use `vimdiff` if available.

Please file an [issue](https://github.com/orionarts/sharpie/issues/new/choose)
for any `sharpie` reports that differ from `SpringSharp`. Include both the
original `.sship` file and information on which lines are different.

Although the `sharpie` report is intended to be identical to the `SpringSharp`
report, small differences due to rounding or oddities in the way `SpringSharp`
outputs values can occur. These should still be reported although they may not
result in any changes.

