#[derive(Debug)]
pub enum Error {
    StringConversion(core::str::Utf8Error),
    IO(std::io::Error),
}

impl Error {
    pub fn code(&self) -> u8 {
        match self {
            Error::StringConversion(_) => 1,
            Error::IO(_) => 2,
        }
    }

    pub fn message(&self) -> crate::string::String {
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
