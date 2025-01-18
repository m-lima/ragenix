#![deny(warnings, clippy::pedantic)]

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn decrypt(
    path: *const core::ffi::c_char,
    pub_key: *const core::ffi::c_char,
    code: *mut core::ffi::c_uchar,
) -> RageString {
    unsafe { code.write(0) };
    let path = unsafe { core::ffi::CStr::from_ptr(path) };
    let pub_key = unsafe { core::ffi::CStr::from_ptr(pub_key) };

    match inner_decrypt(
        <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(path.to_bytes()),
        pub_key,
    ) {
        Ok(string) => string,
        Err(err) => {
            unsafe { code.write(err.code()) };
            err.message()
        }
    }
}

#[no_mangle]
pub extern "C" fn dealloc(RageString { data, len, cap }: RageString) {
    core::mem::drop(unsafe { Vec::from_raw_parts(data, len, cap) });
}

fn inner_decrypt<P: AsRef<std::path::Path>>(
    path: P,
    pub_key: &core::ffi::CStr,
) -> Result<RageString> {
    let path = path.as_ref();
    let _pub_key = pub_key.to_str()?;

    let mut file = std::fs::File::open(path)?;
    let mut buffer = std::fs::metadata(path)
        .ok()
        .and_then(|m| usize::try_from(m.len()).ok())
        .map_or_else(Vec::new, Vec::with_capacity);

    std::io::Read::read_to_end(&mut file, &mut buffer)?;

    Ok(buffer.into())
}

#[repr(C)]
pub struct RageString {
    data: *mut u8,
    len: usize,
    cap: usize,
}

impl From<String> for RageString {
    fn from(value: String) -> Self {
        let mut value = value.into_bytes();
        if !matches!(value.last(), Some(0)) {
            value.push(0);
        }
        let data = value.as_mut_ptr();
        let len = value.len();
        let cap = value.capacity();
        core::mem::forget(value);
        Self { data, len, cap }
    }
}

impl From<Vec<u8>> for RageString {
    fn from(mut value: Vec<u8>) -> Self {
        if !matches!(value.last(), Some(0)) {
            value.push(0);
        }
        let data = value.as_mut_ptr();
        let len = value.len();
        let cap = value.capacity();
        core::mem::forget(value);
        Self { data, len, cap }
    }
}

type Result<T = ()> = core::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    StringConversion(core::str::Utf8Error),
    IO(std::io::Error),
}

impl Error {
    fn code(&self) -> u8 {
        match self {
            Error::StringConversion(_) => 1,
            Error::IO(_) => 2,
        }
    }

    fn message(&self) -> RageString {
        self.to_string().into()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::StringConversion(erro) => erro.fmt(f),
            Error::IO(error) => error.fmt(f),
        }
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(error: core::str::Utf8Error) -> Self {
        Self::StringConversion(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl core::error::Error for Error {}

extern "C" {
    fn cpp_entry();
}

#[no_mangle]
pub extern "C" fn nix_plugin_entry() {
    unsafe { cpp_entry() }
}
