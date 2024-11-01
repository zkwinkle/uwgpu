use std::{path::Path, sync::Arc};

use sqlx::PgPool;

use crate::data_store::{DataStore, PostgresDataStore};

/// Configuration for the server.
pub struct AppConfig {
    /// Directory that holds static files like WASM modules
    pub public_dir: &'static Path,
    /// See [`PhotomulesDataStore`]
    pub data_store: Arc<dyn DataStore>,
    /// Example: https://zkwinkle.is-a.dev/microbenchmarks
    pub server_url: &'static str,
    /// User agent parser
    pub ua_parser: Arc<ua_parser::Extractor<'static>>,
}

/// Create AppConfig to use when running the server binary.
pub async fn create_app_config_from_env() -> AppConfig {
    let public_dir_str: &'static str = Box::new(read_env_public_dir()).leak();
    let public_dir = Path::new(public_dir_str);

    let pool = PgPool::connect(&read_env_database_url()).await.unwrap();
    let data_store = Arc::new(PostgresDataStore::new(pool));

    let server_url = Box::leak(Box::new(read_server_url()));

    let regexes = std::fs::File::open("crates/web-server/src/regexes.yaml").unwrap();
    let regexes: ua_parser::Regexes = serde_yaml::from_reader(regexes).unwrap();
    let ua_parser = Arc::new(ua_parser::Extractor::try_from(regexes).unwrap());

    AppConfig {
        data_store,
        public_dir,
        server_url,
        ua_parser,
    }
}

/// Read an environment variable, falling back on the default value only if the
/// `debug` feature flag is set, otherwise panicking.
fn read_env(var_name: &str, default_value_dev: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| {
        if cfg!(feature = "debug") {
            default_value_dev.to_owned()
        } else {
            panic!("Missing environment variable: {var_name}");
        }
    })
}

/// # Panics
///
/// Panics when the "debug" feature is disabled and the environment variable is
/// not found.
fn read_env_public_dir() -> String {
    read_env("PUBLIC_DIR", "crates/web-server/public")
}

/// # Panics
///
/// Panics when the "debug" feature is disabled and the environment variable is
/// not found.
fn read_env_database_url() -> String {
    read_env("DATABASE_URL", "postgres://postgres@localhost/uwgpu-local")
}

/// # Panics
///
/// Panics when the "debug" feature is disabled and the environment variable is
/// not found.
fn read_server_url() -> String {
    read_env("SERVER_URL", "http://localhost:31416")
}
