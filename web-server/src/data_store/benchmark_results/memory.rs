#[derive(Debug, Clone)]
pub struct DataStoreMemoryBenchmark {
    /// The kind of memory benchmark this is
    kind: DataStoreMemoryBenchmarkKind,
    /// Bandwidth of memory copied in bytes per second (B/s)
    ///
    /// Example: 1.05MB/s = 1_050_000.0
    bandwidth: f64,
}

#[derive(Debug, Clone)]
pub enum DataStoreMemoryBenchmarkKind {
    /// Buffer sequential memory accesses
    BufferSequential,
    /// Buffer shuffled memory accesses
    BufferShuffled,
    /// Copying from buffer -> texture in a sequential manner
    BufferToTexture,
    /// Copying from texture -> texture in a sequential manner
    TextureToTexture,
}
