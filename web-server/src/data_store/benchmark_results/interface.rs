use crate::api_types::{BenchmarkResultsFilters, MicrobenchmarkKind, Platform};
use crate::data_store::benchmark_results::DataStoreMemoryBenchmarkKind;
use crate::{
    api_types::BenchmarkResultsStatistics,
    data_store::benchmark_results::computational::DataStoreComputationalBenchmarkKind,
};
use sqlx::postgres::PgRow;
use sqlx::{types::Uuid, Acquire, FromRow, Postgres, Row, Transaction};
use std::collections::HashMap;
use std::ops::DerefMut;
use MicrobenchmarkKind::*;

use crate::data_store::PostgresDataStore;

use super::{
    computational::DataStoreComputationalBenchmark,
    memory::DataStoreMemoryBenchmark, DataStoreBenchmarkKind,
    DataStoreBenchmarkResults, DataStoreCreateBenchmarkResult,
};

#[async_trait::async_trait]
pub trait DataStoreBenchmarkResultsInterface {
    /// Creates a record of a benchmark execution result
    async fn create_benchmark_results(
        &self,
        create: DataStoreCreateBenchmarkResult,
    ) -> Result<DataStoreBenchmarkResults, sqlx::Error>;

    /// Returns the stats for a benchmark described by the filters, each result
    /// corresponds to a unique workgroup size combination.
    async fn get_benchmark_results_statistics(
        &self,
        filters: BenchmarkResultsFilters,
    ) -> Result<Vec<BenchmarkResultsStatistics>, sqlx::Error>;
}

#[async_trait::async_trait]
impl DataStoreBenchmarkResultsInterface for PostgresDataStore {
    // TODO: The functions to build this query are wildly messy and untested.
    // Very worthwhile to try to abstract some of this stuff, reorganize the
    // code, break up the functions, and write tests.
    // I don't have any testing infrastructure for datastore methods yet, so
    // should do that first.
    async fn create_benchmark_results(
        &self,
        create: DataStoreCreateBenchmarkResult,
    ) -> Result<DataStoreBenchmarkResults, sqlx::Error> {
        let mut client = self.client().await?;

        let mut tx = client.begin().await?;

        match &create.kind {
            DataStoreBenchmarkKind::Computational(comp_create) => {
                let computational_id =
                    create_computational_benchmark(&mut tx, &comp_create)
                        .await?;

                sqlx::query!(
                    r#"
                        INSERT INTO benchmark_results (platform_id, count, total_time_spent, workgroup_size_x, workgroup_size_y, workgroup_size_z, computational_benchmark_id)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        ON CONFLICT DO NOTHING
                    "#,
                    create.platform_id,
                    create.count as i32,
                    create.total_time_spent,
                    create.workgroup_size.0 as i32,
                    create.workgroup_size.1 as i32,
                    create.workgroup_size.2 as i32,
                    computational_id,
                )
                    .execute(tx.deref_mut()).await?;
            }
            DataStoreBenchmarkKind::Memory(mem_create) => {
                let memory_id =
                    create_memory_benchmark(&mut tx, &mem_create).await?;

                sqlx::query!(
                    r#"
                        INSERT INTO benchmark_results (platform_id, count, total_time_spent, workgroup_size_x, workgroup_size_y, workgroup_size_z, memory_benchmark_id)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        ON CONFLICT DO NOTHING
                    "#,
                    create.platform_id,
                    create.count as i32,
                    create.total_time_spent,
                    create.workgroup_size.0 as i32,
                    create.workgroup_size.1 as i32,
                    create.workgroup_size.2 as i32,
                    memory_id,
                )
                    .execute(tx.deref_mut()).await?;
            }
        };

        tx.commit().await?;

        Ok(DataStoreBenchmarkResults {
            platform_id: create.platform_id,
            count: create.count,
            total_time_spent: create.total_time_spent,
            workgroup_size: create.workgroup_size,
            kind: create.kind,
        })
    }

