#[derive(Debug)]
pub enum Error {
    StringConversion(core::str::Utf8Error),
    IO(std::io::Error),
    Identity(age::cli_common::ReadError),
    Age(age::DecryptError),
}

impl Error {
    pub fn code(&self) -> u8 {
        match self {
            Error::StringConversion(_) => 1,
            Error::IO(_) => 2,
            Error::Identity(_) => 3,
            Error::Age(_) => 4,
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
            Error::Identity(error) => error.fmt(f),
            Error::Age(error) => error.fmt(f),
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

impl From<age::cli_common::ReadError> for Error {
    fn from(error: age::cli_common::ReadError) -> Self {
        Self::Identity(error)
    }
}

impl From<age::DecryptError> for Error {
    fn from(error: age::DecryptError) -> Self {
        Self::Age(error)
    }
}

impl core::error::Error for Error {}

pub fn wrap<I, O, E, F>(f: F) -> impl Fn(I) -> Result<O, Error>
where
    E: Into<Error>,
    F: 'static + Fn(I) -> Result<O, E>,
{
    move |i: I| f(i).map_err(Into::into)
}
