use crate::{nix, Error, Result};

#[derive(Debug)]
pub struct Context<const OWNED: bool> {
    context: *mut nix::nix_c_context,
}

impl Context<true> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: unsafe { nix::nix_c_context_create() },
        }
    }
}

impl Context<false> {
    pub fn wrap(context: *mut nix::nix_c_context) -> Self {
        Self { context }
    }

    pub fn to_owned(&self) -> Result<Context<true>> {
        self.check(|c| unsafe { nix::nix_gc_incref(c, self.context as *const _) })?;
        Ok(Context {
            context: self.context,
        })
    }
}

impl<const OWNED: bool> Context<OWNED> {
    fn inner_check<T>(&self, value: T, code: nix::nix_err) -> Result<T> {
        if code == nix::nix_err_NIX_OK {
            Ok(value)
        } else {
            let mut len = 0;
            let message =
                unsafe { nix::nix_err_msg(core::ptr::null_mut(), self.context, &mut len) };
            let len = len as usize;
            Err(Error::wrap(code, message, len))
        }
    }
}

impl Default for Context<true> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const OWNED: bool> Drop for Context<OWNED> {
    fn drop(&mut self) {
        if OWNED {
            unsafe { nix::nix_c_context_free(self.context) };
        }
    }
}

pub trait AsContext {
    fn check<F: FnOnce(*mut nix::nix_c_context) -> nix::nix_err>(&self, f: F) -> Result;
    fn with_check<T, F: FnOnce(*mut nix::nix_c_context) -> T>(&self, f: F) -> Result<T>;
}

impl<const OWNED: bool> AsContext for Context<OWNED> {
    fn check<F: FnOnce(*mut nix::nix_c_context) -> nix::nix_err>(&self, f: F) -> Result {
        self.inner_check((), f(self.context))
    }

    fn with_check<T, F: FnOnce(*mut nix::nix_c_context) -> T>(&self, f: F) -> Result<T> {
        self.inner_check(f(self.context), unsafe { nix::nix_err_code(self.context) })
    }
}
