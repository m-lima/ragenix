#![deny(warnings, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod nix {
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
mod context;
mod error;
mod primop;
mod state;
mod value;

#[cfg(feature = "log")]
pub mod log;

pub use args::Args;
pub use context::Context;
pub use error::{Error, Result};
pub use primop::PrimOp;
pub use state::State;
pub use value::{Type as ValueType, Value};
