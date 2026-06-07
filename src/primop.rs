use crate::nix;

macro_rules! bail {
    ($err: expr, $msg: literal) => {
        let err = if $err == nix::nix_err::NIX_OK {
            nix::nix_err::NIX_ERR_UNKNOWN
        } else {
            $err
        };
        return Err(NixError::new(err, std::borrow::Cow::Borrowed($msg)));
    };

    ($msg: literal) => {
        bail!(nix::nix_err::NIX_ERR_UNKNOWN, $msg)
    };

    ($ctx: expr => $err: expr, $msg: literal) => {
        let err = if $err == nix::nix_err::NIX_OK {
            nix::nix_err::NIX_ERR_UNKNOWN
        } else {
            $err
        };
        if let Some(msg) = extract_error_with_prefix($ctx, $msg) {
            return Err(NixError::new(err, std::borrow::Cow::Owned(msg)));
        }
        return Err(NixError::new(
            err,
            std::borrow::Cow::Borrowed(unsafe {
                std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($msg, '\0').as_bytes())
            }),
        ));
    };

    ($ctx: expr => $msg: literal) => {
        let error = unsafe { nix::nix_err_code($ctx) };
        bail!($ctx => error, $msg)
    };
}

struct NixError {
    code: nix::nix_err,
    msg: std::borrow::Cow<'static, std::ffi::CStr>,
}

impl NixError {
    fn new(code: nix::nix_err, msg: std::borrow::Cow<'static, std::ffi::CStr>) -> Self {
        Self { code, msg }
    }

    fn convert(err: &crate::error::Error, path: &std::ffi::CStr) -> Self {
        match std::ffi::CString::new(format!(
            "could not decrypt {path}: {err}",
            path = path.to_str().unwrap_or("<invalid path>")
        )) {
            Ok(msg) => NixError::new(nix::nix_err::NIX_ERR_UNKNOWN, std::borrow::Cow::Owned(msg)),
            Err(_) => NixError::new(
                nix::nix_err::NIX_ERR_UNKNOWN,
                std::borrow::Cow::Borrowed(c"error payload contains interior NUL"),
            ),
        }
    }
}

type Result<T = ()> = std::result::Result<T, NixError>;

pub fn register() {
    let ctx = unsafe { nix::nix_c_context_create() };
    let mut args = [c"key".as_ptr(), c"path".as_ptr(), std::ptr::null()];
    let primop = unsafe {
        nix::nix_alloc_primop(
            ctx,
            Some(decrypt_primop),
            2,
            c"__decrypt".as_ptr(),
            args.as_mut_ptr(),
            c"Decrypt and evaluate a file".as_ptr(),
            std::ptr::null_mut(),
        )
    };
    assert!(
        !primop.is_null(),
        "nix_alloc_primop failed: {err}",
        err = extract_error(ctx).unwrap_or("<unknown error>")
    );
    let result = unsafe { nix::nix_register_primop(ctx, primop) };
    assert_eq!(
        result,
        nix::nix_err::NIX_OK,
        "nix_register_primop failed: {err}",
        err = extract_error(ctx).unwrap_or("<unknown error>")
    );
    unsafe { nix::nix_gc_decref(std::ptr::null_mut(), primop.cast()) };
    unsafe { nix::nix_c_context_free(ctx) };
}

fn extract_error<'a>(ctx: *const nix::nix_c_context) -> Option<&'a str> {
    let mut len = 0;
    let msg = unsafe { nix::nix_err_msg(std::ptr::null_mut(), ctx, &raw mut len) };
    if msg.is_null() || len == 0 {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(msg.cast(), len as usize) };
    unsafe { Some(str::from_utf8_unchecked(slice)) }
}

fn extract_error_with_prefix(
    ctx: *const nix::nix_c_context,
    prefix: &str,
) -> Option<std::ffi::CString> {
    let msg = extract_error(ctx)?;
    std::ffi::CString::new(format!("{prefix}: {msg}")).ok()
}

