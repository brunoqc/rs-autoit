extern crate libbindgen;
use libbindgen::Builder;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=AutoItX3_x64_DLL");

    let bindings = Builder::default()
        .no_unstable_rust()
        .header("wrapper.hpp")
        .no_unstable_rust()
        .whitelisted_function("AU3_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
