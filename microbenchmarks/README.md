# uwgpu microbenchmarks

> Library that uses uwgpu under the hood to implement a suite of microbenchmarks

## Microbenchmarks

- matmul
- copy buffers sequential

## To Test

```not_rust
cargo test -p microbenchmarks
```

## Features

- `serde`: Enables serialization via serde on common wgpu types.
