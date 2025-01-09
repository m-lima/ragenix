use crate::{
    error::{Error, Result},
    nix::{self, inner},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context(*mut nix::RawContext);

impl Context {
    pub fn new() -> Self {
        Self(unsafe { inner::nix_c_context_create() })
    }

    pub fn as_ptr(&self) -> *mut nix::RawContext {
        self.0
    }

    pub fn create_primop(
        &self,
        func: nix::PrimOpFunc,
        name: &'static core::ffi::CStr,
        args: &mut [*const core::ffi::c_char],
        doc: &'static core::ffi::CStr,
    ) -> Result {
        let primop = PrimOp::new(self, func, name, args, doc)?;
        primop.register()
    }

    pub fn eval(&self, state: *mut nix::State, value: *mut nix::Value) -> Result {
        self.check_with_code(unsafe { inner::nix_value_force(self.0, state, value) })
    }

    pub fn alloc(&self, state: *mut nix::State) -> Result<*mut nix::Value> {
        self.check(unsafe { inner::nix_alloc_value(self.0, state) })
    }

    pub fn get_int(&self, value: *const nix::Value) -> Result<i64> {
        let value_type = self.get_type(value)?;
        if value_type == inner::ValueType_NIX_TYPE_INT {
            self.check(unsafe { inner::nix_get_int(self.0, value) })
        } else {
            Err(Error::custom(c"Value is not an integer"))
        }
    }

    pub fn set_int(&self, out_value: *mut nix::Value, int: i64) -> Result {
        self.check_with_code(unsafe { inner::nix_init_int(self.0, out_value, int) })
    }

    pub fn get_path(&self, value: *const nix::Value) -> Result<*const core::ffi::c_char> {
        let value_type = self.get_type(value)?;
        if value_type == inner::ValueType_NIX_TYPE_PATH {
            self.check(unsafe { inner::nix_get_path_string(self.0, value) })
        } else {
            Err(Error::custom(c"Value is not a path"))
        }
    }

    pub fn set_path(
        &self,
        state: *mut nix::State,
        out_value: *mut nix::Value,
        path: *const core::ffi::c_char,
    ) -> Result {
        self.check_with_code(unsafe { inner::nix_init_path_string(self.0, state, out_value, path) })
    }
}

impl Context {
    fn check<T>(&self, value: T) -> Result<T> {
        self.check_internal(value, unsafe { inner::nix_err_code(self.0) })
    }

    fn check_with_code(&self, code: nix::Error) -> Result {
        self.check_internal((), code)
    }

    fn check_internal<T>(&self, value: T, code: nix::Error) -> Result<T> {
        if code == inner::nix_err_NIX_OK {
            Ok(value)
        } else {
            let mut len = 0;
            let message = unsafe { inner::nix_err_msg(core::ptr::null_mut(), self.0, &mut len) };
            let len = len as usize;
            Err(Error::new(code, message, len))
        }
    }

    fn get_type(&self, value: *const nix::Value) -> Result<nix::ValueType> {
        self.check(unsafe { inner::nix_get_type(self.0, value) })
    }
}

impl crate::error::Reporter for Context {
    fn report(self, code: nix::Error, message: *const core::ffi::c_char) {
        self.0.report(code, message);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { inner::nix_c_context_free(self.0) };
    }
}

struct PrimOp<'a> {
    primop: *mut inner::PrimOp,
    context: &'a Context,
}

impl<'a> PrimOp<'a> {
    fn new(
        context: &'a Context,
        func: nix::PrimOpFunc,
        name: &'static core::ffi::CStr,
        args: &mut [*const core::ffi::c_char],
        doc: &'static core::ffi::CStr,
    ) -> Result<Self> {
        let len = core::ffi::c_int::try_from(args.len().min(1) - 1)
            .map_err(|_| Error::custom(c"Could not fit argument count within usize"))?;

        context
            .check(unsafe {
                inner::nix_alloc_primop(
                    context.as_ptr(),
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

    fn register(self) -> Result {
        self.context
            .check(unsafe { inner::nix_register_primop(self.context.as_ptr(), self.primop) })?;
        Ok(())
    }
}

impl Drop for PrimOp<'_> {
    fn drop(&mut self) {
        unsafe { inner::nix_gc_decref(self.context.as_ptr(), self.primop as *const _) };
    }
}
