use crate::{context::AsContext, nix, Result};

pub struct State<'a, C: AsContext, const OWNED: bool> {
    state: *mut nix::EvalState,
    context: &'a C,
}

impl<'a, C: AsContext> State<'a, C, true> {
    pub fn new(context: &'a C) -> Result<Self> {
        context.check(|c| unsafe { nix::nix_libexpr_init(c) })?;
        todo!()
    }
}

impl<'a, C: AsContext> State<'a, C, false> {
    pub fn wrap(state: *mut nix::EvalState, context: &'a C) -> Self {
        Self { state, context }
    }
}

impl<C: AsContext, const OWNED: bool> Drop for State<'_, C, OWNED> {
    fn drop(&mut self) {
        if OWNED {
            unsafe { nix::nix_state_free(self.state) }
        }
    }
}

pub trait AsState {
    type Context: AsContext;

    fn with_state<F: FnOnce(*mut nix::EvalState) -> R, R>(&self, f: F) -> R;
    fn context(&self) -> &Self::Context;
}

impl<C: AsContext, const OWNED: bool> AsState for State<'_, C, OWNED> {
    type Context = C;

    fn with_state<F: FnOnce(*mut nix::EvalState) -> R, R>(&self, f: F) -> R {
        f(self.state)
    }

    fn context(&self) -> &Self::Context {
        self.context
    }
}
