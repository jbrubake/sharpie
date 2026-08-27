# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-08-27

### Added

- Add `--image` option to `convert` and `load` commands to generate an SVG of
    the hull

### Changed

- The weight of a barbette now matches SprinSharp's likely buggy behavior
- Added the missing "bulkhead beam too wide" warning to the output report

### Fixed

- Some constants were fixed to match the values in SpringSharp
- `Armor::ct_wgt()` adds both CT weights instead of just the forward weights
    twice
- When calculating `Hull::cwp()` a Transom stern takes precendence over >= 2
    engine shafts or Cb >= 0.75

## [0.3.1] - 2026-08-21

### Added

- Length parameter to `BowType::BulbForward`
- `YEAR_MIN`/`YEAR_MAX` constants bounding valid ship years
- `chkreport` script for diffing generated reports against SpringSharp output
- `GunDistributionType::None` gun distribution layout, now the default
- Crate metadata (`description`, `license`, `repository`, `rust-version`)

### Changed

- `d`/`cb` and `lwl`/`loa` pairs in `Hull` replaced with enums so each GUI widget
  maps 1:1 to a data state
- `BulkheadType::Additional` is now the default bulkhead type
- added `choice_enum!` macro to simplify enum construction for enums that define
    a list of user choices

### Fixed

- Crate dependency requirements (dropped unneeded `derive_builder`, pinned
  major/minor versions)

## [0.3.0] - 2026-08-20

### Added

- Metric and imperial unit support via a new `Measurement` type, now backing all
  hull dimensions, armor, batteries, torpedoes, mines, and derived calculations

### Fixed

- Report display bug where `fc_fwd` was assigned from `fc_len`
- Reports now match SpringSharp output more closely (strings, layout, always
  show Miscellaneous Weights)

## [0.2.0] - 2026-08-19

### Added

- `DeckType::BoxOverMachinery` and `DeckType::BoxOverBoth` deck types
- Substantially expanded test suite (CLI, units, armor weight sums, enum
  display/from-string conversions)

## [0.1.1] - 2026-05-10

### Fixed

- Allow saving a converted SpringSharp sship file to a non-existent Sharpie
  ship file
- Prevent errors when the ship's year is before 1860; provide a default year
  when none is given
- Provide a default block coefficient and handle neither `d` nor `cb` being set

## [0.1.0] - 2026-01-03

### Added

- CLI and GUI to generate reports from ship files and convert SpringSharp sship
  files to Sharpie format

[unreleased]: https://github.com/orionarts/sharpie/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/orionarts/sharpie/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/orionarts/sharpie/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/orionarts/sharpie/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/orionarts/sharpie/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/orionarts/sharpie/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/orionarts/sharpie/releases/tag/v0.1.0
