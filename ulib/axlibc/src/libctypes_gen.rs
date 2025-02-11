/* automatically generated by rust-bindgen 0.66.1 */

pub type __jmp_buf = [::core::ffi::c_ulong; 8usize];
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct __jmp_buf_tag {
    pub __jb: __jmp_buf,
    pub __fl: ::core::ffi::c_ulong,
    pub __ss: [::core::ffi::c_ulong; 16usize],
}
#[test]
fn bindgen_test_layout___jmp_buf_tag() {
    const UNINIT: ::core::mem::MaybeUninit<__jmp_buf_tag> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<__jmp_buf_tag>(),
        200usize,
        concat!("Size of: ", stringify!(__jmp_buf_tag))
    );
    assert_eq!(
        ::core::mem::align_of::<__jmp_buf_tag>(),
        8usize,
        concat!("Alignment of ", stringify!(__jmp_buf_tag))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).__jb) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(__jmp_buf_tag),
            "::",
            stringify!(__jb)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).__fl) as usize - ptr as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(__jmp_buf_tag),
            "::",
            stringify!(__fl)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).__ss) as usize - ptr as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(__jmp_buf_tag),
            "::",
            stringify!(__ss)
        )
    );
}
pub type jmp_buf = [__jmp_buf_tag; 1usize];
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tm {
    pub tm_sec: ::core::ffi::c_int,
    pub tm_min: ::core::ffi::c_int,
    pub tm_hour: ::core::ffi::c_int,
    pub tm_mday: ::core::ffi::c_int,
    pub tm_mon: ::core::ffi::c_int,
    pub tm_year: ::core::ffi::c_int,
    pub tm_wday: ::core::ffi::c_int,
    pub tm_yday: ::core::ffi::c_int,
    pub tm_isdst: ::core::ffi::c_int,
    pub __tm_gmtoff: ::core::ffi::c_long,
    pub __tm_zone: *const ::core::ffi::c_char,
}
#[test]
fn bindgen_test_layout_tm() {
    const UNINIT: ::core::mem::MaybeUninit<tm> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<tm>(),
        56usize,
        concat!("Size of: ", stringify!(tm))
    );
    assert_eq!(
        ::core::mem::align_of::<tm>(),
        8usize,
        concat!("Alignment of ", stringify!(tm))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_sec) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_sec)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_min) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_min)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_hour) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_hour)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_mday) as usize - ptr as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_mday)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_mon) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_mon)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_year) as usize - ptr as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_year)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_wday) as usize - ptr as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_wday)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_yday) as usize - ptr as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_yday)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).tm_isdst) as usize - ptr as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(tm_isdst)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).__tm_gmtoff) as usize - ptr as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(__tm_gmtoff)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).__tm_zone) as usize - ptr as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(tm),
            "::",
            stringify!(__tm_zone)
        )
    );
}
impl Default for tm {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
