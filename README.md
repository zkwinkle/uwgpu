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
