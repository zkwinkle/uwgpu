{
  description = "A flake for building the web-server and the microbenchmarks WASM library";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain
          (p: p.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default)
          );
        craneWasmLib = (crane.mkLib pkgs).overrideToolchain
          (p: p.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              targets = [ "wasm32-unknown-unknown" ];
            })
          );

        commonArgs = {
          strictDeps = true;
          doCheck = false;
        };

        fileSetForCrate = crate: lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./crates/uwgpu
            crate
          ];
        };

        microbenchmarks-wasm = craneWasmLib.buildPackage (commonArgs // {
          pname = "microbenchmarks-wasm-pack";
          version = "0.1";

          src = fileSetForCrate ./crates/microbenchmarks;
          nativeBuildInputs = [
            pkgs.wasm-pack
            # binaryen for wasm-opt, used by wasm-pack
            pkgs.binaryen
            # used by wasm-pack
            pkgs.wasm-bindgen-cli
          ];

          WASM_PACK_CACHE = "$TMPDIR/.wasm-pack-cache";
          buildPhaseCargoCommand = ''
            						wasm-pack build --release crates/microbenchmarks -d "$(realpath .)"/pkg --target web --no-typescript --mode no-install --no-pack -- --features wasm
            					'';
          installPhaseCommand = ''
            						mv pkg $out/
            					'';
        });

        web-server = craneLib.buildPackage (commonArgs // {
          pname = "uwgpu-web-server";
          version = "0.1";
          cargoExtraArgs = "-p web-server --no-default-features";
          src = fileSetForCrate ./crates/web-server;
          postInstall = ''
            cp -r crates/web-server/public $out/public
            cp --no-preserve=mode -r ${microbenchmarks-wasm} $out/public/pkg
          '';
        });
      in
      {
        packages = {
          inherit web-server;
        };
      });
}
