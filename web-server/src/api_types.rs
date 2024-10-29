use serde::{Deserialize, Serialize};
use MicrobenchmarkKind::*;

/// A specific benchmark supported by the website.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum MicrobenchmarkKind {
    Matmul,
    Reduction,
    Convolution,
    Scan,
    BufferSequential,
    BufferShuffled,
    BufferToTexture,
    TextureToTexture,
}

impl MicrobenchmarkKind {
    pub const fn path(&self) -> &'static str {
        match self {
            Matmul => "/matmul",
            Reduction => "/reduction",
            Convolution => "/convolution",
            Scan => "/scan",
            BufferSequential => "/buffer_sequential",
            BufferShuffled => "/buffer_shuffled",
            BufferToTexture => "buffer_to_texture",
            TextureToTexture => "texture_to_texture",
        }
    }
}

/// Filters that can be applied when obtaining statistical data of benchmark
/// results.
#[derive(Debug, serde::Deserialize)]
pub struct BenchmarkResultsFilters {
    pub hardware: Option<Hardware>,
    pub operating_system: Option<String>,
    pub platform: Option<Platform>,
}

/// Fields used when listing and querying available hardware
///
/// Not using the [NonEmptyString] for easy decoding from DB, i can assume the
/// strings aren't empty tho.
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Hardware {
    pub webgpu_vendor: String,
    pub webgpu_architecture: String,
}

/// Supported general "platforms" for filtering results
///
/// These all have different ways of being queried for, so that's why we decide
/// to just state them in this enum instead of doing some heuristic query of the
/// DB to find the available variants. (Like we do for [Hardware] variants for
/// example.)
#[derive(Debug, Serialize, Deserialize)]
pub enum Platform {
    Chromium,
    Firefox,
    OtherBrowser,
    NativeVulkan,
    NativeMetal,
    NativeDx12,
}
