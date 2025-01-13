#![allow(dead_code)]

use crate::nix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    context: *mut nix::RawContext,
}

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

    pub fn create_store(&self) -> Result<Store> {
        self.check_with_code(|c| unsafe { inner::nix_libstore_init(c) })
            .and_then(|()| Store::new(self))
    }

    pub fn create_state(&self, store: &Store) {
        self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })
            .and_then(|()| State::new(self))
    }

    pub fn eval(
        &self,
        state: *mut nix::State,
        expr: *const core::ffi::c_char,
        path: *const core::ffi::c_char,
    ) -> Result<Value> {
        self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })?;
        let value = Value::new(self, state)?;
        self.check_with_code(|c| unsafe {
            inner::nix_expr_eval_from_string(c, state, expr, path, value.value)
        })?;
        Ok(value)
    }

    pub fn bork_all(state: *mut nix::State) {
        unsafe { inner::nix_state_free(state) };
    }

    pub fn init(&self) -> Result {
        self.init_expr()
            .and_then(|()| self.init_util())
            .and_then(|()| self.init_store())
    }

    pub fn init_expr(&self) -> Result {
        self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })
    }

    pub fn init_util(&self) -> Result {
        self.check_with_code(|c| unsafe { inner::nix_libutil_init(c) })
    }

    pub fn init_store(&self) -> Result {
        self.check_with_code(|c| unsafe { inner::nix_libstore_init(c) })
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
            Err(Error::wrap(code, message, len))
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

pub struct Store<'a> {
    store: *mut inner::Store,
    context: &'a Context,
}

impl<'a> Store<'a> {
    fn new(context: &'a Context) -> Result<Self> {
        context
            .check(|c| unsafe {
                inner::nix_store_open(c, core::ptr::null(), core::ptr::null_mut())
            })
            .map(|store| Self { store, context })
    }
}

impl Drop for Store<'_> {
    fn drop(&mut self) {
        unsafe { inner::nix_store_free(self.store) }
    }
}
