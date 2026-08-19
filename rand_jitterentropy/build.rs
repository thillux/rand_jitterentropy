use std::{env::var, fs, path::PathBuf};

/// Parts of the jitterentropy API which do not exist in every supported
/// version (>= 3.4.0), see libjitterentropy-sys.
const OPTIONAL_CFGS: [&str; 4] = [
    "jent_status",
    "jent_secure_memory_supported",
    "jent_majversion",
    "jent_ntg1",
];

fn main() {
    // Which JENT_MAX_MEMSIZE_*/JENT_HASHLOOP_*/JENT_CACHE_ALL/JENT_NTG1 flags
    // exist depends on the jitterentropy version. libjitterentropy-sys reports
    // the ones it found, re-export exactly those.
    let flags = var("DEP_JITTERENTROPY_FLAGS").unwrap_or_default();
    let reexport = if flags.is_empty() {
        String::new()
    } else {
        format!(
            "/// Maximum memory size (`JENT_MAX_MEMSIZE_*`), hash loop count\n\
             /// (`JENT_HASHLOOP_*`) and cache size flags, which can be passed to\n\
             /// [`RandJitterEntropy::with_osr_and_flags`] as well. Which of them exist\n\
             /// depends on the version of the jitterentropy library built against.\n\
             pub use libjitterentropy_sys::jitterentropy::{{\n    {},\n}};\n",
            flags.replace(',', ",\n    ")
        )
    };

    let mut flags_path = PathBuf::from(var("OUT_DIR").unwrap());
    flags_path.push("flags.rs");
    fs::write(&flags_path, reexport).expect("Could not write flag re-exports to file");

    // Same for the items our own API is built on, which become cfgs named
    // after them.
    let items = var("DEP_JITTERENTROPY_ITEMS")
        .unwrap_or_default()
        .to_lowercase();
    for cfg in OPTIONAL_CFGS {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    for item in items.split(',').filter(|item| OPTIONAL_CFGS.contains(item)) {
        println!("cargo:rustc-cfg={item}");
    }
    if flags.split(',').any(|flag| flag == "JENT_NTG1") {
        println!("cargo:rustc-cfg=jent_ntg1");
    }
}
