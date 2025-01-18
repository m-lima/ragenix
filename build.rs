trait AddPkg {
    fn add_pkg_config(&mut self, pkg: pkg_config::Library) -> &mut Self;
}

impl AddPkg for cc::Build {
    fn add_pkg_config(&mut self, pkg: pkg_config::Library) -> &mut Self {
        for p in pkg.include_paths {
            self.flag("-isystem").flag(p.to_str().unwrap());
        }
        for p in pkg.link_paths {
            self.flag(format!("-L{p:?}"));
        }
        for p in pkg.libs {
            self.flag(format!("-l{p}"));
        }
        for p in pkg.framework_paths {
            self.flag(format!("-F{p:?}"));
        }
        for p in pkg.frameworks {
            self.flag(format!("-framework {p}"));
        }
        self
    }
}

impl AddPkg for String {
    fn add_pkg_config(&mut self, pkg: pkg_config::Library) -> &mut Self {
        for p in pkg.include_paths {
            self.push(' ');
            self.push_str("-isystem");
            self.push(' ');
            self.push_str(p.to_str().unwrap());
        }
        for p in pkg.link_paths {
            self.push(' ');
            self.push_str(&format!("-L{p:?}"));
        }
        for p in pkg.libs {
            self.push(' ');
            self.push_str(&format!("-l{p}"));
        }
        for p in pkg.framework_paths {
            self.push(' ');
            self.push_str(&format!("-F{p:?}"));
        }
        for p in pkg.frameworks {
            self.push(' ');
            self.push_str(&format!("-framework {p}"));
        }
        self
    }
}

fn main() {
    let nix_expr = pkg_config::Config::new().probe("nix-expr").unwrap();

    println!("cargo::rerun-if-changed=ragenix.cc");
    generate_outputs(
        cc::Build::new()
            .file("ragenix.cc")
            .cpp(true)
            .add_pkg_config(nix_expr)
            .shared_flag(true)
            .std("c++20"),
    );
}

fn generate_outputs(builder: &cc::Build) {
    use std::io::Write;

    builder.compile("ragenix");
    let command = builder.get_compiler().to_command();

    let mut file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("compile_commands.json")
        .unwrap();

    file.write_all(
        concat!(
            r#"[{"directory":""#,
            env!("CARGO_MANIFEST_DIR"),
            r#"","file":""#,
            env!("CARGO_MANIFEST_DIR"),
            r#"/ragenix.cc","command":""#
        )
        .as_bytes(),
    )
    .unwrap();

    file.write_all(command.get_program().as_encoded_bytes())
        .unwrap();
    for arg in command.get_args() {
        file.write_all(b" ").unwrap();

        let bytes = arg.as_encoded_bytes();
        let mut chunks = bytes.split(|b| *b == b'"');
        let Some(first) = chunks.next() else {
            continue;
        };

        file.write_all(first).unwrap();
        for rest in chunks {
            file.write_all(b"\\\"").unwrap();
            file.write_all(rest).unwrap();
        }
    }
    file.write_all(br#""}]"#).unwrap();
}
