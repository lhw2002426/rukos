/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{sync::Arc, vec, vec::Vec};
use smoltcp::socket::tcp::{SendError, RecvError};
use core::sync::atomic::AtomicIsize;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use axerrno::{LinuxError, LinuxResult, ax_err, ax_err_type, AxError, AxResult};
use axio::{PollState, Result};
use axsync::Mutex;

use lazy_init::LazyInit;

use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{self, AnySocket, tcp::SocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};

use flatten_objects::FlattenObjects;

use ruxtask::yield_now;

const SOCK_ADDR_UN_PATH_LEN: usize = 108;
const UNIX_SOCKET_BUFFER_SIZE: usize = 4096;

struct SockaddrUn {
    sun_family: u16, /* AF_UNIX */
    /// if socket is unnamed, use `sun_path[0]` to save fd
    sun_path: [c_char; SOCK_ADDR_UN_PATH_LEN], /* Pathname */
}

/// unix domain socket.
pub struct UnixSocket<'a> {
    addr: Mutex<SockaddrUn>,
    buf: SocketBuffer<'a>,
    sockethandle: Option<usize>,
    peer_socket: Option<usize>,
    status: UnixSocketStatus,
}

//static UNIX_TABLE: LazyInit<Mutex<SocketSet>> = LazyInit::new();
static UNIX_TABLE: LazyInit<Mutex<FlattenObjects<Arc<UnixSocket>, 1024>>> = LazyInit::new();

#[derive(Debug)]
pub enum UnixSocketType {
    SockStream,
    SockDgram,
    SockSeqpacket,
}

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY -> CLOSED
//       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
pub enum UnixSocketStatus {
    Closed,
    Busy,
    Connecting,
    Connected,
    Listening,
}

impl<'a> UnixSocket<'a> {
    /// create a new socket
    /// only support sock_stream
    pub fn new(_type: UnixSocketType) -> Self {
        info!("lhw debug in unixsocket new {:?}",_type);
        match _type {
            UnixSocketType::SockDgram | UnixSocketType::SockSeqpacket => unimplemented!(),
            UnixSocketType::SockStream => {
                    let mut unixsocket = Self {
                    addr: Mutex::new(SockaddrUn {
                        sun_family: 1, //AF_UNIX
                        sun_path: [0; SOCK_ADDR_UN_PATH_LEN],
                    }),
                    sockethandle: None,
                    buf: SocketBuffer::new(vec![0; 64*1024]),
                    peer_socket: None,
                    status: UnixSocketStatus::Closed,
                };
                let handle = UNIX_TABLE.lock().add(Arc::new(unixsocket)).unwrap();
                unixsocket.set_sockethandle(handle);
                unixsocket
            },
        }
    }

    pub fn set_sockethandle(&mut self, fd: usize) {
        self.sockethandle = Some(fd);
    }

    pub fn get_sockethandle(&self) -> usize {
        self.sockethandle.unwrap()
    }

    pub fn get_state(&self) -> UnixSocketStatus{
        self.status
    }

    pub fn enqueue_buf(&self, data: &[u8]) -> AxResult<usize> {
        match self.get_state() {
            UnixSocketStatus::Closed => Err(AxError::BadState),
            _ => Ok(self.buf.enqueue_slice(data))
        }
    }

    pub fn dequeue_buf(&self, data: &mut [u8]) -> AxResult<usize> {
        match self.get_state() {
            UnixSocketStatus::Closed => Err(AxError::BadState),
            _ => {
                if self.buf.is_empty() {
                    return Err(AxError::WouldBlock);
                }
                Ok(self.buf.dequeue_slice(data))
            }
        }
    }

    // TODO: bind to file system
    pub fn bind(&mut self, addr: SockaddrUn) -> LinuxResult {
        info!("lhw debug in unixsocket bind ");
        match &self.status {
            UnixSocketStatus::Closed => {
                let mut selfaddr = self.addr.lock();
                selfaddr.sun_path = addr.sun_path;
                self.status = UnixSocketStatus::Busy;
                Ok(())
            }
            _ => {
                Err(LinuxError::EINVAL)
            }
        }
        
    }

    pub fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        unimplemented!()
    }
    pub fn recv(&self, buf: &mut [u8], flags: i32) -> LinuxResult<usize> {
        unimplemented!()
    }
    pub fn poll(&self) -> LinuxResult<PollState> {
        unimplemented!()
    }

    pub fn local_addr(&self) -> LinuxResult<SocketAddr> {
        unimplemented!()
    }

    fn fd(&self) -> c_int {
        self.addr.lock().sun_path[0] as _
    }

    pub fn peer_addr(&self) -> LinuxResult<SocketAddr> {
        unimplemented!()
    }
 
    // TODO: check file system
    pub fn connect(&self, addr: SockaddrUn) -> LinuxResult {
        let remote_socket = UNIX_TABLE.lock().iter().find(|socket| socket.addr.lock().sun_path == addr.sun_path).unwrap().1;
        let data = unsafe {
        let bytes = core::mem::transmute::<&usize, &[u8; core::mem::size_of::<usize>()]>(&self.get_sockethandle().into());
            &bytes[..]
        };
        remote_socket.enqueue_buf(data);
        Ok(())
    }
 
    pub fn sendto(&self, buf: &[u8], addr:SockaddrUn) -> LinuxResult<usize> {
        unimplemented!()
    }

    pub fn recvfrom(&self, buf: &mut [u8]) -> LinuxResult<(usize, Option<SocketAddr>)> {
        unimplemented!()
    }

    // TODO: check file system
    pub fn listen(&mut self) -> LinuxResult {
        match &self.status {
            UnixSocketStatus::Busy => {
                self.status = UnixSocketStatus::Listening;
                Ok(())
            }
            _ => {
                Ok(())//ignore simultaneous `listen`s.
            }
        }
    }
 
    pub fn accept(&self) -> AxResult<UnixSocket> {
        match self.get_state() {
            UnixSocketStatus::Listening => {
                // lhw todo impl block
        
                //buf dequeue as handle to get socket
                let data: &mut [u8];
                self.dequeue_buf(data);
                let mut array = [0u8; core::mem::size_of::<usize>()];
                array.copy_from_slice(data);
                let remote_handle = usize::from_ne_bytes(array);
                let unix_socket = UnixSocket::new(UnixSocketType::SockStream);
                let arc_unixsocket = Arc::new(unix_socket);
                let new_sockethandle = UNIX_TABLE.lock().add(arc_unixsocket);
                self.peer_socket = new_sockethandle;
                Ok(unix_socket)
            },
            _ => ax_err!(InvalidInput, "socket accept() failed: not listen")
        }
    }
 
    pub fn shutdown(&self) -> LinuxResult {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) {
        unimplemented!()
    }
}

pub(crate) fn init_unix() {
    UNIX_TABLE.init_by(Mutex::new(FlattenObjects::new()));
}