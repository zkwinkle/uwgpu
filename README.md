# **µwgpu**

## Commands

For launching and watching the web-server crate:

```sh
cargo w
```

To compile the microbenchmark's library as a WASM pack that the server can use:
```sh
wasm-pack build crates/microbenchmarks -d ../web-server/public/pkg --target web --no-typescript --mode no-install --no-pack -- --features wasm
```

Compile and run CLI:

```sh
cargo cli <microbenchmark>
```

# Nix Flake

A `flake.nix` file is supplied, currently it only offers a package for the
`web-server` crate.

To build the package, use the following command:
```sh
nix build '.#web-server' --extra-experimental-features "nix-command flakes" --show-trace
```

# TODO

- Create a separate crate with API types that can be sent to the server as requests, and methods to build them (like the post results request). Can be compiled to WASM and used from JS code to ensure data integrity/consistency.
- Make CLI also report results. (Can also use the separate crate with API types).
