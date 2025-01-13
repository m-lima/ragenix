use crate::nix::{self, inner};

pub type PrimOpFunc = unsafe extern "C" fn(
    user_data: *mut ::core::ffi::c_void,
    context: *mut inner::RawContext,
    state: *mut nix::RawState,
    args: *mut *mut nix::RawValue,
    ret: *mut nix::RawValue,
);

struct PrimOp<'a> {
    primop: *mut inner::PrimOp,
    context: &'a nix::Context,
}

impl<'a> PrimOp<'a> {
    fn new(
        context: &'a nix::Context,
        func: nix::PrimOpFunc,
        name: &'static core::ffi::CStr,
        args: &mut [*const core::ffi::c_char],
        doc: &'static core::ffi::CStr,
    ) -> nix::Result<Self> {
        let len = core::ffi::c_int::try_from(args.len().min(1) - 1)
            .map_err(|_| nix::Error::custom(c"Could not fit argument count within usize"))?;

        context
            .check(|c| unsafe {
                inner::nix_alloc_primop(
                    c,
                    Some(func),
                    len,
                    name.as_ptr(),
                    args.as_mut_ptr().cast(),
                    doc.as_ptr(),
                    core::ptr::null_mut(),
                )
            })
            .map(|primop| Self { primop, context })
    }

    fn register(self) -> nix::Result {
        self.context
            .check(|c| unsafe { inner::nix_register_primop(c, self.primop) })?;
        Ok(())
    }
}

impl Drop for PrimOp<'_> {
    fn drop(&mut self) {
        unsafe { inner::nix_gc_decref(self.context.0, self.primop as *const _) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" fn increment(
    data: *mut ::core::ffi::c_void,
    context: *mut inner::nix_c_context,
    state: *mut inner::EvalState,
    args: *mut *mut inner::nix_value,
    out_value: *mut inner::nix_value,
) {
    let context = nix::Context::wrap(context);
    let state = nix::State::wrap(state, &context);
    let out_value = nix::Value::wrap(out_value, &state);
}
