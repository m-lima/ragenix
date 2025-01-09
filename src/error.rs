use crate::nix;

#[derive(Debug)]
pub struct Error {
    code: nix::Error,
    message: *const core::ffi::c_char,
    len: usize,
}

impl Error {
    pub fn new(code: nix::Error, message: *const core::ffi::c_char, len: usize) -> Self {
        Self { code, message, len }
    }

    pub fn custom(message: &'static core::ffi::CStr) -> Self {
        let len = message.count_bytes();
        Self::new(nix::ERR_UNKNOWN, message.as_ptr(), len)
    }

    pub fn report(&self, reporter: impl Reporter) {
        reporter.report(self.code, self.message);
    }
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let slice = unsafe { core::slice::from_raw_parts(self.message.cast::<u8>(), self.len) };
        if let Ok(string) = core::str::from_utf8(slice) {
            f.write_str(string)
        } else {
            for chunk in slice.utf8_chunks() {
                for ch in chunk.valid().chars() {
                    write!(f, "{ch}")?;
                }
                for byte in chunk.invalid() {
                    write!(f, "\\x{byte:02X}")?;
                }
            }
            Ok(())
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;

pub trait Reporter {
    fn report(self, code: nix::Error, message: *const core::ffi::c_char);
}
