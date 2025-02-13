/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

/// Add `poll` feature to use poll() interface.
/// poll() is a system call function used to monitor I/O events across multiple file descriptors.
/// poll() is a blocking type of interface, make sure that this does not cost too much.
/// To monitor I/O events, you can also use `select` or `epoll` instead.
use crate::{ctypes, imp::fd_ops::get_file_like};
use axerrno::{LinuxError, LinuxResult};
use axhal::time::current_time;

use core::{ffi::c_int, time::Duration};

fn poll_all(fds: &mut [ctypes::pollfd]) -> LinuxResult<usize> {
    let mut events_num = 0;

    for pollfd_item in fds.iter_mut() {
        let intfd = pollfd_item.fd;
        let events = pollfd_item.events;
        let revents = &mut pollfd_item.revents;
        match get_file_like(intfd as c_int)?.poll() {
            Err(_) => {
                if (events & ctypes::EPOLLERR as i16) != 0 {
                    *revents |= ctypes::EPOLLERR as i16;
                }
            }
            Ok(state) => {
                if state.readable && (events & ctypes::EPOLLIN as i16 != 0) {
                    *revents |= ctypes::EPOLLIN as i16;
                }

                if state.writable && (events & ctypes::EPOLLOUT as i16 != 0) {
                    *revents |= ctypes::EPOLLOUT as i16;
                }
            }
        }
        events_num += 1;
    }
    Ok(events_num)
}

/// `ppoll` used by A64. Currently ignore signal
pub unsafe fn sys_ppoll(
    fds: *mut ctypes::pollfd,
    nfds: ctypes::nfds_t,
    timeout: *const ctypes::timespec,
    _sig_mask: *const ctypes::sigset_t,
    _sig_num: ctypes::size_t,
) -> c_int {
    debug!("sys_ppoll <= nfds: {} timeout: {:?}", nfds, *timeout);
    let to = Duration::from(*timeout).as_millis() as c_int;
    sys_poll(fds, nfds, to)
}

/// Used to monitor multiple file descriptors for events
pub unsafe fn sys_poll(fds: *mut ctypes::pollfd, nfds: ctypes::nfds_t, timeout: c_int) -> c_int {
    debug!("ax_poll <= nfds: {} timeout: {} ms", nfds, timeout);

    syscall_body!(ax_poll, {
        if nfds == 0 {
            return Err(LinuxError::EINVAL);
        }
        let fds = core::slice::from_raw_parts_mut(fds, nfds as usize);
        let deadline = (!timeout.is_negative())
            .then(|| current_time() + Duration::from_millis(timeout as u64));
        loop {
            #[cfg(feature = "net")]
            axnet::poll_interfaces();
            let fds_num = poll_all(fds)?;
            if fds_num > 0 {
                return Ok(fds_num as c_int);
            }

            if deadline.map_or(false, |ddl| current_time() >= ddl) {
                debug!("    timeout!");
                return Ok(0);
            }
            crate::sys_sched_yield();
        }
    })
}
