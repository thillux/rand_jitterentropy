# rand_jitterentropy

Rust bindings and wrappers around the CPU jitter based entropy source
[jitterentropy-library](https://github.com/smuellerDD/jitterentropy-library),
plus a small daemon which uses it to seed the Linux kernel CRNG.

## Crates

| Crate | Description |
| --- | --- |
| [`libjitterentropy-sys`](libjitterentropy-sys) | Raw FFI bindings, generated with bindgen |
| [`rand_jitterentropy`](rand_jitterentropy) | Safe wrapper implementing `rand_core::TryRng` |
| [`linux-crng-ioctl`](linux-crng-ioctl) | Wrappers for the Linux kernel RNG ioctls and its `/proc` interface |
| [`jitter-rngd`](rngd) | Daemon which seeds the kernel CRNG from the Jitter RNG |

## Requirements

* jitterentropy-library >= 3.4.0, including its headers
* Rust 2024 edition (1.85 or newer)
* clang/libclang, as bindgen generates the bindings at build time

The library is located with `pkg-config`. If that fails, the build falls back
to `JITTERENTROPY_LIB_DIR` (`/usr/lib` by default) and links
`-ljitterentropy`. Enable the `static` feature to link it statically:

```shell
cargo build -p rand_jitterentropy --features static
```

## Usage

```rust
use rand_core::TryRng;
use rand_jitterentropy::{JENT_FORCE_FIPS, JENT_MAX_MEMSIZE_32MB, RandJitterEntropy};

let mut rng = RandJitterEntropy::new()?;

let mut buf = [0u8; 32];
rng.try_fill_bytes(&mut buf)?;

// or with an explicit oversampling rate and flags
let mut rng = RandJitterEntropy::with_osr_and_flags(
    RandJitterEntropy::DEFAULT_OSR,
    JENT_FORCE_FIPS | JENT_MAX_MEMSIZE_32MB,
)?;

println!("jitterentropy {}", RandJitterEntropy::version_pretty());
```

Every fallible operation returns a `JitterEntropyError`, which covers both the
initialization errors and the runtime health test failures (RCT, APT, LAG,
intermittent and permanent) of the library.

Note that `jitterentropy` runs its self tests once per process: the first live
instance determines the `osr` and `flags` these tests run with, later instances
only pass them to their own entropy collector.

## API which depends on the jitterentropy version

The bindings and the wrapper adapt to the library they are built against, so
only what the underlying version provides is exported.

| API | Requires |
| --- | --- |
| `JENT_MAX_MEMSIZE_32kB` … `JENT_MAX_MEMSIZE_512MB` | >= 3.4.0 |
| `JENT_MAX_MEMSIZE_1kB` … `JENT_MAX_MEMSIZE_16kB` | >= 3.7.0 |
| `JENT_CACHE_ALL` | >= 3.7.0 |
| `JENT_NTG1` and the `ntg1` feature | >= 3.7.0 |
| `JENT_HASHLOOP_1` … `JENT_HASHLOOP_128` | >= 3.7.0 |
| `RandJitterEntropy::status()` | >= 3.7.0 |
| `RandJitterEntropy::secure_memory_supported()` | >= 3.7.0 |

Using the `ntg1` feature (AIS 20/31 NTG.1 compliance) with an older library is
rejected at compile time instead of silently dropping the flag.

`RandJitterEntropy::status()` returns the human readable status of an instance
as JSON: library version, health test state, runtime environment and the
configuration the instance runs with.

## jitter-rngd

`jitter-rngd` reads from the Jitter RNG, mixes the output with its own state
through SHA3-512 and hands the result to the kernel CRNG. Adding entropy
requires `CAP_SYS_ADMIN`.

```shell
# seed once, then exit
sudo jitter-rngd --oneshot

# seed every 30 seconds and force a CRNG reseed after each round
sudo jitter-rngd --seed-interval-s 30 --force-crng-reseed

# log level is controlled by env_logger
RUST_LOG=debug sudo -E jitter-rngd
```

## Development Setup (Nix-based)

Update dependencies:
```shell
nix flake update
```

Enter development shell with automatic dependency fetching:
```shell
nix develop .# --builders ''
```

Build the daemon as a static binary:
```shell
nix build .#rngd
```

Inside the development shell the usual cargo commands work:
```shell
cargo test --workspace
cargo clippy --workspace --all-targets
```

To build against a different jitterentropy version, point the build at it:
```shell
JITTERENTROPY_LIB_DIR=/path/to/lib \
BINDGEN_EXTRA_CLANG_ARGS=-I/path/to/include \
cargo test --workspace
```

## License

MIT, see [LICENSE.MIT](LICENSE.MIT).
