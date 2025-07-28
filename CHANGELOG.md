# Changelog

## [0.1.0]
    initial release

## [0.2.0]
    add feature:
        `frate registry` displays all registered tools
    add feature:
        colored cli outputs




## [0.4.0] - 2025-07-28

### Changed
- Simplified the `run` subcommand by replacing the `name` and `args` fields with a single `command` field.
    - Example: `frate run "ls -la"` instead of `frate run ls -- -la`
- Improved CLI feedback with clearer status messages during key actions.
- Enhanced shim creation logic: better detection and naming of binaries (e.g. `rg` instead of `ripgrep`).
- Updated CLI test suite to match the new `run` command format.

### Fixed
- Fixed an issue where `search` would show no version info if no compatible platform versions were found – now always prints name and description.
- Fixed an issue in `search` where the first instead of the last filtered version was used in output.

### Documentation
- Updated README to reflect the new `frate run` syntax and command structure.

---