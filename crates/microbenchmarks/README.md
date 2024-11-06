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
- `wasm`: Enables WASM support for the library.

## TODOs

- [ ] Fix reduction sum, right now it's flaky, sometimes gives the correct
      result but sometimes not. The test `reduction_works` can be used to try it
      out.
