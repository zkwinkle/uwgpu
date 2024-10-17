//! The datastore is an abstraction over the DB for writing/reading persistent
//! data that the server needs.

mod benchmark_results;
mod platform;

mod non_empty_string;

/// Used to interface with the data store.
///
/// This is backed by a Postgres database.
pub trait DataStore {}
