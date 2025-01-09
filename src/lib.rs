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
    context: *mut nix::nix_c_context,
    state: *mut nix::EvalState,
    args: *mut *mut nix::nix_value,
    out_value: *mut nix::nix_value,
) {
    fn increment(
        state: *mut nix::EvalState,
        args: *mut *mut nix::nix_value,
        out_value: *mut nix::nix_value,
    ) -> Result {
        let arg = unsafe { *args };
        let context = Context::new();
        context.eval(state, arg)?;
        let value = context.get_path(arg)?;
        LOG.write(|f| {
            let value = unsafe { core::ffi::CStr::from_ptr(value) };
            writeln!(f, "{value:?}")
        })?;
        context.set_int(out_value, 3)?;

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
        error.report(context.as_ptr());
    }
}
