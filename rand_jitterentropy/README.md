# rand_jitterentropy

Provides adapter for `libjitterentropy-sys` to `rand_core` crate.

The RNG is initialized in FIPS mode (`JENT_FORCE_FIPS`) with an oversampling
rate of 6. With the `ntg1` feature enabled, it additionally complies with the
NTG.1 requirements of AIS 20/31 (2024).

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

## Features

- `ntg1` — request NTG.1 compliant operation (`JENT_NTG1`)
- `static` — link `libjitterentropy` statically
