#![deny(warnings, clippy::pedantic)]
#![allow(dead_code, unused_imports)]

#[cfg(feature = "log")]
mod log;
mod nix;

macro_rules! c_array {
    ($($value: literal),*) => {
        [$($value.as_ptr()),*, core::ptr::null()]
    }
}

fn load(
    _context: &nix::Context<false>,
    _state: &nix::State<'_, nix::Context<false>, false>,
    args: &nix::Args<'_, nix::State<'_, nix::Context<false>, false>>,
    out: &nix::Value<'_, nix::State<'_, nix::Context<false>, false>, false>,
) -> nix::Result {
    let arg = args
        .get(0)
        .ok_or_else(|| nix::Error::custom(c"Expected a single argument"))?;

    arg.eval()?;

    let int = arg.get_int()?;
    out.set_int(int)
}

#[no_mangle]
pub extern "C" fn nix_plugin_entry() {
    static mut ARGS: &mut [*const core::ffi::c_char; 2] = &mut c_array![c"path"];

    let context = nix::Context::new();
    if let Err(error) = nix::PrimOp::new(
        &context,
        load,
        c"decrypt",
        unsafe { ARGS },
        c"Decrypt and evaluate a file",
    )
    .and_then(nix::PrimOp::register)
    {
        error.report(&context);
    }
}
