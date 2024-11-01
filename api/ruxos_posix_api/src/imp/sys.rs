/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

 use core::ffi::{c_int, c_long, CStr};

 use crate::ctypes;
 
 #[cfg(feature = "alloc")]
 use alloc::ffi::CString;
 
 #[cfg(feature = "alloc")]
 // The values are used to fool glibc since glibc will check the version and os name.
 lazy_static::lazy_static! {
    static ref SYS_NAME: CString = CString::new("Linux").unwrap();
    static ref NODE_NAME: CString = CString::new("WHITLEY").unwrap();
    static ref RELEASE: CString = CString::new("5.13.0").unwrap();
    static ref VERSION: CString = CString::new("5.13.0").unwrap();
    static ref MACHINE: CString = get_machine_name();
    static ref DOMAIN_NAME: CString = CString::new("").unwrap();
    static ref UTS_NAME: UtsName = {
        let mut uts_name = UtsName::new();
        copy_cstring_to_u8_slice(&SYS_NAME, &mut uts_name.sysname);
        copy_cstring_to_u8_slice(&NODE_NAME, &mut uts_name.nodename);
        copy_cstring_to_u8_slice(&RELEASE, &mut uts_name.release);
        copy_cstring_to_u8_slice(&VERSION, &mut uts_name.version);
        copy_cstring_to_u8_slice(&MACHINE, &mut uts_name.machine);
        copy_cstring_to_u8_slice(&DOMAIN_NAME, &mut uts_name.domainname);
        uts_name
    };
}

fn get_machine_name() -> CString {
    if cfg!(target_arch = "aarch64") {
        CString::new("aarch64").unwrap()
    } else if cfg!(target_arch = "x86_64") {
        CString::new("x86_64").unwrap()
    } else {
        CString::new("unknown").unwrap()
    }
}

 
 const UTS_FIELD_LEN: usize = 65;
 
 #[derive(Debug, Clone, Copy)]
 #[repr(C)]
 struct UtsName {
     sysname: [u8; UTS_FIELD_LEN],
     nodename: [u8; UTS_FIELD_LEN],
     release: [u8; UTS_FIELD_LEN],
     version: [u8; UTS_FIELD_LEN],
     machine: [u8; UTS_FIELD_LEN],
     domainname: [u8; UTS_FIELD_LEN],
 }
 
 impl UtsName {
     const fn new() -> Self {
         UtsName {
             sysname: [0; UTS_FIELD_LEN],
             nodename: [0; UTS_FIELD_LEN],
             release: [0; UTS_FIELD_LEN],
             version: [0; UTS_FIELD_LEN],
             machine: [0; UTS_FIELD_LEN],
             domainname: [0; UTS_FIELD_LEN],
         }
     }
 }
 
 fn copy_cstring_to_u8_slice(src: &CStr, dst: &mut [u8]) {
     let src = src.to_bytes_with_nul();
     let len = src.len().min(dst.len());
     dst[..len].copy_from_slice(&src[..len]);
 }
 
 /// Return sysinfo struct
 #[no_mangle]
 pub unsafe extern "C" fn sys_sysinfo(info: *mut ctypes::sysinfo) -> c_int {
     debug!("sys_sysinfo");
     syscall_body!(sys_sysinfo, {
         let info_mut = info.as_mut().unwrap();
 
         // If the kernel booted less than 1 second, it will be 0.
         info_mut.uptime = ruxhal::time::current_time().as_secs() as c_long;
 
         info_mut.loads = [0; 3];
         #[cfg(feature = "multitask")]
         {
             ruxtask::get_avenrun(&mut info_mut.loads);
         }
 
         info_mut.sharedram = 0;
         // TODO
         info_mut.bufferram = 0;
 
         info_mut.totalram = 0;
         info_mut.freeram = 0;
         #[cfg(feature = "alloc")]
         {
             use core::ffi::c_ulong;
             let allocator = axalloc::global_allocator();
             info_mut.freeram = (allocator.available_bytes()
                 + allocator.available_pages() * memory_addr::PAGE_SIZE_4K)
                 as c_ulong;
             info_mut.totalram = info_mut.freeram + allocator.used_bytes() as c_ulong;
         }
 
         // TODO
         info_mut.totalswap = 0;
         info_mut.freeswap = 0;
 
         info_mut.procs = 1;
 
         // unused in 64-bit
         info_mut.totalhigh = 0;
         info_mut.freehigh = 0;
 
         info_mut.mem_unit = 1;
 
         Ok(0)
     })
 }
 
 /// Print system information
 pub fn sys_uname(uts: *mut core::ffi::c_void) -> c_int {
     debug!("sys_uname return fake uname");
     #[cfg(feature = "alloc")]
     {
         let uts_ptr = uts as *mut UtsName;
         unsafe { *uts_ptr = *UTS_NAME };
     }
     
     syscall_body!(sys_uname, Ok(0))
 }
 