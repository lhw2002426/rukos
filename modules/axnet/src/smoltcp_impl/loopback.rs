/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

 use alloc::collections::VecDeque;
 use alloc::vec::Vec;
 use log::debug;
 
 use smoltcp::iface::SocketSet;
 use smoltcp::phy::{self, ChecksumCapabilities, Device, DeviceCapabilities, Medium};
 use smoltcp::time::Instant;

 use super::snoop_tcp_packet;
 
 /// A loopback device.
 #[derive(Debug)]
 pub struct Loopback {
     pub(crate) queue: VecDeque<Vec<u8>>,
     medium: Medium,
 }
 
 #[allow(clippy::new_without_default)]
 impl Loopback {
     /// Creates a loopback device.
     ///
     /// Every packet transmitted through this device will be received through it
     /// in FIFO order.
     pub fn new(medium: Medium) -> Loopback {
         Loopback {
             queue: VecDeque::new(),
             medium,
         }
     }
 }
 
 impl Device for Loopback {
     type RxToken<'a> = RxToken;
     type TxToken<'a> = TxToken<'a>;
 
     fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 65535;
        caps.checksum = ChecksumCapabilities::ignored();
        caps.medium = self.medium;
        caps
     }
 
     fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
         self.queue.pop_front().map(move |buffer| {
             let rx = RxToken { buffer };
             let tx = TxToken {
                 queue: &mut self.queue,
             };
             (rx, tx)
         })
     }
 
     fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
         let tx = TxToken {
             queue: &mut self.queue,
         };
         
         Some(tx)
     }
 }
 
 #[doc(hidden)]
 pub struct RxToken {
     buffer: Vec<u8>,
 }
 
 impl phy::RxToken for RxToken {
    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_packet(&self.buffer, sockets).ok();
    }

     fn consume<R, F>(mut self, f: F) -> R
     where
         F: FnOnce(&mut [u8]) -> R,
     {
         f(&mut self.buffer)
     }
 }
 
 #[doc(hidden)]
 #[derive(Debug)]
 pub struct TxToken<'a> {
     queue: &'a mut VecDeque<Vec<u8>>,
 }
 
 impl<'a> phy::TxToken for TxToken<'a> {
     fn consume<R, F>(self, len: usize, f: F) -> R
     where
         F: FnOnce(&mut [u8]) -> R,
     {
         let mut buffer = Vec::new();
         buffer.resize(len, 0);
         let result = f(&mut buffer);
         self.queue.push_back(buffer);
         result
     }
 }