struct Pkg {
    lib: pkg_config::Library,
    error_msg: &'static str,
}

trait AddPkg {
    fn add_pkg_config(self, pkg: Pkg) -> Self;
}

impl AddPkg for bindgen::Builder {
    fn add_pkg_config(self, pkg: Pkg) -> Self {
        self.clang_args(
            pkg.lib
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_str().expect(pkg.error_msg),)),
        )
    }
}

fn main() {
    macro_rules! find_pkg {
        ($name: literal) => {
            Pkg {
                lib: pkg_config::probe_library($name).expect(concat!(
                    "Could not find `",
                    $name,
                    "` lib"
                )),
                error_msg: concat!("Could not represent include path for `", $name, "` lib"),
            }
        };
    }

    let nix_expr = find_pkg!("nix-expr");
    let nix_store = find_pkg!("nix-store");
    let nix_main = find_pkg!("nix-main");

    let bindings = bindgen::Builder::default()
        .add_pkg_config(nix_expr)
        .add_pkg_config(nix_store)
        .add_pkg_config(nix_main)
        .header("include.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not defined"));

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings");
}
