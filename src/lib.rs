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
    static mut ARGS: &mut [*const core::ffi::c_char] = &mut [c"number".as_ptr(), core::ptr::null()];

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
