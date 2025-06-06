#[repr(C)]
pub struct String {
    data: *mut u8,
    len: usize,
    cap: usize,
}

impl String {
    pub fn dealloc(self) {
        core::mem::drop(unsafe { Vec::from_raw_parts(self.data, self.len, self.cap) });
    }
}

impl From<Vec<u8>> for String {
    fn from(mut value: Vec<u8>) -> Self {
        if !matches!(value.last(), Some(0)) {
            value.push(0);
        }
        let data = value.as_mut_ptr();
        let len = value.len();
        let cap = value.capacity();
        core::mem::forget(value);
        Self { data, len, cap }
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        value.into_bytes().into()
    }
}
