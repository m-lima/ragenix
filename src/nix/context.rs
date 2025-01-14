use crate::nix::{self, inner};

#[derive(Debug)]
pub struct Context<const OWNED: bool> {
    context: *mut inner::nix_c_context,
}

impl Context<true> {
    pub fn new() -> Self {
        Self {
            context: unsafe { inner::nix_c_context_create() },
        }
    }
}

impl Context<false> {
    pub fn wrap(context: *mut inner::nix_c_context) -> Self {
        Self { context }
    }

    pub fn to_owned(&self) -> nix::Result<Context<true>> {
        self.check(|c| unsafe { inner::nix_gc_incref(c, self.context as *const _) })?;
        Ok(Context {
            context: self.context,
        })
    }

    // pub fn create_primop(
    //     &self,
    //     func: nix::PrimOpFunc,
    //     name: &'static core::ffi::CStr,
    //     args: &mut [*const core::ffi::c_char],
    //     doc: &'static core::ffi::CStr,
    // ) -> Result {
    //     let primop = PrimOp::new(self, func, name, args, doc)?;
    //     primop.register()
    // }

    // pub fn create_value(&self, state: *mut nix::State) -> Result<Value> {
    //     Value::new(self, state)
    // }
    //
    // pub fn create_store(&self) -> Result<Store> {
    //     self.check_with_code(|c| unsafe { inner::nix_libstore_init(c) })
    //         .and_then(|()| Store::new(self))
    // }
    //
    // pub fn create_state(&self, store: &Store) {
    //     self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })
    //         .and_then(|()| State::new(self))
    // }
    //
    // pub fn eval(
    //     &self,
    //     state: *mut nix::State,
    //     expr: *const core::ffi::c_char,
    //     path: *const core::ffi::c_char,
    // ) -> Result<Value> {
    //     self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })?;
    //     let value = Value::new(self, state)?;
    //     self.check_with_code(|c| unsafe {
    //         inner::nix_expr_eval_from_string(c, state, expr, path, value.value)
    //     })?;
    //     Ok(value)
    // }
    //
    // pub fn bork_all(state: *mut nix::State) {
    //     unsafe { inner::nix_state_free(state) };
    // }
    //
    // pub fn init(&self) -> Result {
    //     self.init_expr()
    //         .and_then(|()| self.init_util())
    //         .and_then(|()| self.init_store())
    // }
    //
    // pub fn init_expr(&self) -> Result {
    //     self.check_with_code(|c| unsafe { inner::nix_libexpr_init(c) })
    // }
    //
    // pub fn init_util(&self) -> Result {
    //     self.check_with_code(|c| unsafe { inner::nix_libutil_init(c) })
    // }
    //
    // pub fn init_store(&self) -> Result {
    //     self.check_with_code(|c| unsafe { inner::nix_libstore_init(c) })
    // }
}

impl<const OWNED: bool> Context<OWNED> {
    fn inner_check<T>(&self, value: T, code: inner::nix_err) -> nix::Result<T> {
        if code == inner::nix_err_NIX_OK {
            Ok(value)
        } else {
            let mut len = 0;
            let message =
                unsafe { inner::nix_err_msg(core::ptr::null_mut(), self.context, &mut len) };
            let len = len as usize;
            Err(nix::Error::wrap(code, message, len))
        }
    }
}

impl<const OWNED: bool> Drop for Context<OWNED> {
    fn drop(&mut self) {
        if OWNED {
            unsafe { inner::nix_c_context_free(self.context) };
        }
    }
}

pub trait AsContext {
    fn check<F: FnOnce(*mut inner::nix_c_context) -> inner::nix_err>(&self, f: F) -> nix::Result;
    fn with_check<T, F: FnOnce(*mut inner::nix_c_context) -> T>(&self, f: F) -> nix::Result<T>;
}

impl<const OWNED: bool> AsContext for Context<OWNED> {
    fn check<F: FnOnce(*mut inner::nix_c_context) -> inner::nix_err>(&self, f: F) -> nix::Result {
        self.inner_check((), f(self.context))
    }

    fn with_check<T, F: FnOnce(*mut inner::nix_c_context) -> T>(&self, f: F) -> nix::Result<T> {
        self.inner_check(f(self.context), unsafe {
            inner::nix_err_code(self.context)
        })
    }
}
