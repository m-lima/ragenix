#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::module_name_repetitions,
    clippy::unreadable_literal
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type PrimOpFunc = unsafe extern "C" fn(
    user_data: *mut ::core::ffi::c_void,
    context: *mut nix_c_context,
    state: *mut EvalState,
    args: *mut *mut nix_value,
    ret: *mut nix_value,
);
