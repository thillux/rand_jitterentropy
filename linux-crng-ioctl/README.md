# linux-crng-ioctl

Rust wrappers for the ioctls of the Linux kernel CRNG and for its
`/proc/sys/kernel/random/*` interface. Linux only.

## ioctl

The `ioctl` module operates on `/dev/random`:

| Function | ioctl | Privileges |
| --- | --- | --- |
| `get_ent_cnt()` | `RNDGETENTCNT` | none |
| `add_to_ent_cnt(bits)` | `RNDADDTOENTCNT` | `CAP_SYS_ADMIN` |
| `add_randomness_to_kernel(entropy, bits)` | `RNDADDENTROPY` | `CAP_SYS_ADMIN` |
| `clear_entropy_count()` | `RNDZAPENTCNT` | `CAP_SYS_ADMIN` |
| `clear_pool()` | `RNDCLEARPOOL` | `CAP_SYS_ADMIN` |
| `force_kernel_crng_reseed()` | `RNDRESEEDCRNG` | `CAP_SYS_ADMIN` |

```rust
use linux_crng_ioctl::ioctl::{add_randomness_to_kernel, get_ent_cnt};

let entropy = [0u8; 32];
add_randomness_to_kernel(&entropy, (entropy.len() * 8) as u32)?;

println!("entropy count: {}", get_ent_cnt()?);
```

Only credit entropy which really is entropy: the amount of bits passed to
`add_randomness_to_kernel` is what the kernel adds to its entropy count. It is
rejected if it exceeds `entropy.len() * 8`, and one call writes at most 2 kB.

## proc

The `proc` module reads the read-only files below
`/proc/sys/kernel/random/`: `boot_id()`, `uuid()`, `entropy_avail()`,
`poolsize()`, `urandom_min_reseed_secs()` and `write_wakeup_threshold()`.

```rust
use linux_crng_ioctl::proc::{entropy_avail, poolsize};

println!("{} of {} bits available", entropy_avail()?, poolsize()?);
```

## Tests

The test suite skips the cases which need `CAP_SYS_ADMIN` when it does not have
them, so `cargo test` works as an ordinary user.
