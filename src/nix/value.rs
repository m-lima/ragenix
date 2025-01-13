use crate::nix::{self, context::AsContext, inner};

pub struct Value<'a, S: nix::state::AsState, const OWNED: bool> {
    value: *mut inner::nix_value,
    state: &'a S,
}

impl<'a, S: nix::state::AsState> Value<'a, S, true> {
    pub fn new(state: &'a S) -> nix::Result<Self> {
        state
            .context()
            .with_check(|c| unsafe { inner::nix_alloc_value(c, state.as_state()) })
            .map(|value| Self { value, state })
    }
}

impl<'a, S: nix::state::AsState> Value<'a, S, false> {
    pub fn wrap(value: *mut inner::nix_value, state: &'a S) -> Self {
        Self { value, state }
    }

    pub fn to_owned(&self) -> nix::Result<Value<'a, S, true>> {
        self.state
            .context()
            .check(|c| unsafe { inner::nix_gc_incref(c, self.value as *const _) })?;
        Ok(Value {
            value: self.value,
            state: self.state,
        })
    }
}

// impl Value<'_> {
//     pub fn eval(&self, state: *mut nix::State) -> Result {
//         self.context
//             .check_with_code(|c| unsafe { inner::nix_value_force(c, state, self.value) })
//     }
//
//     pub fn get_int(&self) -> Result<i64> {
//         let value_type = self.get_type()?;
//         if value_type == inner::ValueType_NIX_TYPE_INT {
//             self.context
//                 .check(|c| unsafe { inner::nix_get_int(c, self.value) })
//         } else {
//             Err(Error::custom(c"Value is not an integer"))
//         }
//     }
//
//     pub fn set_int(&self, value: i64) -> Result {
//         self.context
//             .check_with_code(|c| unsafe { inner::nix_init_int(c, self.value, value) })
//     }
//
//     pub fn get_path(&self) -> Result<*const core::ffi::c_char> {
//         let value_type = self.get_type()?;
//         if value_type == inner::ValueType_NIX_TYPE_PATH {
//             self.context
//                 .check(|c| unsafe { inner::nix_get_path_string(c, self.value) })
//         } else {
//             Err(Error::custom(c"Value is not a path"))
//         }
//     }
//
//     pub fn set_path(&self, state: *mut nix::State, path: *const core::ffi::c_char) -> Result {
//         self.context
//             // .check_with_code(|c| unsafe { inner::nix_init_path_string(c, state, self.value, path) })
//             .check_with_code(|c| unsafe { inner::nix_init_string(c, self.value, path) })
//     }
// }
//
// impl Value<'_> {
//     fn get_type(&self) -> Result<nix::ValueType> {
//         self.context
//             .check(|c| unsafe { inner::nix_get_type(c, self.value) })
//     }
// }

impl<S: nix::state::AsState, const OWNED: bool> Drop for Value<'_, S, OWNED> {
    fn drop(&mut self) {
        if OWNED {
            unsafe { inner::nix_value_decref(self.state.context().as_context(), self.value) };
        }
    }
}
