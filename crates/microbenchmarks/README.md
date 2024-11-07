# uwgpu microbenchmarks

> Library that uses uwgpu under the hood to implement a suite of microbenchmarks

## Microbenchmarks

- matmul
- convolution
- reduction sum
- scan (prefix sum)
- memcpy between buffers
- memcpy buffer->texture
- memcpy between textures

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
