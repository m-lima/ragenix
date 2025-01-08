use crate::error::{Error, Result};
use crate::nix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context(*mut nix::nix_c_context);

impl Context {
    pub fn new() -> Self {
        Self(unsafe { nix::nix_c_context_create() })
    }

    pub fn as_ptr(&self) -> *mut nix::nix_c_context {
        self.0
    }

    pub fn create_primop(
        &self,
        func: nix::PrimOpFunc,
        name: &'static core::ffi::CStr,
        args: *mut [&'static core::ffi::CStr],
        doc: &'static core::ffi::CStr,
    ) -> Result {
        let primop = PrimOp::new(self, func, name, args, doc)?;
        primop.register()
    }

    pub fn get_int(&self, value: *const nix::nix_value) -> Result<i64> {
        let value_type = self.get_type(value)?;
        if value_type == nix::ValueType_NIX_TYPE_INT {
            self.check(unsafe { nix::nix_get_int(self.0, value) })
        } else {
            Err(Error::custom("Value is not an integer"))
        }
    }

    pub fn set_int(&self, out_value: *mut nix::nix_value, int: i64) -> Result {
        self.check(unsafe { nix::nix_init_int(self.0, out_value, int) })
            .map(|_| ())
    }
}

impl Context {
    pub fn force_eval(&self, state: *mut nix::EvalState, value: *mut nix::nix_value) -> Result {
        self.check(unsafe { nix::nix_value_force(self.0, state, value) })?;
        Ok(())
    }

    fn check<T>(&self, value: T) -> Result<T> {
        let code = unsafe { nix::nix_err_code(self.0) };
        if code == nix::nix_err_NIX_OK {
            Ok(value)
        } else {
            let mut len = 0;
            let message = unsafe { nix::nix_err_msg(core::ptr::null_mut(), self.0, &mut len) };
            let len = len as usize;
            Err(Error::new(code, message, len))
        }
    }

    fn get_type(&self, value: *const nix::nix_value) -> Result<nix::ValueType> {
        self.check(unsafe { nix::nix_get_type(self.0, value) })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { nix::nix_c_context_free(self.0) };
    }
}

struct PrimOp<'a> {
    primop: *mut nix::PrimOp,
    context: &'a Context,
}

impl<'a> PrimOp<'a> {
    fn new(
        context: &'a Context,
        func: nix::PrimOpFunc,
        name: &'static core::ffi::CStr,
        args: *mut [&'static core::ffi::CStr],
        doc: &'static core::ffi::CStr,
    ) -> Result<Self> {
        let len = core::ffi::c_int::try_from(args.len())
            .map_err(|_| Error::custom("Could not fit argument count within usize"))?;
        let args = args.cast::<*const u8>();

        let primop = context.check(unsafe {
            nix::nix_alloc_primop(
                context.as_ptr(),
                Some(func),
                len,
                name.as_ptr(),
                args,
                doc.as_ptr(),
                core::ptr::null_mut(),
            )
        })?;
        Ok(Self { primop, context })
    }

    fn register(self) -> Result {
        self.context
            .check(unsafe { nix::nix_register_primop(self.context.as_ptr(), self.primop) })?;
        Ok(())
    }
}

impl Drop for PrimOp<'_> {
    fn drop(&mut self) {
        unsafe { nix::nix_gc_decref(self.context.as_ptr(), self.primop as *const _) };
    }
}
