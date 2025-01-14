use crate::nix::{self, inner};

pub type PrimOpFunc<const CO: bool = false, const SO: bool = false, const VO: bool = false> =
    dyn for<'c, 's> Fn(
        &'c nix::Context<CO>,
        &'s nix::State<'c, nix::Context<CO>, SO>,
        &nix::Args<'s, nix::State<'c, nix::Context<CO>, SO>>,
        &nix::Value<'s, nix::State<'c, nix::Context<CO>, SO>, VO>,
    ) -> nix::Result;

pub struct PrimOp<'a, C: nix::context::AsContext> {
    primop: *mut inner::PrimOp,
    context: &'a C,
}

impl<'a, C: nix::context::AsContext> PrimOp<'a, C> {
    pub fn new(
        context: &'a C,
        func: Box<PrimOpFunc>,
        name: &'static core::ffi::CStr,
        args: &mut [*const core::ffi::c_char],
        doc: &'static core::ffi::CStr,
    ) -> nix::Result<Self> {
        let len = core::ffi::c_int::try_from(args.len().min(1) - 1)
            .map_err(|_| nix::Error::custom(c"Could not fit argument count within usize"))?;

        let func = Box::leak(func);
        #[cfg(feature = "log")]
        crate::log::write(|f| writeln!(f, "Incoming {:?}", core::ptr::from_ref(func)))?;

        context
            .with_check(|c| unsafe {
                inner::nix_alloc_primop(
                    c,
                    Some(wrapper),
                    len,
                    name.as_ptr(),
                    args.as_mut_ptr().cast(),
                    doc.as_ptr(),
                    core::ptr::from_mut(func).cast(),
                )
            })
            .map(|primop| Self { primop, context })
    }

    pub fn register(self) -> nix::Result {
        self.context
            .check(|c| unsafe { inner::nix_register_primop(c, self.primop) })?;
        Ok(())
    }
}

impl<C: nix::context::AsContext> Drop for PrimOp<'_, C> {
    fn drop(&mut self) {
        if let Err(err) = self
            .context
            .check(|c| unsafe { inner::nix_gc_decref(c, self.primop as *const _) })
        {
            #[cfg(feature = "log")]
            let _ = crate::log::write(|f| writeln!(f, "{err}"));
            drop(err);
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" fn wrapper(
    data: *mut ::core::ffi::c_void,
    context: *mut inner::nix_c_context,
    state: *mut inner::EvalState,
    args: *mut *mut inner::nix_value,
    out_value: *mut inner::nix_value,
) {
    fn inner(
        data: *mut ::core::ffi::c_void,
        context: *mut inner::nix_c_context,
        state: *mut inner::EvalState,
        args: *mut *mut inner::nix_value,
        out_value: *mut inner::nix_value,
    ) -> nix::Result {
        let context = nix::Context::wrap(context);
        let state = nix::State::wrap(state, &context);
        let args = nix::Args::wrap(args, &state)?;
        let out_value = nix::Value::wrap(out_value, &state);

        let func: *mut PrimOpFunc = {
            let raw = unsafe {
                core::slice::from_raw_parts_mut(
                    data.cast::<u8>(),
                    core::mem::size_of::<*mut PrimOpFunc>(),
                )
            };
            let raw: [u8; core::mem::size_of::<*mut PrimOpFunc>()] =
                unsafe { core::mem::transmute(raw) };
            unsafe { std::mem::transmute(raw) }
        };

        let func = unsafe { func.as_mut() }
            .ok_or_else(|| nix::Error::custom(c"Pointer to function is invalid"))?;
        #[cfg(feature = "log")]
        crate::log::write(|f| writeln!(f, "Outgoing {:?}", core::ptr::from_ref(func)))?;
        func(&context, &state, &args, &out_value)
    }

    if let Err(error) = inner(data, context, state, args, out_value) {
        let context = nix::Context::wrap(context);
        error.report(&context);
    }
}
