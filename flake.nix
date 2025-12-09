{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    systems.url = "github:nix-systems/default";

    nix-github-actions.url = "github:nix-community/nix-github-actions";
    nix-github-actions.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    systems,
    nixpkgs,
    nix-github-actions,
    crane,
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

    rustToolchain = eachSystem (pkgs: pkgs.rust-bin.stable.latest.default.override {
      extensions = ["rust-src" "rust-analyzer"];
      targets = ["wasm32-unknown-unknown"];
    });

    dioxus-cli = eachSystem (pkgs: pkgs.dioxus-cli.overrideAttrs (oldAttrs: {
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
        version = wasmBindgen.${pkgs.stdenv.hostPlatform.system}.version;
        hash = "sha256-9kW+a7IreBcZ3dlUdsXjTKnclVW1C1TocYfY8gUgewE=";
      };
      cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
        inherit src;
        inherit (src) pname version;
        hash = "sha256-V0AV5jkve37a5B/UvJ9B3kwOW72vWblST8Zxs8oDctE=";
      };
    }));



  in rec {
    devShells = eachSystem (pkgs: {
      # Based on a discussion at https://github.com/oxalica/rust-overlay/issues/129
      default = pkgs.mkShell (with pkgs; {
        nativeBuildInputs = [
          # Use mold when we are runnning in Linux
          (lib.optionals stdenv.isLinux mold)
          sqlite
          darwin.sigtool
          binaryen
        ];

        buildInputs = [
          rustToolchain.${pkgs.stdenv.hostPlatform.system}
          cargo
          dioxus-cli.${pkgs.stdenv.hostPlatform.system}
          wasm-bindgen-cli.${pkgs.stdenv.hostPlatform.system}
          nodejs
          lld
        ] ++
        (pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          openssl
          pkg-config
        ])) ++
        (pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [
          apple-sdk_15
        ]));

        # RUST_SRC_PATH = "${
        #   rustToolchain.${pkgs.stdenv.hostPlatform.system}.rust-src
        # }/lib/rustlib/src/rust/library";
        RUST_BACKTRACE = "1";
        RUST_LOG = "warn,nju_schedule_ics=debug";
      });
    });

    packages = eachSystem(pkgs: rec {
      server = let
        system = pkgs.stdenv.hostPlatform.system;
        craneLib = crane.mkLib pkgs;
        assets = pkgs.runCommand "assets" { } ''
            mkdir -p $out
            cd assets/
            npx tailwind -o $out/tailwind_output.css
          '';
        commonArgs = rec {
          src = (pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = manifestFilter;
            name = "source";
          });
          # src = builtins.trace src1.outPath src1;

          nativeBuildInputs = devShells.${system}.default.nativeBuildInputs;
          buildInputs = devShells.${system}.default.buildInputs;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        # Keep all source and /assets for building this crate
        sourceFilter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*assets/.*" path != null);
        # Only keep Cargo.toml and Cargo.lock, for building dependencies
        manifestFilter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*/Cargo\\..*" path != null);
        tailwind-assets = pkgs.buildNpmPackage {
          name = "tailwind-assets";
          src = ./assets;

          npmDepsHash = "sha256-HRMLzN2s0CKjHXx23MAL4EURhzHhpb6gtSsocva6q8s=";

          # Override the build command to generate the specific file you need
          # Adjust 'input.css' to whatever your source css file is named
          buildPhase = ''
            npx @tailwindcss/cli -i tailwind.css -o tailwind_output.css
          '';

          installPhase = ''
            mkdir -p $out
            cp tailwind_output.css $out/
          '';
        };

        in craneLib.buildPackage (
          commonArgs // {
            inherit cargoArtifacts;

            src = (pkgs.lib.cleanSourceWith {
              src = ./.;
              filter = sourceFilter;
              name = "source";
            });

            buildPhase = ''
              cp ${tailwind-assets}/tailwind_output.css assets/tailwind_output.css
              export CARGO_HOME=$cargoVendorDir

              dx bundle --release
            '';

            installPhaseCommand = ''
              mkdir -p $out/
              cp -r ./target/dx/nju-schedule-ics/release/web/* $out/
            '';
        });
      docker = pkgs.dockerTools.buildImage {
        name = "nju-schedule-ics";
        config = {
          Cmd = [ "${server}/nju-schedule-ics" ];
        };
      };
    });

    githubActions = nix-github-actions.lib.mkGithubMatrix {
      checks = nixpkgs.lib.getAttrs ["x86_64-linux" "aarch64-linux" "aarch64-darwin"] self.packages;
    };
  };
}