    async fn get_benchmark_results_statistics(
        &self,
        filters: BenchmarkResultsFilters,
    ) -> Result<Vec<BenchmarkResultsStatistics>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT
                br.workgroup_size_x,
                br.workgroup_size_y,
                br.workgroup_size_z,
                AVG(br.total_time_spent / br.count) AS average_time_per_iter,
                AVG(mk.{custom_metric}) AS average_custom_result,
                COUNT(*) AS result_count
            FROM
                benchmark_results br
            JOIN
                {microbenchmark_kind_table} mk ON br.{microbenchmark_kind_table}_id = mk.{microbenchmark_kind_table}_id
            WHERE
                {platform_subquery}
                mk.kind = $1
            GROUP BY
                br.workgroup_size_x,
                br.workgroup_size_y,
                br.workgroup_size_z;
            "#,
            platform_subquery = platform_subquery(&filters),
            custom_metric = custom_metric_column(filters.microbenchmark),
            microbenchmark_kind_table =
                microbenchmark_table_join(filters.microbenchmark),
        );

        let mut client = self.client().await?;

        let query = sqlx::query_as(&query);
        let query = match filters.microbenchmark {
            Matmul => query.bind(DataStoreComputationalBenchmarkKind::Matmul),
            Reduction => {
                query.bind(DataStoreComputationalBenchmarkKind::Reduction)
            }
            Convolution => {
                query.bind(DataStoreComputationalBenchmarkKind::Convolution)
            }
            Scan => query.bind(DataStoreComputationalBenchmarkKind::Scan),
            BufferSequential => {
                query.bind(DataStoreMemoryBenchmarkKind::BufferSequential)
            }
            BufferShuffled => {
                query.bind(DataStoreMemoryBenchmarkKind::BufferShuffled)
            }
            BufferToTexture => {
                query.bind(DataStoreMemoryBenchmarkKind::BufferToTexture)
            }
            TextureToTexture => {
                query.bind(DataStoreMemoryBenchmarkKind::TextureToTexture)
            }
        };

        let results: Vec<BenchmarkResultsStatistics> =
            query.fetch_all(client.acquire().await?).await?;

        Ok(results)
    }
}

fn platform_subquery(filters: &BenchmarkResultsFilters) -> String {
    if filters.hardware.is_none()
        && filters.platform.is_none()
        && filters.operating_system.is_none()
    {
        return String::from("");
    }

    // <Table , Vec<WHERE conditions> >
    //
    // ## Possible future extensions:
    //
    // Currently, each possible filter uses a different table. So that's why I
    // just insert(). We could use the entry API in the future if it gets more
    // complicated.
    //
    // If we need to use dynamic params ($2, $3, ...) or multiple checks on
    // the same field.
    //
    // // <table, <<field name>, <value>> >
    // HashMap<PlatformTables, HashMap<String, Box<dyn list of traits needed by
    // bind()> > >
    let mut conditions: HashMap<PlatformTables, Vec<String>> = HashMap::new();

    if let Some(hardware) = &filters.hardware {
        // TODO: The hardware filter could also take into account wgpu stuff and
        // maybe user agent device id in the future, when I have more data to
        // experiment with.
        conditions.insert(
            PlatformTables::WebGpu,
            vec![
                format!("architecture = '{}'", hardware.webgpu_architecture),
                format!("vendor = '{}'", hardware.webgpu_vendor),
            ],
        );
    }

    if let Some(os) = &filters.operating_system {
        conditions.insert(
            PlatformTables::UserAgentOs,
            vec![format!("operating_system = '{}'", os)],
        );
    }

    if let Some(platform) = &filters.platform {
        match platform {
            Platform::Chromium => conditions.insert(
                PlatformTables::UserAgent,
                vec![format!("family = 'Chrome'")],
            ),
            Platform::Firefox => conditions.insert(
                PlatformTables::UserAgent,
                vec![format!("family = 'Firefox'")],
            ),
            Platform::OtherBrowser => {
                todo!("Other browsers not supported yet to filter by")
            }
            Platform::NativeVulkan => {
                todo!("Native not supported yet to filter by")
            }
            Platform::NativeMetal => {
                todo!("Native not supported yet to filter by")
            }
            Platform::NativeDx12 => {
                todo!("Native not supported yet to filter by")
            }
        };
    }

    let mut joins = String::from("");

    if conditions.contains_key(&PlatformTables::UserAgent)
        || conditions.contains_key(&PlatformTables::UserAgentOs)
    {
        joins += "JOIN user_agent_string_info uasi ON uasi.user_agent_string_info_id = p.user_agent_string_info_id ";
    }
    let joins = conditions.keys().fold(joins, |joins, table| {
        joins
            + &match table {
                PlatformTables::Wgpu | PlatformTables::WebGpu => {
                    format!(
                        " JOIN {name} ON {name}.{name}_id = p.{name}_id ",
                        name = table.table_name()
                    )
                }
                PlatformTables::UserAgent | PlatformTables::UserAgentOs => {
                    format!(
                        " JOIN {name} ON {name}.{name}_id = uasi.{name}_id ",
                        name = table.table_name()
                    )
                }
            }
    });

    let where_clauses = conditions
        .iter()
        .flat_map(|(table, conditions)| {
            conditions.iter().map(move |condition| {
                format!("{}.{}", table.table_name(), condition)
            })
        })
        .collect::<Vec<String>>()
        .join(" AND ");

    let platform_subquery = format!(
        r#"
    br.platform_id IN (
        SELECT p.platform_id
        FROM platform p
        {joins}
        WHERE
        {where_clauses}
    ) AND
    "#
    );

    platform_subquery
}

