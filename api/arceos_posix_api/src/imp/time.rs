/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axerrno::LinuxError;
#[cfg(feature = "signal")]
use axruntime::Signal;
use core::ffi::{c_int, c_long};
use core::time::Duration;

use crate::ctypes;

impl From<ctypes::timespec> for Duration {
    fn from(ts: ctypes::timespec) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

impl From<ctypes::timeval> for Duration {
    fn from(tv: ctypes::timeval) -> Self {
        Duration::new(tv.tv_sec as u64, tv.tv_usec as u32 * 1000)
    }
}

impl From<Duration> for ctypes::timespec {
    fn from(d: Duration) -> Self {
        ctypes::timespec {
            tv_sec: d.as_secs() as c_long,
            tv_nsec: d.subsec_nanos() as c_long,
        }
    }
}

impl From<Duration> for ctypes::timeval {
    fn from(d: Duration) -> Self {
        ctypes::timeval {
            tv_sec: d.as_secs() as c_long,
            tv_usec: d.subsec_micros() as c_long,
        }
    }
}

/// Get clock time since booting
pub unsafe fn sys_clock_gettime(_clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_gettime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let now = axhal::time::current_time().into();
        unsafe { *ts = now };
        debug!("sys_clock_gettime: {}.{:09}s", now.tv_sec, now.tv_nsec);
        Ok(0)
    })
}

/// Get clock time since booting
pub unsafe fn sys_clock_settime(_clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_setttime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let new_tv = Duration::from(*ts);
        debug!(
            "sys_clock_setttime: {}.{:09}s",
            new_tv.as_secs(),
            new_tv.as_nanos()
        );
        axhal::time::set_current_time(new_tv);
        Ok(0)
    })
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
pub unsafe fn sys_nanosleep(req: *const ctypes::timespec, rem: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_nanosleep, {
        unsafe {
            if req.is_null() || (*req).tv_nsec < 0 || (*req).tv_nsec > 999999999 {
                return Err(LinuxError::EINVAL);
            }
        }

        let dur = unsafe {
            debug!("sys_nanosleep <= {}.{:09}s", (*req).tv_sec, (*req).tv_nsec);
            Duration::from(*req)
        };

        let now = axhal::time::current_time();

        #[cfg(feature = "multitask")]
        axtask::sleep(dur);
        #[cfg(not(feature = "multitask"))]
        axhal::time::busy_wait(dur);

        let after = axhal::time::current_time();
        let actual = after - now;

        if let Some(diff) = dur.checked_sub(actual) {
            if !rem.is_null() {
                unsafe { (*rem) = diff.into() };
            }
            return Err(LinuxError::EINTR);
        }
        Ok(0)
    })
}

#[cfg(feature = "signal")]
/// Set a timer to send a signal to the current process after a specified time
pub unsafe fn sys_setitimer(which: c_int, new: *const ctypes::itimerval) -> c_int {
    syscall_body!(sys_setitimer, {
        let which = which as usize;
        let new_interval = Duration::from((*new).it_interval).as_nanos() as u64;
        Signal::timer_interval(which, Some(new_interval));

        let new_ddl =
            axhal::time::current_time_nanos() + Duration::from((*new).it_value).as_nanos() as u64;
        Signal::timer_deadline(which, Some(new_ddl));
        Ok(0)
    })
}

#[cfg(feature = "signal")]
/// Get timer to send signal after some time
pub unsafe fn sys_getitimer(which: c_int, curr_value: *mut ctypes::itimerval) -> c_int {
    syscall_body!(sys_getitimer, {
        let ddl = Duration::from_nanos(Signal::timer_deadline(which as usize, None).unwrap());
        if ddl.as_nanos() == 0 {
            return Err(LinuxError::EINVAL);
        }
        let mut now: ctypes::timespec = ctypes::timespec::default();
        unsafe {
            sys_clock_gettime(0, &mut now);
        }
        let now = Duration::from(now);
        if ddl > now {
            (*curr_value).it_value = ctypes::timeval::from(ddl - now);
        } else {
            (*curr_value).it_value = ctypes::timeval::from(Duration::new(0, 0));
        }
        (*curr_value).it_interval =
            Duration::from_nanos(Signal::timer_interval(which as usize, None).unwrap()).into();
        Ok(0)
    })
}
