use crate::nix::{self, context::AsContext, inner};

pub struct Value<'a, S: nix::state::AsState, const OWNED: bool> {
    value: *mut inner::nix_value,
    state: &'a S,
}

impl<'a, S: nix::state::AsState> Value<'a, S, true> {
    pub fn new(state: &'a S) -> nix::Result<Self> {
        state
            .context()
            .with_check(|c| state.with_state(|s| unsafe { inner::nix_alloc_value(c, s) }))
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

impl<S: nix::state::AsState> Value<'_, S, false> {
    pub fn get_type(&self) -> nix::Result<Type> {
        self.state
            .context()
            .with_check(|c| unsafe { inner::nix_get_type(c, self.value) })
            .and_then(|v| {
                Type::from_inner(v).ok_or_else(|| nix::Error::custom(c"Unknown value type"))
            })
    }

    pub fn eval(&self) -> nix::Result {
        self.state.context().check(|c| {
            self.state
                .with_state(|s| unsafe { inner::nix_value_force(c, s, self.value) })
        })
    }

    pub fn get_int(&self) -> nix::Result<i64> {
        let value_type = self.get_type()?;
        if value_type == Type::Int {
            self.state
                .context()
                .with_check(|c| unsafe { inner::nix_get_int(c, self.value) })
        } else {
            Err(nix::Error::custom(c"Value is not an integer"))
        }
    }

    pub fn set_int(&self, value: i64) -> nix::Result {
        self.state
            .context()
            .check(|c| unsafe { inner::nix_init_int(c, self.value, value) })
    }
}

impl<S: nix::state::AsState, const OWNED: bool> Drop for Value<'_, S, OWNED> {
    fn drop(&mut self) {
        if OWNED {
            if let Err(err) = self
                .state
                .context()
                .check(|c| unsafe { inner::nix_value_decref(c, self.value) })
            {
                #[cfg(feature = "log")]
                let _ = crate::log::write(|f| writeln!(f, "{err}"));
                drop(err);
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Type {
    Thunk,
    Int,
    Float,
    Bool,
    String,
    Path,
    Null,
    Attr,
    List,
    Function,
    External,
}

impl Type {
    fn from_inner(inner: inner::ValueType) -> Option<Self> {
        match inner {
            0 => Some(Self::Thunk),
            1 => Some(Self::Int),
            2 => Some(Self::Float),
            3 => Some(Self::Bool),
            4 => Some(Self::String),
            5 => Some(Self::Path),
            6 => Some(Self::Null),
            7 => Some(Self::Attr),
            8 => Some(Self::List),
            9 => Some(Self::Function),
            10 => Some(Self::External),
            _ => None,
        }
    }
}