{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default";

    nix-github-actions.url = "github:nix-community/nix-github-actions";
    nix-github-actions.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    systems,
    nixpkgs,
    nix-github-actions,
    ...
  } @ inputs: let
    eachSystem = f:
      nixpkgs.lib.genAttrs (import systems) (
        system:
          f (import nixpkgs {
            inherit system;
            overlays = [inputs.rust-overlay.overlays.default];
          })
      );

    # rustToolchain = eachSystem (pkgs: (pkgs.rust-bin.selectLatestStableWith (t: t.default)).override {
    #   extensions = ["rust-src" "rust-analyzer"];
    #   targets = ["wasm32-unknown-unknown"];
    # });
    rustToolchain = eachSystem (pkgs: pkgs.rust-bin.stable.latest.default.override {
      extensions = ["rust-src" "rust-analyzer"];
      targets = ["wasm32-unknown-unknown"];
    });

    dioxus-cli = eachSystem (pkgs: pkgs.dioxus-cli.overrideAttrs (_: {
      postPatch = ''
        rm Cargo.lock
        cp ${./Dioxus.lock} Cargo.lock
      '';

      cargoDeps = pkgs.rustPlatform.importCargoLock {
        lockFile = ./Dioxus.lock;
      };
    }));

    cargoLock = builtins.fromTOML (builtins.readFile ./Cargo.lock);

    wasmBindgen = eachSystem (pkgs: (pkgs.lib.findFirst
      (pkg: pkg.name == "wasm-bindgen")
      (throw "Could not find wasm-bindgen package")
      cargoLock.package));

    wasm-bindgen-cli = eachSystem (pkgs: (pkgs.buildWasmBindgenCli rec {
      src = pkgs.fetchCrate {
        pname = "wasm-bindgen-cli";
        version = wasmBindgen.${pkgs.system}.version;
        hash = "sha256-1VwY8vQy7soKEgbki4LD+v259751kKxSxmo/gqE6yV0=";
      };
      cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
        inherit src;
        inherit (src) pname version;
        hash = "sha256-81vQkKubMWaX0M3KAwpYgMA1zUQuImFGvh5yTW+rIAs=";
      };
    }));



  in rec {
    devShells = eachSystem (pkgs: {
      # Based on a discussion at https://github.com/oxalica/rust-overlay/issues/129
      default = pkgs.mkShell (with pkgs; {
        nativeBuildInputs = [
          clang
          # Use mold when we are runnning in Linux
          (lib.optionals stdenv.isLinux mold)
        ];

        buildInputs = [
          rustToolchain.${pkgs.system}
          cargo
          diesel-cli
          dioxus-cli.${pkgs.system}
          wasm-bindgen-cli.${pkgs.system}
        ] ++
        (pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          openssl
          pkg-config
        ]));

        # RUST_SRC_PATH = "${
        #   rustToolchain.${pkgs.system}.rust-src
        # }/lib/rustlib/src/rust/library";
        RUST_BACKTRACE = "1";
      });
    });

    # TODO: We'll use `dx bundle` to create bundle.
    # Should we still use nix to build it?
    # Or just run `dx bundle` in CI?
  };
}
