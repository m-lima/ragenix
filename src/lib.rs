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
    fn decrypt<P: AsRef<std::path::Path>>(path: P, _pub_key: &str) -> Result<string::String> {
        let path = path.as_ref();

        let mut file = std::fs::File::open(path)?;
        let mut buffer = std::fs::metadata(path)
            .ok()
            .and_then(|m| usize::try_from(m.len()).ok())
            .map_or_else(Vec::new, Vec::with_capacity);

        std::io::Read::read_to_end(&mut file, &mut buffer)?;

        Ok(buffer.into())
    }

    unsafe { code.write(0) };

    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(
        unsafe { core::ffi::CStr::from_ptr(path) }.to_bytes(),
    );
    let pub_key = unsafe { core::ffi::CStr::from_ptr(pub_key) };

    match pub_key
        .to_str()
        .map_err(error::Error::from)
        .and_then(|pub_key| decrypt(path, pub_key))
    {
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
