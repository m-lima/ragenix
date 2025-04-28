#![deny(warnings, clippy::pedantic)]

mod error;
mod string;

type Result<T = ()> = core::result::Result<T, error::Error>;

unsafe extern "C" {
    fn cpp_entry();
}

#[unsafe(no_mangle)]
pub extern "C" fn nix_plugin_entry() {
    unsafe { cpp_entry() }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn decrypt(
    key: *const core::ffi::c_char,
    path: *const core::ffi::c_char,
    code: *mut core::ffi::c_uchar,
) -> string::String {
    fn decrypt<P: AsRef<std::path::Path>>(
        key: &core::ffi::CStr,
        path: P,
    ) -> Result<string::String> {
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

        Ok(out.into())
    }

    unsafe { code.write(0) };

    let key = unsafe { core::ffi::CStr::from_ptr(key) };
    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(
        unsafe { core::ffi::CStr::from_ptr(path) }.to_bytes(),
    );

    match decrypt(key, path) {
        Ok(string) => string,
        Err(err) => {
            unsafe { code.write(err.code()) };
            err.message()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc(string: string::String) {
    string.dealloc();
}
