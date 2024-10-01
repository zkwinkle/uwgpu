# uwgpu

> The core library for building wgpu microbenchmarks.

## Overview

The `uwgpu` crate is the core library used to create microbenchmarks.

## To Test

```not_rust
cargo test -p uwgpu
```

## Features

- `serde`: Enables serialization via serde on common wgpu types.

### Shading language support

The WGSL shading language is enabled by default, other options that can be enabled with feature flags:

- `spirv`: Enable accepting SPIR-V shaders as input.
- `naga-ir`: Enable accepting naga IR shaders as input.

## TODOs

- [ ] Change the way microbenchmarks are run to make a lot of smaller passes rather than 1 big pass with all the runs. (almost done)
- [x] Should I change from # of runs to time spent? probs not cause it would make it very slowwww
