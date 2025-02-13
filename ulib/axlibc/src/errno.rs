/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axerrno::LinuxError;
use core::ffi::{c_char, c_int};

/// The global errno variable.
#[cfg_attr(feature = "tls", thread_local)]
#[no_mangle]
#[allow(non_upper_case_globals)]
pub static mut errno: c_int = 0;

pub fn set_errno(code: i32) {
    unsafe {
        errno = code;
    }
}

/// Returns a pointer to the global errno variable.
#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    &mut errno
}

/// Returns a pointer to the string representation of the given error code.
#[no_mangle]
pub unsafe extern "C" fn strerror(e: c_int) -> *mut c_char {
    #[allow(non_upper_case_globals)]
    static mut strerror_buf: [u8; 256] = [0; 256]; // TODO: thread safe

    let err_str = if e == 0 {
        "Success"
    } else {
        LinuxError::try_from(e)
            .map(|e| e.as_str())
            .unwrap_or("Unknown error")
    };
    unsafe {
        strerror_buf[..err_str.len()].copy_from_slice(err_str.as_bytes());
        strerror_buf.as_mut_ptr() as *mut c_char
    }
}
