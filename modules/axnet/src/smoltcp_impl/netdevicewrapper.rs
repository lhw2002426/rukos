use core::cell::RefCell;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use log::debug;

use smoltcp::iface::SocketSet;
use smoltcp::phy::{self, ChecksumCapabilities, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use ruxdriver::prelude::*;

use super::{snoop_tcp_packet, DeviceWrapper};

struct Rule {

}

struct RouteTable {
    rules: Vec<Rule>,
}

impl RouteTable {
    fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
}


enum NetDeviceListEntry {
    Loopback(super::loopback::Loopback),
    DeviceWrapper(super::DeviceWrapper),
}

struct NetDeviceList {
    inner: Vec<Box<AxNetDevice>>
}

impl NetDeviceList {
    fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    pub fn add_device(&mut self, dev: AxNetDevice) {
        self.inner.push(Box::new(dev))
    }

    pub fn get(&self, device_name: &str) -> Option<&AxNetDevice> {
        self.inner
            .iter()
            .find(|dev| dev.device_name() == device_name)
            .map(|device| device.as_ref())
    }

    pub fn get_mut(&mut self, device_name: &str) -> Option<&mut AxNetDevice> {
        self.inner
            .iter_mut()
            .find(|dev| dev.device_name() == device_name)
            .map(|device| device.as_mut())
    }

    pub fn iter(&self) -> impl Iterator<Item = &AxNetDevice> {
        self.inner
            .iter()
            .map(|b| b.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AxNetDevice> {
        self.inner
            .iter_mut()
            .map(|b| b.as_mut())
    }
}

/*
impl NetDeviceList {
    fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    pub fn add_device(&mut self, dev: DeviceWrapper) {
        self.inner.push(Box::new(dev))
    }

    pub fn get(&self, device_name: &str) -> Option<&DeviceWrapper> {
        self.inner
            .iter()
            .find(|dev| dev.inner.borrow().device_name() == device_name)
            .map(|device| device.as_ref())
    }

    pub fn get_mut(&mut self, device_name: &str) -> Option<&mut DeviceWrapper> {
        self.inner
            .iter_mut()
            .find(|dev| dev.inner.borrow().device_name() == device_name)
            .map(|device| device.as_mut())
    }

    pub fn iter(&self) -> impl Iterator<Item = &DeviceWrapper> {
        self.inner
            .iter()
            .map(|b| b.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DeviceWrapper> {
        self.inner
            .iter_mut()
            .map(|b| b.as_mut())
    }
}
*/

struct NetDeviceWrapper {
    inner: RefCell<NetDeviceList>,
    route_table: RefCell<RouteTable>,
}

impl NetDeviceWrapper {
    fn new() -> Self {
        Self {
            inner: RefCell::new(NetDeviceList::new()),
            route_table: RefCell::new(RouteTable::new()),
        }
    }
}

struct AxNetRxToken<'a>(&'a RefCell<AxNetDevice>, NetBufPtr);
struct AxNetTxToken<'a>(&'a RefCell<AxNetDevice>);

