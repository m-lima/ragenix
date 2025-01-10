mod error;
mod nix;

use error::Result;
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
            .map_err(|_| error::Error::custom(c"Failed to write to log"))
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
        let arg = nix::Value::own(&context, unsafe { *args })?;

        arg.eval(state)?;

        let path = arg.get_path()?;
        let path_str = unsafe { core::ffi::CStr::from_ptr(path) };
        let path_copy = Vec::from(path_str.to_bytes_with_nul());
        // let new_value = context.create_value(state)?;

        LOG.write(|f| writeln!(f, "{path_str:?} :: {path_copy:?}"))?;
        let out_value = nix::Value::own(&context, out_value)?;
        out_value.set_path(state, path_copy.as_ptr())?;

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
