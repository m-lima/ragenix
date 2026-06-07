fn main() {
    let nix_expr_c = pkg_config::Config::new().probe("nix-expr-c").unwrap();

    println!("cargo::rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("nix_.*")
        .allowlist_type("nix_.*")
        .allowlist_type("NIX_.*")
        .allowlist_type("ValueType")
        .allowlist_type("EvalState")
        .allowlist_type("PrimOp")
        .allowlist_type("PrimOpFun")
        .allowlist_type("BindingsBuilder")
        .allowlist_type("ListBuilder")
        .allowlist_var("NIX_.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    for path in &nix_expr_c.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.display()));
    }

    let out = std::path::PathBuf::from(
        std::env::var("OUT_DIR").expect("Could not find OUT_DIR during build time"),
    );
    builder
        .generate()
        .expect("Could not generate nix C API bindings")
        .write_to_file(out.join("nix.rs"))
        .expect("Could not write nix.rs");

    // match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
    //     Ok("macos") => {
    //         println!("cargo:rustc-link-arg=-undefined");
    //         println!("cargo:rustc-link-arg=dynamic_lookup");
    //     }
    //     Ok("linux") => println!("cargo:rustc-link-arg=-Wl,--unresolved-symbols=ignore-all"),
    //     _ => {}
    // }
}
