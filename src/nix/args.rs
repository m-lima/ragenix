use crate::nix::{self, inner};

pub struct Args<'a, S: nix::state::AsState> {
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

    pub fn iter(&self) -> impl Iterator<Item = nix::Value<'_, S, false>> {
        self.values.iter().map(|v| nix::Value::wrap(*v, self.state))
    }
}

fn detect_size<T>(values: *mut *mut T) -> Option<usize> {
    let mut i = 0;
    loop {
        if i > 16 {
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
    fn empty() {
        let values: *mut *mut usize = [core::ptr::null_mut()].as_mut_ptr();

        assert_eq!(detect_size(values).unwrap(), 0);
    }

    #[test]
    fn detect_size_ok() {
        let mut v0 = 0;
        let mut v1 = 1;
        let mut v2 = 2;
        let mut v3 = 3;
        let mut v4 = 4;
        let mut v5 = 5;
        let mut v6 = 6;
        let mut v7 = 7;
        let mut v8 = 8;
        let mut v9 = 9;
        let mut va = 10;
        let mut vb = 11;
        let mut vc = 12;
        let mut vd = 13;
        let mut ve = 14;
        let mut vf = 15;
        let values = [
            core::ptr::from_mut(&mut v0),
            core::ptr::from_mut(&mut v1),
            core::ptr::from_mut(&mut v2),
            core::ptr::from_mut(&mut v3),
            core::ptr::from_mut(&mut v4),
            core::ptr::from_mut(&mut v5),
            core::ptr::from_mut(&mut v6),
            core::ptr::from_mut(&mut v7),
            core::ptr::from_mut(&mut v8),
            core::ptr::from_mut(&mut v9),
            core::ptr::from_mut(&mut va),
            core::ptr::from_mut(&mut vb),
            core::ptr::from_mut(&mut vc),
            core::ptr::from_mut(&mut vd),
            core::ptr::from_mut(&mut ve),
            core::ptr::from_mut(&mut vf),
            core::ptr::null_mut(),
        ]
        .as_mut_ptr();

        assert_eq!(detect_size(values).unwrap(), 16);
    }

    #[test]
    fn detect_size_too_big() {
        let mut v0 = 0;
        let mut v1 = 1;
        let mut v2 = 2;
        let mut v3 = 3;
        let mut v4 = 4;
        let mut v5 = 5;
        let mut v6 = 6;
        let mut v7 = 7;
        let mut v8 = 8;
        let mut v9 = 9;
        let mut va = 10;
        let mut vb = 11;
        let mut vc = 12;
        let mut vd = 13;
        let mut ve = 14;
        let mut vf = 15;
        let mut vz = 16;
        let values = [
            core::ptr::from_mut(&mut v0),
            core::ptr::from_mut(&mut v1),
            core::ptr::from_mut(&mut v2),
            core::ptr::from_mut(&mut v3),
            core::ptr::from_mut(&mut v4),
            core::ptr::from_mut(&mut v5),
            core::ptr::from_mut(&mut v6),
            core::ptr::from_mut(&mut v7),
            core::ptr::from_mut(&mut v8),
            core::ptr::from_mut(&mut v9),
            core::ptr::from_mut(&mut va),
            core::ptr::from_mut(&mut vb),
            core::ptr::from_mut(&mut vc),
            core::ptr::from_mut(&mut vd),
            core::ptr::from_mut(&mut ve),
            core::ptr::from_mut(&mut vf),
            core::ptr::from_mut(&mut vz),
            core::ptr::null_mut(),
        ]
        .as_mut_ptr();

        assert!(detect_size(values).is_none());
    }
}
