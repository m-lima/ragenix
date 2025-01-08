use crate::nix;

#[derive(Debug)]
pub struct Error {
    code: nix::nix_err,
    message: *const core::ffi::c_char,
    len: usize,
}

impl Error {
    pub fn new(code: nix::nix_err, message: *const core::ffi::c_char, len: usize) -> Self {
        Self { code, message, len }
    }

    pub fn custom(message: &'static core::ffi::CStr) -> Self {
        let len = message.count_bytes();
        Self::new(nix::nix_err_NIX_ERR_UNKNOWN, message.as_ptr(), len)
    }

    pub fn report(&self, context: *mut nix::nix_c_context) {
        unsafe { nix::nix_set_err_msg(context, self.code, self.message) };
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.message, self.len) };
        match std::str::from_utf8(slice) {
            Ok(string) => f.write_str(string),
            Err(_) => f.write_str(&String::from_utf8_lossy(slice)),
        }
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;
