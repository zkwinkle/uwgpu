# **µwgpu**

_µwgpu_ is a project leveraging the [wgpu](https://github.com/gfx-rs/wgpu)
graphics API to create a cross-platform microbenchmarking compute pipeline, a
collection of microbenchmarks, a CLI tool for native execution and a website
for browser execution.

This repository includes the following crates:

- [`uwgpu`](/crates/uwgpu): Core library for writing microbenchmarks.
- [`microbenchmarks`](/crates/microbenchmarks): Suite of microbenchmarks implemented with `uwgpu`.
- [`microbenchmarks-cli`](/crates/microbenchmarks-cli): CLI tool for native execution of the microbenchmarks.
- [`web-server`](/crates/web-server): Server that serves the [µwgpu website](https://zkwinkle.is-a.dev/uwgpu) and collects execution results.


**⚠️ NOTE:** This project is in highly experimental stages. None of the crates
have been published to [crates.io](https://crates.io/) yet due to being very
likely to receive **a lot** of breaking changes.

## Rationale

This project takes inspiration from [µVkCompute](https://github.com/google/uVkCompute) and also attempts to probe and understand the characteristics of the target hardware.

But, thanks to leveraging [wgpu](https://github.com/gfx-rs/wgpu) "cross-platformness", we can gather execution statistics for a wide array of hardware being executed on different platforms. It might also give insight to the differences in performance of the WebGPU backends.

## Native support

Native execution through the CLI has only been tested on Linux, but there
should be no reason for it to not work on other major platforms.

## Browser support

At the moment, browser execution only works on Chrome or Chromium-based browsers.
And if you're on Linux you'll need to set [an additional flag](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status#implementation-status) to enable WebGPU.

Firefox will be supported once it [implements support for timestamp queries](https://bugzilla.mozilla.org/show_bug.cgi?id=1838729).


## Useful commands

For launching and watching the web-server crate:

```sh
cargo w
```

To compile the microbenchmark's library as a WASM pack that the server can use:
```sh
wasm-pack build crates/microbenchmarks -d ../web-server/public/pkg --target web --no-typescript --no-pack -- --features wasm
```

Compile and run CLI:

```sh
cargo cli <microbenchmark>
```

## Nix Flake

A `flake.nix` file is supplied, currently it only offers a package for the
`web-server` crate.

To build the package, use the following command:
```sh
nix build '.#web-server' --extra-experimental-features "nix-command flakes" --show-trace
```

## TODO

- [ ] Create a separate crate with API types that can be sent to the server as requests, and methods to build them (like the post results request). Can be compiled to WASM and used from JS code to ensure data integrity/consistency. Can also be used by the CLI.
