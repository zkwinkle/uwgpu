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

To update DB cache for `sqlx` to check queries locally:
```
# From web-server dir
cargo sqlx prepare --database-url 'postgres://postgres@localhost/uwg
p-local'
```

Make sure the `postgresql` service is running:
```
sudo systemctl start postgresql
```

## Production

For the production build, the `debug` flag must be disabled by disabling the
default features with `--no-default-features` flag.

Website can be found at \[TODO\]

### Environment variables

These must be set when running in production

- `PUBLIC_DIR`: Path to the `public` directory.

## Adding migrations

Use the `sqlx` command. For examlpe in my local development environment:

```sh
sqlx migrate run --database-url 'postgres://postgres@localhost/uwgp-local'
```

## TODOs

- [ ] (optional) make benchmarks take count and warmup count as params
- [ ] (optional) make web benchmarks take longer
- [ ] CSV download.
