[![Rust](https://github.com/jbrubake/sharpie/actions/workflows/test.yaml/badge.svg)](https://github.com/jbrubake/sharpie/actions/workflows/test.yaml)

# Sharpie

A [springsharp](http://springsharp.com) remake.

**Version 1** is intended to be a bug-for-bug clone of `Springsharp v3b3`
(excluding some difficult to reproduce bugs related to how `Springsharp` stores
values). New features will be added in **Version 2**.

# Usage

Running `sharpie` without any arguments launches the GUI.

Load a ship FILE and print a report:

    sharpie load [FILE]

Convert a `SpringSharp` file to `sharpie` format:

    sharpie convert [SpringSharp FILE] --to [OUTPUT FILE]

Convert a `SpringSharp` file to `sharpie` format and print a report:

    sharpie convert [SpringSharp FILE] --to [OUTPUT FILE] --report

# Comparing Sharpie reports to Springsharp reports

The report output by `sharpie` is supposed to be formatted exactly like a
`Springsharp` report, except for differences in spacing. If you run both reports
through the following command you should be able to use `diff(1)` to easily spot
differences between the two reports:

    sed -e 's/\t\t*/ /g' -e 's/  */ /g' -e 's/^ *//' [REPORT] > [REPORT].nospaces

Please file an [issue](https://github.com/jbrubake/sharpie/issues/new/choose)
for any `sharpie` reports that differ from `Springsharp`. Include both the
original `.sship` file and information on which lines are different.

Although the `sharpie` report is intended to be identical to the `Springsharp`
report, small differences due to rounding or oddities in the way `Springsharp`
outputs values can occur. These should still be reported although they may not
result in any changes.

`sharpie` currently only supports Imperial measurements. It will display Metric
equivalents but loading a `Springsharp` file with Metric units will give
incorrect results as the numbers will be interpreted as Imperial.

