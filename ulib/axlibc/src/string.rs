use crate::ctypes;
use core::ffi::c_char;
/// calculate the length of a string, excluding the terminating null byte
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> ctypes::size_t {
    strnlen(s, ctypes::size_t::MAX)
}

/// calculate the length of a string like strlen, but at most maxlen.
#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const c_char, size: ctypes::size_t) -> ctypes::size_t {
    let mut i = 0;
    while i < size {
        if *s.add(i) == 0 {
            break;
        }
        i += 1;
    }
    i
}
