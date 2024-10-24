//! The datastore is an abstraction over the DB for writing/reading persistent
//! data that the server needs.

use benchmark_results::DataStoreBenchmarkResultsInterface;
use platform::DataStorePlatformInterface;
use sqlx::{pool::PoolConnection, PgPool};

mod benchmark_results;
mod platform;

mod non_empty_string;

/// Used to interface with the data store.
#[async_trait::async_trait]
pub trait DataStore:
    Send + Sync + DataStorePlatformInterface + DataStoreBenchmarkResultsInterface
{
}

/// Datastore implementation backed by a Postgres connection pool.
#[derive(Clone)]
pub struct PostgresDataStore {
    pool: PgPool,
}

impl DataStore for PostgresDataStore {}

impl PostgresDataStore {
    /// Create a new RealDataStore from a postgres connection pool.
    pub fn new(pool: PgPool) -> Self { PostgresDataStore { pool } }

    /// Get the database client
    pub(crate) async fn client(
        &self,
    ) -> Result<PoolConnection<sqlx::Postgres>, sqlx::Error> {
        self.pool.acquire().await
    }
}
