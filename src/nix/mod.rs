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

mod context;
pub use context::{Context, Value};

pub use inner::{
    nix_c_context as RawContext, nix_err as Error, nix_err_NIX_ERR_UNKNOWN as ERR_UNKNOWN,
    nix_value as RawValue, EvalState as State, ValueType,
};

pub type PrimOpFunc = unsafe extern "C" fn(
    user_data: *mut ::core::ffi::c_void,
    context: *mut RawContext,
    state: *mut State,
    args: *mut *mut RawValue,
    ret: *mut RawValue,
);

impl crate::error::Reporter for *mut RawContext {
    fn report(self, code: Error, message: *const core::ffi::c_char) {
        unsafe { inner::nix_set_err_msg(self, code, message) };
    }
}
