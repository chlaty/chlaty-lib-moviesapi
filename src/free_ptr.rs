use std::ffi::{c_char, CString};

#[unsafe(no_mangle)]
pub extern "C" fn free_ptr(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}
