mod context;
mod error;
mod nix;

use context::Context;
use error::Result;

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
        context.force_eval(state, arg)?;
        let value = context.get_int(arg)?;
        context.set_int(out_value, value + 1)?;

        Ok(())
    }

    if let Err(error) = increment(state, args, out_value) {
        error.report(context);
    }
}

#[no_mangle]
pub extern "C" fn nix_plugin_entry() {
    fn plugin_entry(context: &Context) -> Result {
        static mut ARGS: [&core::ffi::CStr; 1] = [c"number"];
        context.create_primop(
            increment,
            c"increment",
            &raw mut ARGS,
            c"Increment the value",
        )
    }

    let context = Context::new();
    if let Err(error) = plugin_entry(&context) {
        error.report(context.as_ptr());
    }
}
