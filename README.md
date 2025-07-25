[![CI](https://github.com/konni332/frate/actions/workflows/ci.yml/badge.svg)](https://github.com/konni332/frate/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/konni332/frate/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/frate)](https://crates.io/crates/frate)

# frate
**frate** is a developer tool and package manager designed for managing local software installations
without requiring root privileges.
Inspired by Cargo’s user experience, it provides a clean and simple CLI with commands like init, 
add, list, and sync to manage dependencies declared in a frate.toml manifest and locked with a frate.lock file.

Unlike traditional package managers, frate focuses on user-level installations,
making it ideal for environments where users don’t have administrative rights.
It aims to streamline dependency management with a straightforward,
Cargo-like workflow, enabling developers to easily install, update,
and manage tools in a reproducible and isolated manner.

**frate** also includes command line sugar to maximize developer comfort. For example ``frate shell``,
to create a new shell session with all local tools in the ``PATH``


---

## Features

- Local User-Level package management
  - install and manage tools without root or admin privileges

- Cargo-Like UX - intuitive commands like:
  - ``init``
  - ``add``
  - ``sync``
  - ``list``
  - ``shell``

- Isolated, reproducible environments
- Declarative dependencies using ``frate.toml``
- Locking using ``frate.lock``
- Hash validation for security
- GitHub registry using JSON (local/custom registries planned) - see [frate-registry](https://github.com/konni332/frate-registry)
for the registry and the auto-gen tool


---

## Installation

#### Using cargo

````shell
cargo install frate
````

#### Build your self

````shell
git clone https://github.com/konni332/frate frate
cd frate
cargo build --release
````

#### Test with:

````shell
frate --version
````

---

## Quickstart

- Install and test the installation
- Initialize frate for your project
````shell
cargo install frate
frate --vesion
cd ~/your_project

frate init 
````

- Have a look at the available tools
````shell
frate registry
````

- Start using ``frate``
````shell
frate add just@1.42.1
frate sync

frate install
frate list

frate shell
just --vesion
````

---

## CLI

Frate offers a clean and expressive CLI inspired by tools like Cargo. Here's a breakdown of all available commands:



| Command                  | Description                                                                                         | Example Usage                 |
|--------------------------|-----------------------------------------------------------------------------------------------------|-------------------------------|
| `frate init`             | Initializes a new `frate.toml` in the current directory.                                            | `frate init`                  |
| `frate add <name>@<ver>` | Adds a tool to `frate.toml` and updates the lock file. Does **not** install the tool automatically. | `frate add just@1.14.0`       |
| `frate sync`             | Synchronizes `frate.lock` with the current `frate.toml`.                                            | `frate sync`                  |
| `frate install`          | Installs **all** packages listed in the lockfile.                                                   | `frate install`               |
| `frate install --name`   | Installs a **single** package by name.                                                              | `frate install --name just`   |
| `frate uninstall`        | Uninstalls **all** installed packages.                                                              | `frate uninstall`             |
| `frate uninstall --name` | Uninstalls a specific tool.                                                                         | `frate uninstall --name just` |
| `frate list`             | Lists all tools in `frate.toml`. Use `--verbose` for details.                                       | `frate list --verbose`        |
| `frate run <tool>`       | Runs a tool's binary from `.frate/bin/<tool>/`. Arguments can be passed through.                    | `frate run just -- --version` |
| `frate shell`            | Launches a new shell with all tools available in `PATH`.                                            | `frate shell`                 |
| `frate clean`            | Removes the global cache for **all** tools.                                                         | `frate clean`                 |
| `frate clean --name`     | Removes the cache for a **specific** tool.                                                          | `frate clean --name just`     |
| `frate search <name>`    | Searches for a tool and lists available versions from the registry.                                 | `frate search just`           |
| `frate which <name>`     | Outputs the full path to a tool's installed binary and its shim (if available).                     | `frate which just`            |

---

## frate.toml

````toml
[project]
name = "frate"
version = "0.1.0"

[dependencies]
just = "1.42.1"
ripgrep = "14.1.1"
````

---

## Use Case

Frate is designed for developers and teams who want to:

- Install CLI tools without requiring admin/root permissions

- Use a consistent, declarative configuration via frate.toml and frate.lock

- Reproduce development environments easily across machines or CI

- Avoid global pollution of PATH or conflicts between versions

- Share CLI tool dependencies with teammates through versioned lockfiles

It’s especially useful in the following scenarios:

- Local-only dev setups (e.g., on locked-down systems)

- Portable projects that include their CLI tooling

- CI pipelines that need deterministic toolchains

- Bootstrapping environments without relying on system package managers




---

## Contributing

Contributions are very welcome! Here’s how to get started:

1. Fork the repository

2. Clone your fork:
````shell
git clone https://github.com/your-username/frate.git
````

3. Create a feature branch:
````shell
git checkout -b my-feature
````

4. Make your changes and add tests if relevant

5. Run the test suite:
````shell
cargo test
````

6. Open a Pull Request with a clear description

Please follow Conventional Commits for your commit messages and try to keep the code style idiomatic.

If you're unsure where to start, check the [issues](https://github.com/konni332/frate/issues) marked good ``first issue`` or ``help wanted``.

### Notes

**All contributions are welcome!**  
*Bug fixes, docs, tests, feature/UX/CLI ideas, ...*

- If you create functions please make sure to use doc strings for all non trivial functions
- If you edit functions, please update existing doc strings accordingly

---

## Bug-Reports

🐛 Reporting Bugs

If you encounter a bug, please help improve Frate by reporting it.

Before submitting, make sure to:

1. Check the existing issues – your problem might already be reported.

2. Use a clear and descriptive title.

3. Include steps to reproduce the bug, your platform and OS, and any relevant logs or screenshots.

4. If possible, test against the latest ``master`` branch to see if the bug still occurs.

5. Create a [new issue](./.github/ISSUE_TEMPLATE/bug_report.md)

Thanks for helping make Frate better!

## License

This Project is licensed under either of:

[MIT](./LICENSE-MIT)  
[APACHE-2.0](./LICENSE-APACHE)

at your option

---
