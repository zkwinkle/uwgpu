# CLI

> CLI tool for executing microbenchmarks

## Usage

From the root folder the command `cargo cli` can be used to run this program.

Example executing scan with multiple workgroup sizes:

```sh
cargo cli scan -w 8 -w 16 -w 32 -w 64
```

An example of a microbenchmark that takes pairs of values for its workgroup
sizes:

```sh
cargo cli mat-mul -w 8,8 -w 64,1 -w 1,64
```

To see all available microbenchmarks:

```sh
cargo cli --help
```

## TODOs

- [ ] Add a command to run microbenchmarks and POST results to start gathering
      native execution data.
