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

    rustToolchain = eachSystem (pkgs: pkgs.rust-bin.stable.latest);
    name = "nju-schedule-ics";
    version = "0.9.0";
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
        ] ++
        (pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
          SystemConfiguration
        ])) ++
        (pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          openssl
        ]));

        RUST_SRC_PATH = "${
          rustToolchain.${pkgs.system}.rust-src
        }/lib/rustlib/src/rust/library";
        RUST_BACKTRACE = "1";
      });
    });

    packages = eachSystem (pkgs: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = name;
        inherit version;
        src = pkgs.lib.cleanSourceWith {
          filter = (path: type:
              (
                let
                  name = builtins.baseNameOf path;
                in
                  (builtins.match ".*src.*" path != null || name == "Cargo.toml" || name == "Cargo.lock") &&
                  builtins.match ".*\.DS_Store" path == null
              )
            );
          src = (pkgs.lib.cleanSource ./.);
        } ;

        cargoSha256 = "sha256-IAHdSEQZheG1gXwytmymnxoJVuJtxZho3/6n0RzGjb0=";
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

        copyToRoot = [ pkgs.cacert ];

        config = {
          Cmd = [ "${packages.${pkgs.system}.default}/bin/nju-schedule-ics" "--config" "/config.toml" ];
          Env = [
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            "RUST_BACKTRACE=1"
          ];
        };
      };
    });

    githubActions = nix-github-actions.lib.mkGithubMatrix {
      checks = nixpkgs.lib.getAttrs [ "x86_64-linux" "x86_64-darwin" ] self.packages;
    };
  };
}
