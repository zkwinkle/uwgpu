use crate::data_store::benchmark_results::computational::DataStoreComputationalBenchmarkKind;
use crate::data_store::benchmark_results::DataStoreMemoryBenchmarkKind;
use std::ops::DerefMut;

use sqlx::{types::Uuid, Acquire, Postgres, Transaction};

use crate::data_store::PostgresDataStore;

use super::{
    computational::DataStoreComputationalBenchmark,
    memory::DataStoreMemoryBenchmark, DataStoreBenchmarkKind,
    DataStoreBenchmarkResults, DataStoreCreateBenchmarkResult,
};

pub trait DataStoreBenchmarkResultsInterface {
    async fn create_benchmark_results(
        &self,
        create: DataStoreCreateBenchmarkResult,
    ) -> Result<DataStoreBenchmarkResults, sqlx::Error>;
}

impl DataStoreBenchmarkResultsInterface for PostgresDataStore {
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
