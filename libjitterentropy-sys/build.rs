use std::{env::var, path::PathBuf};

use bindgen::Builder;

fn main() {
    #[cfg(feature = "openssl")]
    pkg_config::Config::new().probe("libcrypto").unwrap();

    let bindings = Builder::default()
        .header("jitterentropy-include.h")
        .generate()
        .unwrap();
    let mut bindings_path = PathBuf::from(var("OUT_DIR").unwrap());
    bindings_path.push("jitterentropy-bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Could not write bindings to file");

    println!("cargo:rustc-link-lib=jitterentropy");
}
