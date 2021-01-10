#![recursion_limit = "1024"] // required for select!
#![warn(clippy::all, rust_2018_idioms, rust_2018_compatibility)]
// #![warn(missing_docs)]

//! The Daemon which watches for file changes and maintains database state.

/// the artifact cache
mod artifact_cache;

/// the asset hub
mod asset_hub;

/// the asset hub service
mod asset_hub_service;

/// the capnp database
mod capnp_db;

mod daemon;
mod error;
mod file_asset_source;
mod file_tracker;
mod logging;
mod scope;
mod serialized_asset;
mod source_pair_import;
mod watcher;

pub use crate::{
    daemon::{default_importer_contexts, default_importers, AssetDaemon, ImporterMap},
    error::{Error, Result},
    logging::init_logging,
};
