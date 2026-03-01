---
name: update-dioxus
description: Update dioxus to a new release version from crates.io. Use when bumping dioxus to an official release.
---

# Update Dioxus (crates.io release)

Update dioxus and related crates to a new official release from crates.io.

## Steps

### 1. Update Cargo.toml

Update the version for all dioxus crates in `Cargo.toml`:
- `dioxus`
- `dioxus-cli-config`
- `dioxus-logger`

These should use plain version strings (e.g. `version = "0.7.3"`), not git dependencies.

### 2. Update Cargo.lock

Run `cargo update` to refresh the lock file.

### 3. Check wasm-bindgen version

```bash
grep '^name = "wasm-bindgen"' -A 1 Cargo.lock
```

Note the version — you'll need it for flake.nix.

### 4. Update flake.nix — dioxus-cli

The `dioxus-cli` definition should override the nixpkgs package with a `Dioxus.lock` file (the lock file for building dioxus-cli from source). This is needed when the nixpkgs version doesn't match our target version.

If nixpkgs already has the exact version we want (`nix eval nixpkgs#dioxus-cli.version`), the override can be simplified or removed.

Otherwise, obtain a `Dioxus.lock` for the target version. You can get it from the dioxus repo's `Cargo.lock` at the corresponding tag. The current pattern:

```nix
dioxus-cli = eachSystem (pkgs: pkgs.dioxus-cli.overrideAttrs (oldAttrs: {
  postPatch = ''
    rm Cargo.lock
    cp ${./Dioxus.lock} Cargo.lock
  '';

  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ./Dioxus.lock;
  };
}));
```

### 5. Update flake.nix — wasm-bindgen-cli

If the wasm-bindgen version changed (step 3), update the hashes in the `wasm-bindgen-cli` definition. The version is auto-detected from `Cargo.lock`, but the hashes need manual updating.

To get the new hashes, use nix with empty hashes and read the "got:" output:

```bash
# fetchCrate hash
nix build --impure --expr '
  let pkgs = import (builtins.getFlake "nixpkgs") {};
  in pkgs.fetchCrate {
    pname = "wasm-bindgen-cli";
    version = "NEW_VERSION";
    hash = "";
  }
' 2>&1 | grep "got:"

# fetchCargoVendor hash
nix build --impure --expr '
  let pkgs = import (builtins.getFlake "nixpkgs") {};
      src = pkgs.fetchCrate {
        pname = "wasm-bindgen-cli";
        version = "NEW_VERSION";
        hash = "HASH_FROM_ABOVE";
      };
  in pkgs.rustPlatform.fetchCargoVendor {
    inherit src;
    pname = "wasm-bindgen-cli";
    version = "NEW_VERSION";
    hash = "";
  }
' 2>&1 | grep "got:"
```

Update both hashes in the `wasm-bindgen-cli` block in flake.nix.

### 6. Verify

Run `nix flake check` or attempt a build to verify everything resolves correctly.
