#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::all)]
#![allow(warnings)]

include!(concat!(env!("OUT_DIR"), "/jitterentropy-bindings.rs"));

// JENT_MAX_MEMSIZE_TO_FLAGS()/JENT_FLAGS_TO_MAX_MEMSIZE() are function-like
// macros, which bindgen cannot translate. Provide them as const functions.

/// Encode a maximum memory size id into the corresponding flag bits.
#[must_use]
pub const fn JENT_MAX_MEMSIZE_TO_FLAGS(val: u32) -> u32 {
    val << JENT_FLAGS_TO_MEMSIZE_SHIFT
}

/// Decode the maximum memory size id from the given flags.
#[must_use]
pub const fn JENT_FLAGS_TO_MAX_MEMSIZE(val: u32) -> u32 {
    val >> JENT_FLAGS_TO_MEMSIZE_SHIFT
}

// Same for JENT_HASHLOOP_TO_FLAGS()/JENT_FLAGS_TO_HASHLOOP(), which only
// exist in jitterentropy versions after 3.7.0.

/// Encode a hash loop count id into the corresponding flag bits.
#[cfg(jent_hashloop)]
#[must_use]
pub const fn JENT_HASHLOOP_TO_FLAGS(val: u32) -> u32 {
    val << JENT_FLAGS_TO_HASHLOOP_SHIFT
}

/// Decode the hash loop count id from the given flags.
#[cfg(jent_hashloop)]
#[must_use]
pub const fn JENT_FLAGS_TO_HASHLOOP(val: u32) -> u32 {
    (val >> JENT_FLAGS_TO_HASHLOOP_SHIFT) & (JENT_MAX_HASHLOOP_MASK >> JENT_FLAGS_TO_HASHLOOP_SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_memsize_flag_conversion() {
        assert_eq!(
            JENT_MAX_MEMSIZE_TO_FLAGS(JENT_FLAGS_TO_MAX_MEMSIZE(JENT_MAX_MEMSIZE_512MB)),
            JENT_MAX_MEMSIZE_512MB
        );
        assert_eq!(
            JENT_FLAGS_TO_MAX_MEMSIZE(JENT_MAX_MEMSIZE_512MB),
            JENT_FLAGS_TO_MAX_MEMSIZE(JENT_MAX_MEMSIZE_256MB) + 1
        );
        assert_eq!(JENT_MAX_MEMSIZE_MAX & !JENT_MAX_MEMSIZE_MASK, 0);
    }

    #[cfg(jent_hashloop)]
    #[test]
    fn hashloop_flag_conversion() {
        assert_eq!(
            JENT_HASHLOOP_TO_FLAGS(JENT_FLAGS_TO_HASHLOOP(JENT_HASHLOOP_64)),
            JENT_HASHLOOP_64
        );
        assert_eq!(
            JENT_FLAGS_TO_HASHLOOP(JENT_HASHLOOP_128),
            JENT_FLAGS_TO_HASHLOOP(JENT_HASHLOOP_64) + 1
        );
        assert_eq!(JENT_MAX_HASHLOOP & !JENT_MAX_HASHLOOP_MASK, 0);
    }
}
