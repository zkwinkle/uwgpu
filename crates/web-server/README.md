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

- `PUBLIC_DIR`: URL for the postgres DB to use.
- `DATABASE_URL`: Path to the `public` directory.
- `SERVER_URL`: The public URL of the server.

## Adding migrations

Use the `sqlx` command. For examlpe in my local development environment:

```sh
sqlx migrate migrate add create_some_table
sqlx run --database-url 'postgres://postgres@localhost/uwgp-local'
```

## TODOs

- [x] Explain compatible browsers in home page.
- [x] Add explanation about how microbenchmarks are very naively implemented and not optimized.
- [ ] Add estimated time for full suite.
- [ ] Add notification or msg at the top of execution log to let user know when microbenchmarks finished.
- [ ] Figure out why errors in WASM don't get caught (trying to run tests on Firefox for example).
- [ ] (optional) make benchmarks take count and warmup count as params
- [ ] (optional) make web benchmarks take longer
- [ ] CSV download.
- [ ] Make it so historical data filters include other filters in their request
      (and re-request when they change), so for example users don't even have
      the option to select apple hardware with a Windows filter.
- [ ] Sort results by workgroup sizes or something when showing the historic
      data table.