fn get_path<'a>(
    ctx: *mut nix::nix_c_context,
    state: *mut nix::EvalState,
    arg: *mut nix::nix_value,
) -> Result<&'a std::ffi::CStr> {
    if arg.is_null() {
        bail!(c"expected path parameter");
    }

    let force_result = unsafe { nix::nix_value_force(ctx, state, arg) };
    if force_result != nix::nix_err::NIX_OK {
        bail!(ctx => force_result, "failed to force argument evaluation");
    }

    let value_type = unsafe { nix::nix_get_type(ctx, arg) };
    match value_type {
        nix::ValueType::NIX_TYPE_THUNK => {
            bail!(c"parameter is a thunk while a path was expected");
        }
        nix::ValueType::NIX_TYPE_INT => {
            bail!(c"parameter is an integer while a path was expected");
        }
        nix::ValueType::NIX_TYPE_FLOAT => {
            bail!(c"parameter is a float while a path was expected");
        }
        nix::ValueType::NIX_TYPE_BOOL => {
            bail!(c"parameter is a Boolean while a path was expected");
        }
        nix::ValueType::NIX_TYPE_STRING => {
            bail!(c"parameter is a string while a path was expected");
        }
        nix::ValueType::NIX_TYPE_NULL => {
            bail!(c"parameter is null while a path was expected");
        }
        nix::ValueType::NIX_TYPE_ATTRS => {
            bail!(c"parameter is a set while a path was expected");
        }
        nix::ValueType::NIX_TYPE_LIST => {
            bail!(c"parameter is a list while a path was expected");
        }
        nix::ValueType::NIX_TYPE_FUNCTION => {
            bail!(c"parameter is a function while a path was expected");
        }
        nix::ValueType::NIX_TYPE_EXTERNAL => {
            bail!(c"parameter is an external value while a path was expected");
        }
        nix::ValueType::NIX_TYPE_FAILED => {
            bail!(c"parameter is an error while a path was expected");
        }
        nix::ValueType::NIX_TYPE_PATH => {}
    }
    let path = unsafe { nix::nix_get_path_string(ctx, arg) };
    if path.is_null() {
        bail!(ctx => "failed to read path");
    }
    Ok(unsafe { std::ffi::CStr::from_ptr(path) })
}

unsafe extern "C" fn decrypt_primop(
    _user_data: *mut std::ffi::c_void,
    ctx: *mut nix::nix_c_context,
    state: *mut nix::EvalState,
    args: *mut *mut nix::nix_value,
    ret: *mut nix::nix_value,
) {
    fn inner(
        ctx: *mut nix::nix_c_context,
        state: *mut nix::EvalState,
        args: *mut *mut nix::nix_value,
        ret: *mut nix::nix_value,
    ) -> Result {
        let key = unsafe { get_path(ctx, state, *args)? };
        let path = unsafe { get_path(ctx, state, *args.add(1))? };
        let file_path =
            <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(path.to_bytes());
        let builder = unsafe { nix::nix_make_bindings_builder(ctx, state, 1) };
        if builder.is_null() {
            bail!(ctx => "failed to allocate bindings builder");
        }

        let value = unsafe { nix::nix_alloc_value(ctx, state) };
        if value.is_null() {
            bail!(ctx => "failed to allocate value");
        }

        let field = match super::decrypt(key, file_path)
            .map_err(|err| NixError::convert(&err, path))
            .and_then(|payload| insert_ok(ctx, state, value, path, &payload))
        {
            Ok(()) => c"ok",
            Err(err) => {
                insert_err(ctx, value, &err)?;
                c"err"
            }
        };

        let result =
            unsafe { nix::nix_bindings_builder_insert(ctx, builder, field.as_ptr(), value) };
        if result != nix::nix_err::NIX_OK {
            bail!(ctx => result, "failed to insert value into return attribute set");
        }
        finish_attrs(ctx, ret, builder)?;
        Ok(())
    }

    if let Err(err) = inner(ctx, state, args, ret) {
        unsafe {
            nix::nix_set_err_msg(ctx, err.code, err.msg.as_ptr());
        }
    }
}

fn insert_ok(
    ctx: *mut nix::nix_c_context,
    state: *mut nix::EvalState,
    value: *mut nix::nix_value,
    path: &std::ffi::CStr,
    payload: &str,
) -> Result {
    let Ok(payload) = std::ffi::CString::new(payload) else {
        bail!(c"decrypted payload contains interior NUL");
    };

    let result = unsafe {
        nix::nix_expr_eval_from_string(ctx, state, payload.as_ptr(), path.as_ptr(), value)
    };
    if result != nix::nix_err::NIX_OK {
        bail!(ctx => result, "could not evaluate decrypted payload");
    }

    Ok(())
}

fn insert_err(ctx: *mut nix::nix_c_context, value: *mut nix::nix_value, err: &NixError) -> Result {
    let result = unsafe { nix::nix_init_string(ctx, value, err.msg.as_ptr()) };
    if result != nix::nix_err::NIX_OK {
        bail!(ctx => result, "failed to initialize string value");
    }

    Ok(())
}

fn finish_attrs(
    ctx: *mut nix::nix_c_context,
    ret: *mut nix::nix_value,
    builder: *mut nix::BindingsBuilder,
) -> Result {
    let result = unsafe { nix::nix_make_attrs(ctx, ret, builder) };
    if result != nix::nix_err::NIX_OK {
        bail!(ctx => result, "failed to build attribute set");
    }
    unsafe { nix::nix_bindings_builder_free(builder) };
    Ok(())
}
