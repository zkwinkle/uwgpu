#[derive(Debug, Clone)]
pub struct DataStoreComputationalBenchmark {
    /// The kind of computational benchmark this is
    pub kind: DataStoreComputationalBenchmarkKind,
    /// FLOPS (Floating Point Operations Per Second)
    pub flops: f64,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "computational_benchmark_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum DataStoreComputationalBenchmarkKind {
    Matmul,
    Reduction,
    Convolution,
    Scan,
}
