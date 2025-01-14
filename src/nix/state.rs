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

pub trait AsState {
    type Context: nix::context::AsContext;

    fn with_state<F: FnOnce(*mut inner::EvalState) -> R, R>(&self, f: F) -> R;
    fn context(&self) -> &Self::Context;
}

impl<C: nix::context::AsContext, const OWNED: bool> AsState for State<'_, C, OWNED> {
    type Context = C;

    fn with_state<F: FnOnce(*mut inner::EvalState) -> R, R>(&self, f: F) -> R {
        f(self.state)
    }

    fn context(&self) -> &Self::Context {
        self.context
    }
}
