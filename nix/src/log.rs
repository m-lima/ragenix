use crate::{Error, Result};

static LOG: Log = Log::new();

struct Log(std::sync::LazyLock<std::sync::Mutex<Option<std::fs::File>>>);

impl Log {
    const fn new() -> Self {
        Self(std::sync::LazyLock::new(|| {
            std::sync::Mutex::new(
                std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open("/tmp/ragenix.log")
                    .ok(),
            )
        }))
    }
}

pub fn write<F, R>(f: F) -> Result<R>
where
    F: Fn(&mut dyn std::io::Write) -> std::io::Result<R>,
{
    let mut lock = LOG
        .0
        .lock()
        .map_err(|err| Error::custom(format!("Could not acquire log lock: {err}")))?;

    let Some(file) = core::ops::DerefMut::deref_mut(&mut lock) else {
        return Err(Error::custom(c"Could not open log file"));
    };

    f(file).map_err(|err| Error::custom(format!("Failed to write to log: {err}")))
}
