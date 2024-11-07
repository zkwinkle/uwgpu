#[derive(Debug, Clone)]
pub struct DataStoreMemoryBenchmark {
    /// The kind of memory benchmark this is
    pub kind: DataStoreMemoryBenchmarkKind,
    /// Bandwidth of memory copied in bytes per second (B/s)
    ///
    /// Example: 1.05MB/s = 1_050_000.0
    pub bandwidth: f64,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "memory_benchmark_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum DataStoreMemoryBenchmarkKind {
    /// Buffer sequential memory accesses
    BufferSequential,
    /// Buffer shuffled memory accesses
    BufferShuffled,
    /// Copying from buffer -> buffer
    BufferToBuffer,
    /// Copying from buffer -> texture
    BufferToTexture,
    /// Copying from texture -> texture
    TextureToTexture,
}
