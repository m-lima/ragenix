mod error;
mod nix;

use error::{Error, Result};
use nix::Context;

macro_rules! c_array {
    ($($value: literal),*) => {
        [$($value.as_ptr()),*, core::ptr::null()]
    }
}

static LOG: Log = Log::new();

struct Log(std::sync::LazyLock<std::sync::Mutex<std::fs::File>>);

impl Log {
    const fn new() -> Self {
        Self(std::sync::LazyLock::new(|| {
            std::sync::Mutex::new(
                std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open("/tmp/ragenix.log")
                    .unwrap(),
            )
        }))
    }

    fn get(&self) -> std::sync::MutexGuard<'_, std::fs::File> {
        self.0.lock().unwrap()
    }

    fn write<F, R>(&self, f: F) -> Result<R>
    where
        F: Fn(&mut dyn std::io::Write) -> std::io::Result<R>,
    {
        f(std::ops::DerefMut::deref_mut(&mut self.get()))
            .map_err(|err| error::Error::custom(format!("Failed to write to log: {err}")))
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" fn increment(
    _: *mut ::core::ffi::c_void,
    context: *mut nix::RawContext,
    state: *mut nix::State,
    args: *mut *mut nix::RawValue,
    out_value: *mut nix::RawValue,
) {
    fn increment(
        state: *mut nix::State,
        args: *mut *mut nix::RawValue,
        out_value: *mut nix::RawValue,
    ) -> Result {
        let context = Context::new();
        context.init()?;
        let arg = nix::Value::own(&context, unsafe { *args })?;

        arg.eval(state)?;

        LOG.write(|f| writeln!(f, "State: {state:?}"))?;

        let path_str = unsafe { core::ffi::CStr::from_ptr(arg.get_path()?) };
        let path =
            <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(path_str.to_bytes());
        let mut file = std::fs::File::open(path)
            .map_err(|err| Error::custom(format!("Could not open the referenced file: {err}")))?;

        let mut file_content = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut file_content)
            .map_err(|err| Error::custom(format!("Failed to read file: {err}")))?;
        file_content.push(0);

        LOG.write(|f| {
            write!(f, "{path_str:?} :: ")?;
            f.write_all(&file_content)
        })?;

        let evaluated = context.eval(state, file_content.as_ptr().cast(), path_str.as_ptr())?;
        LOG.write(|f| writeln!(f, "Evaluated"))?;
        let a = evaluated.get_int()?;

        LOG.write(|f| writeln!(f, "{a}"))?;

        let out_value = nix::Value::own(&context, out_value)?;
        out_value.set_path(state, file_content.as_ptr().cast())?;

        Ok(())
    }

    if let Err(error) = increment(state, args, out_value) {
        error.report(context);
    }
}

#[no_mangle]
pub extern "C" fn nix_plugin_entry() {
    static mut ARGS: &mut [*const core::ffi::c_char; 2] = &mut c_array![c"number"];

    let context = Context::new();
    if let Err(error) = context.create_primop(
        increment,
        c"increment",
        unsafe { ARGS },
        c"Increment the value",
    ) {
        error.report(context);
    }
}
