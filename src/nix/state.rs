use crate::nix::{self, inner};

pub struct State<'a, C: nix::context::AsContext, const OWNED: bool> {
    state: *mut inner::EvalState,
    context: &'a C,
}

impl<'a, C: nix::context::AsContext> State<'a, C, true> {
    pub fn new(context: &'a C) -> nix::Result<Self> {
        context.check(|c| unsafe { inner::nix_libexpr_init(c) })?;
        todo!()
        // context.check(|c| unsafe{inner::nix_state_create(c, core::ptr::null_mut(), store})
        // .map(|state| Self {
        //     state,
        //     context,
        //     owned: true,
        // })
    }
}

impl<'a, C: nix::context::AsContext> State<'a, C, false> {
    pub fn wrap(state: *mut inner::EvalState, context: &'a C) -> Self {
        Self { state, context }
    }
}

impl<C: nix::context::AsContext, const OWNED: bool> Drop for State<'_, C, OWNED> {
    fn drop(&mut self) {
        if OWNED {
            unsafe { inner::nix_state_free(self.state) }
        }
    }
}

// pub(super) trait AsState: nix::context::AsContext {
pub(super) trait AsState {
    type Context: nix::context::AsContext;

    fn as_state(&self) -> *mut inner::EvalState;
    fn context<'a>(&'a self) -> &'a Self::Context;
}

impl<'a, C: nix::context::AsContext, const OWNED: bool> AsState for State<'a, C, OWNED> {
    type Context = C;

    fn as_state(&self) -> *mut inner::EvalState {
        self.state
    }

    fn context<'b>(&'b self) -> &'b Self::Context {
        self.context
    }
}

// impl<'a, C: nix::context::AsContext, const OWNED: bool> nix::context::AsContext
//     for State<'a, C, OWNED>
// {
//     fn check<F: FnOnce(*mut inner::nix_c_context) -> inner::nix_err>(&self, f: F) -> nix::Result {
//         self.context.check(f)
//     }
//
//     fn with_check<T, F: FnOnce(*mut inner::nix_c_context) -> T>(&self, f: F) -> nix::Result<T> {
//         self.context.with_check(f)
//     }
// }
