{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.url = "nixpkgs";
    };
  };

  outputs = {
    self,
    systems,
    nixpkgs,
    treefmt-nix,
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

    rustToolchain = eachSystem (pkgs: pkgs.rust-bin.stable.latest);
    name = "nju-schedule-ics";
    version = "0.9.0";
    treefmtEval = eachSystem (pkgs: treefmt-nix.lib.evalModule pkgs ./treefmt.nix);
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
          rustToolchain.${pkgs.system}.default
          rust-analyzer-unwrapped
          cargo
          # pkg-config
          # openssl
        ];
        RUST_SRC_PATH = "${
          rustToolchain.${pkgs.system}.rust-src
        }/lib/rustlib/src/rust/library";
      });
    });

    packages = eachSystem (pkgs: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = name;
        inherit version;
        src = pkgs.lib.cleanSource ./.;
        cargoSha256 = "sha256-yzm14wCqxuf75KoRHoYRAErWTkjPZXmWmzDYZq+xJaY=";
        buildInputs = []  ++
          (pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            SystemConfiguration
          ])) ++
          (pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
            openssl
          ]));
        nativeBuildInputs = (pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          pkg-config
        ]));
        doCheck=false;
      };

      docker = pkgs.dockerTools.buildImage {
        inherit name;

        config.Cmd = [ "${packages.${pkgs.system}.default}/bin/nju-schedule-ics" ];
      };
    });

    formatter = eachSystem (pkgs: treefmtEval.${pkgs.system}.config.build.wrapper);

    checks = eachSystem (pkgs: {
      formatting = treefmtEval.${pkgs.system}.config.build.check self;
    });
  };
}
