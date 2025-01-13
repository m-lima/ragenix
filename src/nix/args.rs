use crate::nix::{self, inner};

struct Args<'a, S: nix::state::AsState> {
    values: &'static mut [*mut inner::nix_value],
    state: &'a S,
}

impl<'a, S: nix::state::AsState> Args<'a, S> {
    pub fn wrap(values: *mut *mut inner::nix_value, state: &'a S) -> nix::Result<Self> {
        let len = detect_size(values)
            .ok_or_else(|| nix::Error::custom(c"Cannot have more than 16 argumente"))?;

        let values = unsafe { std::slice::from_raw_parts_mut(values, len) };
        Ok(Self { values, state })
    }
}

impl<S: nix::state::AsState> Args<'_, S> {
    pub fn get(&self, index: usize) -> Option<nix::Value<'_, S, false>> {
        self.values
            .get(index)
            .map(|v| nix::Value::wrap(*v, self.state))
    }
}

fn detect_size<T>(values: *mut *mut T) -> Option<usize> {
    let mut i = 0;
    loop {
        if i >= 16 {
            return None;
        }

        if unsafe { *values.add(i) }.is_null() {
            return Some(i);
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detext_size_ok() {
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;
        let values = [
            core::ptr::from_mut(&mut a),
            core::ptr::from_mut(&mut b),
            core::ptr::from_mut(&mut c),
            core::ptr::null_mut(),
        ]
        .as_mut_ptr();

        assert_eq!(detect_size(values).unwrap(), 3);
    }
}