#[derive(Eq, Hash, PartialEq)]
enum PlatformTables {
    #[expect(
        unused,
        reason = "Because we're only collecting data from the web where wgpu
doesn't return any useful values."
    )]
    // TODO: Take Wgpu into account for filtering when also collecting data
    // from native platforms
    Wgpu,
    WebGpu,
    UserAgent,
    UserAgentOs,
}

impl PlatformTables {
    fn table_name(&self) -> &'static str {
        match self {
            PlatformTables::Wgpu => "wgpu_adapter_info",
            PlatformTables::WebGpu => "webgpu_adapter_info",
            PlatformTables::UserAgent => "user_agent",
            PlatformTables::UserAgentOs => "user_agent_os",
        }
    }
}

impl FromRow<'_, PgRow> for BenchmarkResultsStatistics {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            workgroup_size: (
                row.try_get::<i32, _>("workgroup_size_x")? as u32,
                row.try_get::<i32, _>("workgroup_size_y")? as u32,
                row.try_get::<i32, _>("workgroup_size_z")? as u32,
            ),
            average_time_per_iter: row.try_get("average_time_per_iter")?,
            average_custom_result: row.try_get("average_custom_result")?,
            result_count: row.try_get::<i64, _>("result_count")? as usize,
        })
    }
}

fn custom_metric_column(microbenchmark: MicrobenchmarkKind) -> &'static str {
    match microbenchmark {
        Matmul | Reduction | Convolution | Scan => "flops",
        BufferSequential | BufferShuffled | BufferToTexture
        | TextureToTexture => "bandwidth",
    }
}

fn microbenchmark_table_join(
    microbenchmark: MicrobenchmarkKind,
) -> &'static str {
    match microbenchmark {
        Matmul | Reduction | Convolution | Scan => "computational_benchmark",
        BufferSequential | BufferShuffled | BufferToTexture
        | TextureToTexture => "memory_benchmark",
    }
}

async fn create_computational_benchmark(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreComputationalBenchmark,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
        r#"
                INSERT INTO computational_benchmark (kind, flops)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                RETURNING computational_benchmark_id
            "#,
        &create.kind as &DataStoreComputationalBenchmarkKind,
        &create.flops,
    )
    .fetch_one(tx.deref_mut())
    .await?;

    Ok(id)
}

async fn create_memory_benchmark(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreMemoryBenchmark,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
        r#"
                INSERT INTO memory_benchmark (kind, bandwidth)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                RETURNING memory_benchmark_id
            "#,
        &create.kind as &DataStoreMemoryBenchmarkKind,
        &create.bandwidth,
    )
    .fetch_one(tx.deref_mut())
    .await?;

    Ok(id)
}
