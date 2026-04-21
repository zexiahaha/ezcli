# Changelog

All notable changes to this project will be documented in this file.

The format is based on a simple versioned changelog, grouped by release and change type.

## [Unreleased]

- No unreleased changes yet.

## [0.1.2] - 2026-04-21

### Fixed

- Fully fixed the `cmd` / Cmder wrapper workflow for Chinese-path environments.
- Fixed temporary `.cmd` script generation issues caused by shell redirection and encoding mismatches.
- Fixed environment capture decoding so `TEMP`, `TMP`, `USERPROFILE`, `APPDATA`, and `LOCALAPPDATA` are no longer corrupted after `ecl`.
- Fixed wrapper lookup issues after loading the MSVC environment so `ecl` and `ep` remain available.

### Changed

- `cmd` wrappers now stay ASCII-only and no longer embed Chinese absolute paths.
- `emit load-cl` and `emit enter-project` now support `--output` so `ezcli` can write temporary scripts directly.
- Temporary `cmd` scripts are now written using the current `cmd` code page.
- `env_capture` now captures before/after environment snapshots in the same `cmd.exe` process and only applies diffs.

## [0.1.1] - 2026-04-20

### Fixed

- Fixed PowerShell wrapper and profile issues when paths contain Chinese characters.
- Fixed PowerShell environment variable rendering for names such as `CommonProgramFiles(x86)`.

### Changed

- PowerShell wrapper scripts and related profile writes now use UTF-8 with BOM.

## [0.1.0] - 2026-04-20

### Added

- Initial usable release of `ezcli`.
- Added a shell-wrapper based workflow so environment changes apply to the current shell session.
- Added `ecl` for loading the MSVC `cl` environment into the current shell.
- Added `ep <name>` for entering configured project directories quickly.
