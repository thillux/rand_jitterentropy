# libjitterentropy-sys

Raw FFI bindings to the C library
[jitterentropy-library](https://github.com/smuellerDD/jitterentropy-library),
see also its [website](https://www.chronox.de/jent). For a safe interface use
the `rand_jitterentropy` crate.

The bindings are generated with bindgen at build time, so they always match the
library version they are built against. jitterentropy >= 3.4.0 is supported.

## Building

The library is located with `pkg-config`. If that fails, the build falls back
to `JITTERENTROPY_LIB_DIR` (`/usr/lib` by default) and links
`-ljitterentropy`.

| Environment variable | Purpose |
| --- | --- |
| `JITTERENTROPY_LIB_DIR` | Directory of `libjitterentropy`, used when `pkg-config` finds nothing |
| `BINDGEN_EXTRA_CLANG_ARGS` | Extra clang arguments, e.g. `-I` for the headers |

The `static` feature links the library statically.

## What is exported

* all `jent_*` functions and the opaque `rand_data` type
* all `JENT_*` constants of the header

`JENT_MAX_MEMSIZE_*` and `JENT_HASHLOOP_*` are function-like macro invocations
in `jitterentropy.h`, which bindgen cannot evaluate. `jitterentropy-include.h`
mirrors them into enums, so their values are picked up from the header instead
of being hardcoded here - they differ between versions. The macros converting
between the flag bits and the ids are provided as const functions:

```rust
use libjitterentropy_sys::jitterentropy::{
    JENT_FLAGS_TO_MAX_MEMSIZE, JENT_MAX_MEMSIZE_32MB, JENT_MAX_MEMSIZE_TO_FLAGS,
};
```

Which of these exist depends on the jitterentropy version:

| API | Requires |
| --- | --- |
| `JENT_MAX_MEMSIZE_32kB` … `JENT_MAX_MEMSIZE_512MB` | >= 3.4.0 |
| `JENT_MAJVERSION`, `JENT_MINVERSION`, `JENT_PATCHLEVEL` | >= 3.6.0 |
| `JENT_MAX_MEMSIZE_1kB` … `JENT_MAX_MEMSIZE_16kB` | >= 3.7.0 |
| `JENT_CACHE_ALL` | >= 3.7.0 |
| `JENT_NTG1` | >= 3.7.0 |
| `JENT_HASHLOOP_1` … `JENT_HASHLOOP_128` | >= 3.7.0 |
| `jent_status()` | >= 3.7.0 |
| `jent_secure_memory_supported()` | >= 3.7.0 |

Dependent crates do not have to guess: the build script reports what it found
through the `links` metadata.

| Metadata | Content |
| --- | --- |
| `DEP_JITTERENTROPY_FLAGS` | Names of the version dependent flag constants |
| `DEP_JITTERENTROPY_ITEMS` | Names of the version dependent functions and constants of the rest of the API |

## Safety

This crate is a thin FFI layer, every function is `unsafe`. The entropy
collector returned by `jent_entropy_collector_alloc` has to be freed with
`jent_entropy_collector_free`, and `jent_read_entropy_safe` may replace the
collector it is given.
