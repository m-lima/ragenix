fn main() {
    let bindings = bindgen::Builder::default()
        .header("include.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not defined"));

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings");
}
