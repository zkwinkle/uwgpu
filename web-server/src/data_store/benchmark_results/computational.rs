#[derive(Debug, Clone)]
pub struct DataStoreComputationalBenchmark {
    /// The kind of computational benchmark this is
    kind: DataStoreComputationalBenchmarkKind,
    /// FLOPS (Floating Point Operations Per Second)
    flops: f64,
}

#[derive(Debug, Clone)]
pub enum DataStoreComputationalBenchmarkKind {
    Matmul,
    Reduction,
    Convolution,
    Scan,
}
