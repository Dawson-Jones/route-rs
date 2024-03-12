use std::path::PathBuf;


fn main() {
    let docs_builder = std::env::var("DOCS_RS").is_ok();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "macos" && !docs_builder {
        build_macos();
    }
}


fn build_macos() {
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");


    // let pwd = PathBuf::from(std::env::var("PWD").unwrap());
    // let out_path = pwd.join("src/macos/extern_c.rs");
    let out_dir= PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("extern_c.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}