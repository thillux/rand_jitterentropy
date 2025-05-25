These crates provide Rust wrappers around [jitterentropy-library](https://github.com/smuellerDD/jitterentropy-library).

## Development Setup (Nix-based)

Update dependencies:
```shell
nix flake update
```

Enter development shell with automatic dependency fetching:
```shell
nix develop .# --builders ''
```