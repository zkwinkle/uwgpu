# uwgpu

> The core library for building wgpu microbenchmarks.

## Overview

The `uwgpu` crate is the core library used to create microbenchmarks.

## To Test

```not_rust
cargo test -p uwgpu
```

## Features

- `serde`: Enables serialization via serde on common wgpu types and the librarie's error ttpes.
- `spir-v`: Exposes support to compile spir-v shaders with wgpu.
- `naga-ir`: Exposes support to compile naga intermediate representation shaders with wgpu.
- `wasm`: Enables support for building for WASM.

### Shading language support

The WGSL shading language is enabled by default, other options that can be enabled with feature flags:

- `spirv`: Enable accepting SPIR-V shaders as input.
- `naga-ir`: Enable accepting naga IR shaders as input.
