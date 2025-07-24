//! # Frate Core Library
//!
//! This crate contains the core logic and building blocks of the `frate` tool – a developer-friendly package manager
//! for local tool installations with a `cargo`-like UX.
//!
//! `frate` allows teams to define, sync, and install binary tools using versioned manifests (`frate.toml`, `frate.lock`),
//! without needing system-wide privileges.
//!
//! This library is built for the `frate` CLI, but you can also reuse it as a backend in other tools.
//!
//! ## Modules Overview
//! - [`toml`] – Parsing and serialization of `frate.toml` manifest files
//! - [`lock`] – Lockfile structure and sync logic (`frate.lock`)
//! - [`registry`] – Handling registry sources and resolving tool versions
//! - [`installer`] – Installing, caching, and managing tool binaries
//! - [`shims`] – Creating proxy shims to forward tool invocations
//! - [`util`] – Shared utilities (paths, logging, hashing, etc.)
//! - [`global`] – Global state and configuration (e.g., cache directory)


pub mod toml;
pub mod lock;
pub mod registry;
pub mod util;
pub mod installer;
pub mod shims;
pub mod global;

pub use shims::*;
pub use installer::*;
pub use lock::*;
pub use registry::*;
pub use toml::*;
pub use util::*;
pub use global::cache::*;
