# Micro WebGPU Compute

## Comandos

Watch el servidor web:

```sh
cargo w
```

Compilar WASM que el servidor pueda acceder:

```sh
wasm-pack build uwgpu -d ../web-server/public/pkg --target web
```

Compilar y ejecutar CLI:

```sh
cargo cli <microbenchmark>
```

# TODO

Writing this down here so I don't forget, I should create a separate crate with the types that can be sent to the server as requests, and methods to
build them (like the post results request).

I could compile it to WASM and use it from the JS code to ensure data integrity/consistency.

Then I could also use the types in the CLI so that the CLI can easily report its results.
