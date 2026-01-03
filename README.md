[![Rust](https://github.com/jbrubake/sharpie/actions/workflows/test.yaml/badge.svg)](https://github.com/jbrubake/sharpie/actions/workflows/test.yaml)

# Sharpie

A [springsharp](http://springsharp.com) remake.

**Version 1** is intended to be a bug-for-bug clone of `Springsharp v3b3`
(excluding some difficult to reproduce bugs related to how `Springsharp` stores
values). New features will be added in **Version 2**.

# Usage

`sharpie` can convert `Springsharp` files to its own format, load its own
`*.ship` files and generate reports for both. `sharpie` files can only be edited
by hand for now. Running `sharpie` without any arguments launches the GUI.

Load a ship FILE and print a report:

    sharpie load [FILE]

Convert a `SpringSharp` file to `sharpie` format:

    sharpie convert [SpringSharp FILE] --to [OUTPUT FILE]

Convert a `SpringSharp` file to `sharpie` format and print a report:

    sharpie convert [SpringSharp FILE] --to [OUTPUT FILE] --report

# Missing Functionality

- Metric units are not suppored in either `sharpie` or `Springsharp` files.
  Files using them will load but all values are interpreted as Impreial so
  they will not work properly.
- **Box over Machinery** and **Box over Machinery & Magazines** decks types
  are not fully implemented and will generate values different than
  `Springsharp`.

# Comparing Sharpie reports to Springsharp reports

The report output by `sharpie` is supposed to be formatted exactly like a
`Springsharp` report, except for differences in spacing. If you run both reports
through the following command you should be able to use `diff(1)` to easily spot
differences between the two reports:

    sed -e 's/\t/ /g' -e 's/  */ /g' -e 's/^ *//' -e 's/ *$// [REPORT] > [REPORT].nospaces

Please file an [issue](https://github.com/jbrubake/sharpie/issues/new/choose)
for any `sharpie` reports that differ from `Springsharp`. Include both the
original `.sship` file and information on which lines are different.

Although the `sharpie` report is intended to be identical to the `Springsharp`
report, small differences due to rounding or oddities in the way `Springsharp`
outputs values can occur. These should still be reported although they may not
result in any changes.

