use crate::nix::{self, inner};

struct Array<T> {
    values: *mut *mut T,
    len: u8,
}

impl<T> Array<T> {
    pub fn wrap(values: *mut *mut T) -> nix::Result<Self> {
        let len = detect_size(values)
            .ok_or_else(|| nix::Error::custom(c"Cannot have more than 16 argumente"))?;

        Ok(Self { values, len })
    }

    // pub fn get(&self, index: usize) -> Option<nix::Value<'_, S, false>> {
    //     (index < self.len).then_some({
    //         let value = unsafe { *self.values.add(index) };
    //         nix::Value::wrap(value, self.state)
    //     })
    // }
}

impl<T> std::ops::Index<u8> for Array<T> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        if index >= self.len {
            Vec::new().get
        } else {
        }
    }
}

fn detect_size<T>(values: *mut *mut T) -> Option<u8> {
    let mut i = 0;
    loop {
        if unsafe { *values.add(usize::from(i)) }.is_null() {
            return Some(i);
        }

        if i == 255 {
            return None;
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
