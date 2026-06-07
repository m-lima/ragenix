#![deny(warnings, clippy::pedantic)]

mod error;
mod primop;

mod nix {
    #![allow(
        dead_code,
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        clippy::all
    )]
    include!(concat!(env!("OUT_DIR"), "/nix.rs"));
}

fn decrypt(key: &core::ffi::CStr, path: &std::ffi::OsStr) -> Result<String, error::Error> {
    let key = key.to_str().map(String::from)?;
    let mut guard = age::cli_common::StdinGuard::new(true);
    // TODO: This needs to print to the CLI
    let identities = age::cli_common::read_identities(vec![key], None, &mut guard)?;

    let decryptor = std::fs::File::open(path)
        .map_err(Into::into)
        .map(std::io::BufReader::new)
        .map(age::armor::ArmoredReader::new)
        .and_then(error::wrap(age::Decryptor::new))?;

    let mut reader = decryptor.decrypt(identities.iter().map(|b| &**b))?;
    let mut out = String::new();
    std::io::Read::read_to_string(&mut reader, &mut out)?;

    Ok(out)
}

#[unsafe(no_mangle)]
pub extern "C" fn nix_plugin_entry() {
    primop::register();
}
