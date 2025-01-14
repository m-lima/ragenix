use crate::{context::AsContext, nix};

#[derive(Debug)]
pub enum Error {
    Static {
        code: nix::nix_err,
        message: *const core::ffi::c_char,
        len: usize,
    },
    String {
        code: nix::nix_err,
        message: std::ffi::CString,
    },
}

impl Error {
    pub fn custom<I: Initialiazer>(initializer: I) -> Self {
        initializer.to_error()
    }

    pub fn report<C: AsContext>(&self, context: &C) {
        let (code, message) = match self {
            Error::Static { code, message, .. } => (*code, *message),
            Error::String { code, message } => (*code, message.as_ptr()),
        };

        let _ = context.check(|c| unsafe { nix::nix_set_err_msg(c, code, message) });
        #[cfg(feature = "log")]
        let _ = crate::log::write(|f| writeln!(f, "{self}"));
    }
}

impl Error {
    pub(super) fn wrap(code: nix::nix_err, message: *const core::ffi::c_char, len: usize) -> Self {
        Self::Static { code, message, len }
    }
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let slice = match self {
            Error::Static { message, len, .. } => unsafe {
                core::slice::from_raw_parts(message.cast::<u8>(), *len)
            },
            Error::String { message, .. } => message.to_bytes(),
        };

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

pub trait Initialiazer {
    fn to_error(self) -> Error;
}

impl Initialiazer for &'static core::ffi::CStr {
    fn to_error(self) -> Error {
        let len = self.count_bytes();
        Error::wrap(nix::nix_err_NIX_ERR_UNKNOWN, self.as_ptr(), len)
    }
}

impl Initialiazer for std::ffi::CString {
    fn to_error(self) -> Error {
        Error::String {
            code: nix::nix_err_NIX_ERR_UNKNOWN,
            message: self,
        }
    }
}

impl Initialiazer for String {
    fn to_error(self) -> Error {
        let message = match std::ffi::CString::new(self) {
            Ok(message) => message,
            Err(err) => {
                let len = err.nul_position();
                let mut bytes = err.into_vec();
                bytes.truncate(len);
                std::ffi::CString::new(bytes).unwrap()
            }
        };
        Error::String {
            code: nix::nix_err_NIX_ERR_UNKNOWN,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_to_string() {
        let error = Error::custom(c"Error").to_string();
        assert_eq!(error, "Error");

        let error = Error::custom(String::from("Error")).to_string();
        assert_eq!(error, "Error");

        let error = Error::custom(String::from("Error\0yo")).to_string();
        assert_eq!(error, "Error");
    }
}
