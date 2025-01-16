use crate::{context::AsContext, nix, Args, Context, Error, Result, State, Value};

pub type PrimOpFunc<const L: usize> = fn(
    &Context<false>,
    &State<'_, Context<false>, false>,
    Args<'_, State<'_, Context<false>, false>, L>,
    Value<'_, State<'_, Context<false>, false>, false>,
) -> Result;

pub struct PrimOp<'a, C: AsContext, const L: usize> {
    primop: *mut nix::PrimOp,
    context: &'a C,
}

impl<'a, C: AsContext, const L: usize> PrimOp<'a, C, L> {
    pub fn new(
        context: &'a C,
        func: PrimOpFunc<L>,
        name: &'static core::ffi::CStr,
        args: &mut [*const core::ffi::c_char; L],
        doc: &'static core::ffi::CStr,
    ) -> Result<Self> {
        let len = core::ffi::c_int::try_from(L.min(1) - 1)
            .map_err(|_| Error::custom(c"Could not fit argument count within usize"))?;

        context
            .with_check(|c| unsafe {
                nix::nix_alloc_primop(
                    c,
                    Some(Self::wrapper),
                    len,
                    name.as_ptr(),
                    args.as_mut_ptr().cast(),
                    doc.as_ptr(),
                    func as *mut _,
                )
            })
            .map(|primop| Self { primop, context })
    }

    pub fn register(self) -> Result {
        self.context
            .check(|c| unsafe { nix::nix_register_primop(c, self.primop) })?;
        Ok(())
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    extern "C" fn wrapper(
        data: *mut ::core::ffi::c_void,
        context: *mut nix::nix_c_context,
        state: *mut nix::EvalState,
        args: *mut *mut nix::nix_value,
        out_value: *mut nix::nix_value,
    ) {
        fn inner<const L: usize>(
            data: *mut ::core::ffi::c_void,
            context: *mut nix::nix_c_context,
            state: *mut nix::EvalState,
            args: *mut *mut nix::nix_value,
            out_value: *mut nix::nix_value,
        ) -> Result {
            let context = Context::wrap(context);
            let state = State::wrap(state, &context);
            let args = Args::<_, L>::wrap(args, &state)?;
            let out_value = Value::wrap(out_value, &state);

            let func: PrimOpFunc<L> = unsafe { core::mem::transmute(data) };
            func(&context, &state, args, out_value)
        }

        if let Err(error) = inner::<L>(data, context, state, args, out_value) {
            let context = Context::wrap(context);
            error.report(&context);
        }
    }
}

impl<C: AsContext, const L: usize> Drop for PrimOp<'_, C, L> {
    fn drop(&mut self) {
        if let Err(err) = self
            .context
            .check(|c| unsafe { nix::nix_gc_decref(c, self.primop as *const _) })
        {
            #[cfg(feature = "log")]
            let _ = crate::log::write(|f| writeln!(f, "{err}"));
            drop(err);
        }
    }
}
