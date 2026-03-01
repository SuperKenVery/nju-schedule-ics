---
name: update-dioxus-fork
description: Update dioxus to use a custom fork/branch instead of a crates.io release. Use when you need unreleased fixes or patches.
---

# Update Dioxus (custom fork/branch)

Update dioxus to use a custom Git fork instead of a crates.io release. This is for when you need unreleased fixes.

## Steps

### 1. Gather fork info

From the fork repository, collect:
- GitHub owner/repo (e.g. `SuperKenVery/dioxus`)
- Branch name (e.g. `fix/hash-filename-when-wasm-split`)
- Full commit hash (`git rev-parse HEAD`)
- Dioxus version in the fork (`grep '^version' Cargo.toml` in the workspace root)

Verify the branch is pushed to GitHub:
```bash
git ls-remote origin BRANCH_NAME
```

### 2. Update Cargo.toml

Change all dioxus crates to git dependencies:

```toml
dioxus = { git = "https://github.com/OWNER/dioxus.git", branch = "BRANCH", features = ["fullstack", "router"] }
dioxus-cli-config = { git = "https://github.com/OWNER/dioxus.git", branch = "BRANCH" }
dioxus-logger = { git = "https://github.com/OWNER/dioxus.git", branch = "BRANCH" }
```

### 3. Update Cargo.lock

Run `cargo update` to refresh the lock file.

### 4. Check wasm-bindgen version

```bash
grep '^name = "wasm-bindgen"' -A 1 Cargo.lock
```

### 5. Update flake.nix — dioxus-cli

Replace the dioxus-cli override to build from GitHub source using `fetchFromGitHub` + `fetchCargoVendor`:

```nix
dioxus-cli = eachSystem (pkgs: pkgs.dioxus-cli.overrideAttrs (oldAttrs: {
  version = "VERSION-DESCRIPTION";

  src = pkgs.fetchFromGitHub {
    owner = "OWNER";
    repo = "dioxus";
    rev = "FULL_COMMIT_HASH";
    hash = "HASH";
  };

  cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
    inherit (oldAttrs) pname;
    version = "VERSION-DESCRIPTION";
    src = pkgs.fetchFromGitHub {
      owner = "OWNER";
      repo = "dioxus";
      rev = "FULL_COMMIT_HASH";
      hash = "HASH";
    };
    hash = "CARGO_VENDOR_HASH";
  };

  postPatch = "";
}));
```

`Dioxus.lock` is NOT needed with this approach — delete it if it exists.

#### Getting the hashes

Source hash:
```bash
nix-prefetch-url --unpack "https://github.com/OWNER/dioxus/archive/FULL_COMMIT_HASH.tar.gz"
# Convert to SRI:
nix hash convert --hash-algo sha256 --to sri HASH_FROM_ABOVE
```

Cargo vendor hash (use empty hash `""` and read the "got:" line):
```bash
nix build --impure --expr '
  let pkgs = import (builtins.getFlake "nixpkgs") {};
  in pkgs.rustPlatform.fetchCargoVendor {
    src = pkgs.fetchFromGitHub {
      owner = "OWNER";
      repo = "dioxus";
      rev = "FULL_COMMIT_HASH";
      hash = "SOURCE_HASH_FROM_ABOVE";
    };
    pname = "dioxus-cli";
    version = "VERSION";
    hash = "";
  }
' 2>&1 | grep "got:"
```

### 6. Update flake.nix — wasm-bindgen-cli

If the wasm-bindgen version changed (step 4), update the hashes. The version is auto-detected from `Cargo.lock`, but hashes must be updated manually.

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

### 7. Verify

Run `nix flake check` or attempt a build to verify everything resolves correctly.

### Reverting back to crates.io

When the fix is merged upstream and released, revert to a crates.io release using the `update-dioxus` skill. Key changes:
- Cargo.toml: git deps → version strings
- flake.nix: `fetchFromGitHub` → `importCargoLock` with `Dioxus.lock` (or remove override if nixpkgs matches)
- Re-create `Dioxus.lock` from the dioxus repo's Cargo.lock at the release tag
