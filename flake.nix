{
  description = "A flake for building the web-server and the microbenchmarks WASM library";

inputs = {
		nixpkgs.url = "nixpkgs/nixos-24.05";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, rust-overlay, ... }:
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
							extensions = [ "llvm-tools-preview" ];
						})
					);
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          doCheck = false;
        };

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

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
						pkgs.cargo-binutils
						pkgs.wasm-pack
						pkgs.lld
						# binaryen for wasm-opt, used by wasm-pack
						pkgs.binaryen
						# used by wasm-pack
						pkgs.wasm-bindgen-cli
					];

          WASM_PACK_CACHE = "$TMPDIR/.wasm-pack-cache";
					# Needed because dependencies build was failing with:
					# > error: the `-Z unstable-options` flag must also be passed to enable
					#	the flag `check-cfg`
					RUSTFLAGS="-Z unstable-options";
					buildPhaseCargoCommand = ''
						wasm-pack build --release crates/microbenchmarks -d "$(realpath .)"/pkg --target web --no-typescript --mode no-install -- --features wasm
					'';
					installPhaseCommand = ''
						mv pkg $out/
					'';
        });

        web-server = craneLib.buildPackage (commonArgs // {
          pname = "uwgpu-web-server";
					version = "0.1";
					nativeBuildInputs = [
						pkgs.makeWrapper
					];
          cargoExtraArgs = "-p web-server --no-default-features";
          src = fileSetForCrate ./crates/web-server;
					postInstall = ''
            cp -r crates/web-server/public $out/public
            cp --no-preserve=mode -r ${microbenchmarks-wasm} $out/public/pkg
            wrapProgram $out/bin/web-server \
            --set PUBLIC_DIR "$out/public" \
            --set SERVER_URL "https://zkwinkle.is-a.dev/uwgpu" \
            --set DATABASE_URL "postgres://postgres@localhost/uwgpu"
            '';
        });
      in
      {
        packages = {
          inherit web-server;
        } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          my-workspace-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs);
        };
			});
}
