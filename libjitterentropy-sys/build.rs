use std::{
    env::var,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bindgen::{
    Builder,
    callbacks::{EnumVariantValue, IntKind, ParseCallbacks},
};

/// Flags which are plain macros, but do not exist in every supported
/// jitterentropy version (>= 3.4.0).
const OPTIONAL_FLAGS: [&str; 2] = ["JENT_CACHE_ALL", "JENT_NTG1"];

/// Further API which does not exist in every supported jitterentropy version.
const OPTIONAL_ITEMS: [&str; 3] = [
    "jent_status",
    "jent_secure_memory_supported",
    // the version components are only in the header since 3.6.0
    "JENT_MAJVERSION",
];

/// Collects the flags whose existence depends on the jitterentropy version,
/// while bindgen walks the header. The `JENT_MAX_MEMSIZE_*`/`JENT_HASHLOOP_*`
/// enums in `jitterentropy-include.h` carry a `RUST_` prefix to avoid a clash
/// with the macros they mirror, which is stripped here again.
#[derive(Clone, Debug, Default)]
struct FlagCollector(Arc<Mutex<Vec<String>>>);

impl FlagCollector {
    fn push(&self, flag: &str) {
        self.0
            .lock()
            .expect("Flag list is poisoned")
            .push(flag.to_string());
    }

    fn flags(&self) -> Vec<String> {
        self.0.lock().expect("Flag list is poisoned").clone()
    }
}

impl ParseCallbacks for FlagCollector {
    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        let name = variant_name.strip_prefix("RUST_")?;
        self.push(name);
        Some(name.to_string())
    }

    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        if OPTIONAL_FLAGS.contains(&name) {
            self.push(name);
        }
        // keep bindgen's own type deduction
        None
    }
}

fn main() {
    println!("cargo:rerun-if-changed=jitterentropy-include.h");
    println!("cargo:rerun-if-env-changed=JITTERENTROPY_LIB_DIR");

    let statik = cfg!(feature = "static");

    let collector = FlagCollector::default();
    let bindings = Builder::default()
        .header("jitterentropy-include.h")
        .allowlist_function("jent_.*")
        .allowlist_type("rand_data")
        .allowlist_type("jent_max_memsize")
        .allowlist_type("jent_hashloop")
        .allowlist_var("JENT_.*")
        .prepend_enum_name(false)
        .parse_callbacks(Box::new(collector.clone()))
        .generate()
        .expect("Could not generate jitterentropy bindings");
    let mut bindings_path = PathBuf::from(var("OUT_DIR").unwrap());
    bindings_path.push("jitterentropy-bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Could not write bindings to file");

    // Hand the version dependent flags to dependent crates (as
    // DEP_JITTERENTROPY_FLAGS), so that they can re-export exactly those, and
    // gate our own hash loop helpers.
    let flags = collector.flags();
    println!("cargo:flags={}", flags.join(","));

    println!("cargo:rustc-check-cfg=cfg(jent_hashloop)");
    if flags.iter().any(|flag| flag.starts_with("JENT_HASHLOOP_")) {
        println!("cargo:rustc-cfg=jent_hashloop");
    }

    // Same for the rest of the optional API (as DEP_JITTERENTROPY_ITEMS).
    let generated = fs::read_to_string(&bindings_path).expect("Could not read generated bindings");
    let items: Vec<&str> = OPTIONAL_ITEMS
        .into_iter()
        .filter(|item| generated.contains(item))
        .collect();
    println!("cargo:items={}", items.join(","));

    let found_jitterentropy = pkg_config::Config::new()
        .atleast_version("3.4.0")
        .statik(statik)
        .probe("jitterentropy");

    if found_jitterentropy.is_err() {
        let lib_path =
            std::env::var("JITTERENTROPY_LIB_DIR").unwrap_or_else(|_| "/usr/lib".to_string());

        println!("cargo:rustc-link-search=native={lib_path}");
        if statik {
            println!("cargo:rustc-link-lib=static=jitterentropy");
        } else {
            println!("cargo:rustc-link-lib=jitterentropy");
        }
    }
}
