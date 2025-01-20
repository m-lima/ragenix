#![deny(warnings, clippy::pedantic)]

mod error;
mod string;

type Result<T = ()> = core::result::Result<T, error::Error>;

extern "C" {
    fn cpp_entry();
}

#[no_mangle]
pub extern "C" fn nix_plugin_entry() {
    unsafe { cpp_entry() }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn decrypt(
    path: *const core::ffi::c_char,
    pub_key: *const core::ffi::c_char,
    code: *mut core::ffi::c_uchar,
) -> string::String {
    fn decrypt<P: AsRef<std::path::Path>>(
        path: P,
        pub_key: &core::ffi::CStr,
    ) -> Result<string::String> {
        let pub_key = pub_key.to_str().map(String::from)?;
        let mut guard = age::cli_common::StdinGuard::new(true);
        // TODO: This needs to print to the CLI
        let identities = age::cli_common::read_identities(vec![pub_key], None, &mut guard)?;

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

    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(
        unsafe { core::ffi::CStr::from_ptr(path) }.to_bytes(),
    );
    let pub_key = unsafe { core::ffi::CStr::from_ptr(pub_key) };

    match decrypt(path, pub_key) {
        Ok(string) => string,
        Err(err) => {
            unsafe { code.write(err.code()) };
            err.message()
        }
    }
}

#[no_mangle]
pub extern "C" fn dealloc(string: string::String) {
    string.dealloc();
}
