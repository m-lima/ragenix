#![allow(dead_code)]

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

    pub fn create_value(&self, state: *mut nix::State) -> Result<Value> {
        Value::new(self, state)
    }
}

impl Context {
    fn check<T, F: FnOnce(*mut nix::RawContext) -> T>(&self, f: F) -> Result<T> {
        self.check_internal(f(self.0), unsafe { inner::nix_err_code(self.0) })
    }

    fn check_with_code<F: FnOnce(*mut nix::RawContext) -> nix::Error>(&self, f: F) -> Result {
        self.check_internal((), f(self.0))
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

    fn register(self) -> Result {
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

pub struct Value<'a> {
    value: *mut inner::nix_value,
    context: &'a Context,
}

impl<'a> Value<'a> {
    pub fn own(context: &'a Context, value: *mut inner::nix_value) -> Result<Self> {
        context
            .check_with_code(|c| unsafe { inner::nix_gc_incref(c, value as *const _) })
            .map(|()| Self { value, context })
    }

    fn new(context: &'a Context, state: *mut nix::State) -> Result<Self> {
        context
            .check(|c| unsafe { inner::nix_alloc_value(c, state) })
            .map(|value| Self { value, context })
    }
}

impl Value<'_> {
    pub fn eval(&self, state: *mut nix::State) -> Result {
        self.context
            .check_with_code(|c| unsafe { inner::nix_value_force(c, state, self.value) })
    }

    pub fn get_int(&self) -> Result<i64> {
        let value_type = self.get_type()?;
        if value_type == inner::ValueType_NIX_TYPE_INT {
            self.context
                .check(|c| unsafe { inner::nix_get_int(c, self.value) })
        } else {
            Err(Error::custom(c"Value is not an integer"))
        }
    }

    pub fn set_int(&self, value: i64) -> Result {
        self.context
            .check_with_code(|c| unsafe { inner::nix_init_int(c, self.value, value) })
    }

    pub fn get_path(&self) -> Result<*const core::ffi::c_char> {
        let value_type = self.get_type()?;
        if value_type == inner::ValueType_NIX_TYPE_PATH {
            self.context
                .check(|c| unsafe { inner::nix_get_path_string(c, self.value) })
        } else {
            Err(Error::custom(c"Value is not a path"))
        }
    }

    pub fn set_path(&self, state: *mut nix::State, path: *const core::ffi::c_char) -> Result {
        self.context
            .check_with_code(|c| unsafe { inner::nix_init_path_string(c, state, self.value, path) })
    }
}

impl Value<'_> {
    fn get_type(&self) -> Result<nix::ValueType> {
        self.context
            .check(|c| unsafe { inner::nix_get_type(c, self.value) })
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        unsafe { inner::nix_gc_decref(self.context.0, self.value as *const _) };
    }
}
