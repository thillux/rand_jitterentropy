# rand_jitterentropy

Safe wrapper around the `libjitterentropy-sys` crate, which implements
`rand_core::TryRng` for the CPU jitter based entropy source
[jitterentropy-library](https://github.com/smuellerDD/jitterentropy-library).

The RNG is initialized in FIPS mode (`JENT_FORCE_FIPS`) with an oversampling
rate of 6. With the `ntg1` feature enabled, it additionally complies with the
NTG.1 requirements of AIS 20/31 (2024).

jitterentropy >= 3.4.0 is supported. The library is located with `pkg-config`;
if that fails, the build falls back to the directory in `JITTERENTROPY_LIB_DIR`
(`/usr/lib` by default) and links `-ljitterentropy`. Its headers have to be
found by bindgen, use `BINDGEN_EXTRA_CLANG_ARGS=-I/path/to/include` if they are
in a non-standard location.

## Usage

```rust
use rand_core::TryRng;
use rand_jitterentropy::RandJitterEntropy;

fn main() -> Result<(), rand_jitterentropy::JitterEntropyError> {
    let mut rng = RandJitterEntropy::new()?;

    let mut buf = [0u8; 32];
    rng.try_fill_bytes(&mut buf)?;

    let x = rng.try_next_u64()?;
    println!("{x}");
    Ok(())
}
```

The oversampling rate and the flags of an instance can be chosen explicitly:

```rust
use rand_jitterentropy::{JENT_FORCE_FIPS, JENT_MAX_MEMSIZE_32MB, RandJitterEntropy};

let rng = RandJitterEntropy::with_osr_and_flags(
    RandJitterEntropy::DEFAULT_OSR,
    JENT_FORCE_FIPS | JENT_MAX_MEMSIZE_32MB,
)?;
```

Note that the library runs its self tests once per process: the first live
instance determines the `osr` and `flags` these tests run with, later instances
only pass them to their own entropy collector.

Every fallible operation returns a `JitterEntropyError`, which covers the
initialization errors as well as the runtime health test failures (RCT, APT,
LAG, intermittent and permanent) of the library. A read which loses its entropy
collector - jitterentropy < 3.7.0 frees it before reallocating it on an
intermittent health test failure - is reported as an error, the instance takes
itself up again on the next read.

## Information about the library

```rust
use rand_jitterentropy::RandJitterEntropy;

println!("jitterentropy {}", RandJitterEntropy::version_pretty());
println!("version number: {}", RandJitterEntropy::version());

// jitterentropy >= 3.7.0
let rng = RandJitterEntropy::new()?;
println!("secure memory: {}", RandJitterEntropy::secure_memory_supported());
println!("{}", rng.status()?);
```

`status()` returns the state of an instance as JSON: library version, health
test state, runtime environment and the configuration it runs with.

## Flags

Besides the flags of the base API (`JENT_FORCE_FIPS`, `JENT_NTG1`,
`JENT_DISABLE_MEMORY_ACCESS`, `JENT_FORCE_INTERNAL_TIMER`,
`JENT_DISABLE_INTERNAL_TIMER`, `JENT_CACHE_ALL`) the memory size
(`JENT_MAX_MEMSIZE_*`) and hash loop count (`JENT_HASHLOOP_*`) flags are
re-exported. Which of them exist depends on the version of the jitterentropy
library built against:

| API | Requires |
| --- | --- |
| `JENT_MAX_MEMSIZE_32kB` … `JENT_MAX_MEMSIZE_512MB` | >= 3.4.0 |
| `JENT_MAX_MEMSIZE_1kB` … `JENT_MAX_MEMSIZE_16kB` | >= 3.7.0 |
| `JENT_CACHE_ALL` | >= 3.7.0 |
| `JENT_NTG1` and the `ntg1` feature | >= 3.7.0 |
| `JENT_HASHLOOP_1` … `JENT_HASHLOOP_128` | >= 3.7.0 |
| `RandJitterEntropy::status()` | >= 3.7.0 |
| `RandJitterEntropy::secure_memory_supported()` | >= 3.7.0 |

## Features

- `ntg1` — request NTG.1 compliant operation (`JENT_NTG1`), needs
  jitterentropy >= 3.7.0 and is rejected at compile time otherwise
- `static` — link `libjitterentropy` statically
