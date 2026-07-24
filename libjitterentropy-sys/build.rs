use std::{env::var, path::PathBuf};

use bindgen::Builder;

fn main() {
    println!("cargo:rerun-if-changed=jitterentropy-include.h");
    println!("cargo:rerun-if-env-changed=JITTERENTROPY_LIB_DIR");

    let statik = cfg!(feature = "static");

    let bindings = Builder::default()
        .header("jitterentropy-include.h")
        .allowlist_function("jent_.*")
        .allowlist_type("rand_data")
        .allowlist_var("JENT_.*")
        .generate()
        .expect("Could not generate jitterentropy bindings");
    let mut bindings_path = PathBuf::from(var("OUT_DIR").unwrap());
    bindings_path.push("jitterentropy-bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Could not write bindings to file");

    let found_jitterentropy = pkg_config::Config::new()
        .atleast_version("3.7")
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
