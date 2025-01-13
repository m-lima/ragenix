mod inner {
    #![allow(
        dead_code,
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        clippy::module_name_repetitions,
        clippy::unreadable_literal
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod args;
mod array;
mod context;
mod error;
mod primop;
mod state;
mod value;

pub use context::Context;
pub use error::{Error, Result};
pub use state::State;
pub use value::Value;

// use inner::{
//     nix_c_context as RawContext, nix_err as RawError, nix_err_NIX_ERR_UNKNOWN as ERR_UNKNOWN,
//     nix_value as RawValue, EvalState as RawState, ValueType,
// };
