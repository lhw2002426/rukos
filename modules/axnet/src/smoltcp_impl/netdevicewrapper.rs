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

pub struct Rule {

}

pub struct RouteTable {
    rules: Vec<Rule>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
}

pub struct NetDeviceList {
    inner: Vec<RefCell<AxNetDevice>>
}

impl NetDeviceList {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    pub fn debug_out(&mut self) {
        info!("net device debug_out {} ", self.inner.len());
    }

    pub fn add_device(&mut self, dev: AxNetDevice) {
        self.inner.push(RefCell::new(dev))
    }

    pub fn get(&self, device_name: &str) -> Option<&RefCell<AxNetDevice>> {
        self.inner
            .iter()
            .find(|dev| dev.borrow().device_name() == device_name)
            .map(|device| device)
    }

    pub fn get_mut(&mut self, device_name: &str) -> Option<&mut RefCell<AxNetDevice>> {
        self.inner
            .iter_mut()
            .find(|dev| dev.borrow().device_name() == device_name)
            .map(|device| device)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RefCell<AxNetDevice>> {
        self.inner
            .iter()
            .map(|b| b)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RefCell<AxNetDevice>> {
        self.inner
            .iter_mut()
            .map(|b| b)
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

