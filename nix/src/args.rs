use crate::{nix, state::AsState, Result, Value};

pub struct Args<'a, S: AsState, const L: usize> {
    array: &'static mut [*mut nix::nix_value],
    state: &'a S,
}

impl<'a, S: AsState, const L: usize> Args<'a, S, L> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn wrap(values: *mut *mut nix::nix_value, state: &'a S) -> Result<Self> {
        let array = unsafe { core::slice::from_raw_parts_mut(values, usize::saturating_sub(L, 1)) };
        Ok(Self { array, state })
    }
}

impl<S: AsState, const L: usize> Args<'_, S, L> {
    pub fn with_state<NS: AsState>(self, state: &NS) -> Args<'_, NS, L> {
        Args {
            array: self.array,
            state,
        }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<Value<'_, S, false>> {
        self.array.get(index).map(|v| Value::wrap(*v, self.state))
    }

    pub fn iter(&self) -> impl Iterator<Item = Value<'_, S, false>> {
        self.array.iter().map(|v| Value::wrap(*v, self.state))
    }
}
