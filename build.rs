use std::env;
use std::path::PathBuf;

fn main() {
    // 1) Telling Cargo where to find the library at link time
    println!("cargo:rustc-link-search=native=./libs/eusign/shared");
    
    // 2) Tellinng Cargo which library to link (`libeuscp.so` in my case)
    println!("cargo:rustc-link-lib=dylib=euscp");

    // 3) Configuring bindgen
    let builder = bindgen::Builder::default()
        // The header that declares EULoad, EUUnload, EUGetInterface, etc.
        .header("libs/eusign/interface/header.h")
        
        // Ensures Cargo automatically rebuilds if the header changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_item("EULoad")
        .blocklist_item("EUGetInterface")
        .blocklist_item("EUUnload")
        ;

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings with bindgen");

    // 4) Write them to $OUT_DIR/bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
