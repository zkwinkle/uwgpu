# web-server

> Axum server for serving the web UI.

## Overview

Serves the web UI with the WASM for running wgpu in the browser.

## To Test

```not_rust
cargo test -p web-server
```

## Debug

For debug builds and running locally, the `debug` feature flag is provided.
It is enabled by default.

To watch:
```
# From root dir
cargo w
```

## Production

For the production build, the `debug` flag must be disabled by disabling the
default features with `--no-default-features` flag.

Website can be found at \[TODO\]

### Environment variables

These must be set when running in production

- `PUBLIC_DIR`: Path to the `public` directory.
