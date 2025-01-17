use crate::{nix, state::AsState, Result, Value};

pub struct Args<'a, S: AsState, const L: usize> {
    array: Array<'static, *mut nix::nix_value, L>,
    state: &'a S,
}

impl<'a, S: AsState, const L: usize> Args<'a, S, L> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn wrap(values: *mut *mut nix::nix_value, state: &'a S) -> Result<Self> {
        let array = Array(unsafe { &mut *values.cast() });
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

struct Array<'a, T: 'a, const L: usize>(&'a mut [T; L]);

impl<'a, T: 'a, const L: usize> Array<'a, T, L> {
    #[must_use]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= usize::saturating_sub(L, 1) {
            None
        } else {
            self.0.get(index)
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().take(usize::saturating_sub(L, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_check() {
        let mut array = [0, 1, 2, 4, 8];
        let array = Array(&mut array);

        assert_eq!(array.get(0), Some(&0));
        assert_eq!(array.get(1), Some(&1));
        assert_eq!(array.get(2), Some(&2));
        assert_eq!(array.get(3), Some(&4));
        assert_eq!(array.get(4), None);

        assert_eq!(array.iter().sum::<u8>(), 7);
    }
}
