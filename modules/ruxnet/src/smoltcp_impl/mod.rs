/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

mod addr;
mod bench;
mod dns;
mod listen_table;
mod tcp;
mod udp;
mod netdevicewrapper;

use alloc::string::String;
use alloc::vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut};
use alloc::sync::Arc;
use core::default;
use core::ops::DerefMut;

use axsync::Mutex;
use driver_net::{DevError, NetBufPtr};
use lazy_init::LazyInit;
use ruxdriver::prelude::*;
use ruxhal::time::{current_time_nanos, NANOS_PER_MICROS};
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{self, AnySocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};

use self::listen_table::ListenTable;

pub use self::dns::dns_query;
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;
//pub use self::loopback::Loopback;
pub use driver_net::loopback::LoopbackDevice;
pub use self::netdevicewrapper::{RouteTable, NetDeviceList};

macro_rules! env_or_default {
    ($key:literal) => {
        match option_env!($key) {
            Some(val) => val,
            None => "",
        }
    };
}

const IP: &str = env_or_default!("RUX_IP");
const GATEWAY: &str = env_or_default!("RUX_GW");
const DNS_SEVER: &str = "8.8.8.8";
const IP_PREFIX: u8 = 24;

const STANDARD_MTU: usize = 1500;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

const TCP_RX_BUF_LEN: usize = 64 * 1024;
const TCP_TX_BUF_LEN: usize = 64 * 1024;
const UDP_RX_BUF_LEN: usize = 64 * 1024;
const UDP_TX_BUF_LEN: usize = 64 * 1024;
const LISTEN_QUEUE_SIZE: usize = 512;

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();
//static ETH0: LazyInit<InterfaceWrapper<DeviceWrapper>> = LazyInit::new();
//static LO: LazyInit<InterfaceWrapper<DeviceWrapper>> = LazyInit::new();//loopback net device
static RUX_IFACE: LazyInit<InterfaceWrapper<DeviceWrapper>> = LazyInit::new();
static default_dev: LazyInit<String> = LazyInit::new();

struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper {
    //inner: RefCell<AxNetDevice>, // use `RefCell` is enough since it's wrapped in `Mutex` in `InterfaceWrapper`.
    inner: Mutex<NetDeviceList>,
    route_table: RefCell<RouteTable>,
}

struct InterfaceWrapper<D: Device> {
    name: &'static str,
    ether_addr: EthernetAddress,
    dev: Mutex<D>,
    iface: Mutex<Interface>,
}

impl<'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        Self(Mutex::new(SocketSet::new(vec![])))
    }

    pub fn new_tcp_socket() -> socket::tcp::Socket<'a> {
        let tcp_rx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        socket::tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    }

    pub fn new_udp_socket() -> socket::udp::Socket<'a> {
        let udp_rx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_RX_BUF_LEN],
        );
        let udp_tx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_TX_BUF_LEN],
        );
        socket::udp::Socket::new(udp_rx_buffer, udp_tx_buffer)
    }

    pub fn new_dns_socket() -> socket::dns::Socket<'a> {
        let server_addr = DNS_SEVER.parse().expect("invalid DNS server address");
        socket::dns::Socket::new(&[server_addr], vec![])
    }

    pub fn add<T: AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        debug!("socket {}: created", handle);
        handle
    }

    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }

    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }

    pub fn poll_interfaces(&self) {
        RUX_IFACE.poll(&self.0)
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}

impl InterfaceWrapper<DeviceWrapper> {
    fn new(name: &'static str, mut dev: DeviceWrapper, ether_addr: EthernetAddress) -> Self {
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
        config.random_seed = RANDOM_SEED;

        //let mut dev = DeviceWrapper::new(dev);
        let iface = Mutex::new(Interface::new(config, &mut dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }
}

/*impl InterfaceWrapper<Loopback> {
    fn new_loopback(name: &'static str, mut dev: Loopback, ether_addr: EthernetAddress) -> Self {
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
        config.random_seed = RANDOM_SEED;

        let iface = Mutex::new(Interface::new(config, &mut dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }
}*/

impl<D: Device> InterfaceWrapper<D> {
    /*fn new(name: &'static str, dev: AxNetDevice, ether_addr: EthernetAddress) -> Self {
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
        config.random_seed = RANDOM_SEED;

        let mut dev = DeviceWrapper::new(dev);
        let iface = Mutex::new(Interface::new(config, &mut dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }*/

    fn current_time() -> Instant {
        Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64)
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ether_addr
    }

    pub fn setup_ip_addr(&self, ip: IpAddress, prefix_len: u8) {
        let mut iface = self.iface.lock();
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs.push(IpCidr::new(ip, prefix_len)).unwrap();
            info!("ipaddr push {} len: {}",ip, ip_addrs.len());
        });
    }

    pub fn setup_gateway(&self, gateway: IpAddress) {
        let mut iface = self.iface.lock();
        match gateway {
            IpAddress::Ipv4(v4) => iface.routes_mut().add_default_ipv4_route(v4).unwrap(),
        };
    }

    pub fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        let timestamp = Self::current_time();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl DeviceWrapper {
    fn new() -> Self {
        Self {
            inner: Mutex::new(NetDeviceList::new()),
            route_table: RefCell::new(RouteTable::new()),
        }
    }
}

//lhw TODO use route based on rule
pub fn get_route_dev(buf: &[u8]) -> &str {
    use smoltcp::wire::{EthernetFrame, EthernetProtocol, Ipv4Packet, ArpPacket, UdpPacket};

    match EthernetFrame::new_checked(buf) {
        Ok(ether_frame) => {
            match Ipv4Packet::new_checked(ether_frame.payload()) {
                Ok(ipv4_packet) => {
                    let dst_addr = ipv4_packet.dst_addr();
                    let src_addr = ipv4_packet.src_addr();
                    if dst_addr.is_loopback() || src_addr.is_loopback(){
                        return "loopback";
                        //return "virtio-net";
                    }
                }
                _ => {
                    match ArpPacket::new_checked(ether_frame.payload()) {
                        Ok(arp_packet) => {
                            let dst_addr = arp_packet.target_protocol_addr();
                            let src_addr = arp_packet.source_protocol_addr();
                            if arp_packet.protocol_type() ==  EthernetProtocol::Ipv4 && (dst_addr[0] == 127 || src_addr[0] == 127) {
                                return "loopback";
                                //return "virtio-net";
                            }
                        }
                        _ => {}
                    }
                }
            }
        },
        _ => {},
    }
    
    "virtio-net"
}

static mut debug_id:u8 = 0;

impl Device for DeviceWrapper {
    type RxToken<'a> = AxNetRxToken<'a> where Self: 'a;
    type TxToken<'a> = AxNetTxToken<'a> where Self: 'a;

    fn receive<'a>(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut rx_buf_option: Option<NetBufPtr> = None;
        let mut dev_name_option: Option<String> = None;
        //info!("lhw debug device wrapper lock in receive");
        for dev in self.inner.lock().iter() {
            //let mut dev = self.inner.borrow_mut();
            if let Err(e) = dev.borrow_mut().recycle_tx_buffers() {
                warn!("recycle_tx_buffers failed: {:?}", e);
                //return None;
                continue;
            }
            let dev_name:String = dev.borrow().device_name().into();

            if !dev.borrow().can_transmit() {
                //return None;
                continue;
            }
            let rx_buf = match dev.borrow_mut().receive() {
                Ok(buf) => {
                    debug!("lhw debug in {} receive {:X?}",dev_name,buf.packet());
                    buf
                },
                Err(err) => {
                    if !matches!(err, DevError::Again) {
                        warn!("receive failed: {:?}", err);
                    }
                    //return None;
                    continue;
                }
            };
            info!("lhw debug device wrapper receive packet {:X?} in dev {}", rx_buf.packet(), dev.borrow().device_name());
            dev_name_option = Some(String::from(dev.borrow().device_name()));
            rx_buf_option = Some(rx_buf);
            break;
        }
        if let Some(dev_name) = dev_name_option {
            let tx_ret = AxNetTxToken(Rc::new(&self.inner), unsafe {debug_id += 1;debug_id});
            {
                info!("lhw debug before borrow mut receive {}", unsafe {debug_id});
                let ax_net_device = (*(tx_ret.0.clone())).lock();
            }
            return Some((AxNetRxToken(&self.inner, rx_buf_option.unwrap(), dev_name), tx_ret))   
        };
        None
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        //lhw TODO transmit only if all dev ready, it's unreasonable
        //let mut dev = self.inner.borrow_mut();
        info!("lhw debug device wrapper lock in transmit");
        for dev in self.inner.lock().iter() {
            if let Err(e) = dev.borrow_mut().recycle_tx_buffers() {
                warn!("recycle_tx_buffers failed: {:?}", e);
                return None;
            }
            if !dev.borrow().can_transmit() {
                debug!("{} can not transmit", dev.borrow().device_name());
                return None;
            }
        }
        let tx_ret = AxNetTxToken(Rc::new(&self.inner), unsafe {debug_id += 1;debug_id});
        {
            info!("lhw debug before borrow mut transmit {}", unsafe {debug_id});
            let ax_net_device = (*(tx_ret.0.clone())).lock();
        }
        Some(tx_ret)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct AxNetRxToken<'a>(&'a Mutex<NetDeviceList>, NetBufPtr, String);
struct AxNetTxToken<'a>(Rc<&'a Mutex<NetDeviceList>>, u8);

impl<'a> AxNetRxToken<'a> {
    
}

impl<'a> RxToken for AxNetRxToken<'a> {
    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_packet(self.1.packet(), sockets).ok();
    }

    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        info!("lhw debug device wrapper lock in rx consume");
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        //info!("lhw debug in rx consume dev {}, packet {:X?}", self.2, rx_buf.packet());
        let result = f(rx_buf.packet_mut());
        let mut ax_net_device = self.0.lock();
        let dev = ax_net_device.borrow_mut().get(self.2.as_str()).unwrap();
        dev.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl<'a> TxToken for AxNetTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        {
            info!("lhw debug before borrow mut should begin here {}", self.1);
            (*(self.0.clone())).lock().debug_out();
        }
        let res = {
            let mut buffer = vec![0u8; len];
            let tx_buf_temp: &mut [u8] = &mut buffer[..];
            let ret = f(tx_buf_temp);
            let dev_name = get_route_dev(tx_buf_temp);
            //debug!("lhw debug in tx consume temp {:X?}", tx_buf_temp);
            let dev_lock = *self.0;
            info!("lhw debug before borrow mut {}", self.1);
            let ax_net_device = dev_lock.lock();
            let dev = ax_net_device.get(&dev_name).unwrap();
            let mut dev_borrowed = dev.borrow_mut();
            let mut tx_buf = dev_borrowed.alloc_tx_buffer(len).unwrap();
            tx_buf.packet_mut().copy_from_slice(&tx_buf_temp);
            debug!("lhw debug in {} SEND {} bytes: {:02X?}",dev_name, len, tx_buf.packet());
            //info!("lhw debug in tx consmue use dev {} trans {:X?}",dev_name, tx_buf.packet());
            dev_borrowed.transmit(tx_buf).unwrap();
            ret
        };
        let dev_lock = *self.0;
        info!("lhw debug before borrow mut should end here {}", self.1);
        let ax_net_device = dev_lock.lock();
        res
    }
}

fn snoop_tcp_packet(buf: &[u8], sockets: &mut SocketSet<'_>) -> Result<(), smoltcp::wire::Error> {
    info!("snoop tcp packet {:X?}", buf);
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ether_frame = EthernetFrame::new_checked(buf)?;
    let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = (ipv4_packet.src_addr(), tcp_packet.src_port()).into();
        let dst_addr = (ipv4_packet.dst_addr(), tcp_packet.dst_port()).into();
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            info!("lhw debug deal incoming tcp packet");
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
        else {
            info!("lhw debug not first");
        }
        //lhw TODO use a more common way , like table
    }
    Ok(())
}

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() {
    SOCKET_SET.poll_interfaces();
}

/// Benchmark raw socket transmit bandwidth.
pub fn bench_transmit() {
    RUX_IFACE.dev.lock().bench_transmit_bandwidth();
}

/// Benchmark raw socket receive bandwidth.
pub fn bench_receive() {
    RUX_IFACE.dev.lock().bench_receive_bandwidth();
}

pub(crate) fn init_netdev(net_dev: AxNetDevice) {
    let ether_addr = EthernetAddress(net_dev.mac_address().0);
    if !RUX_IFACE.is_init() {
        info!("lhw debug in rux_iface init ether_addr {}",ether_addr);
        let dev_wrapper = DeviceWrapper::new();
        let rux_iface = InterfaceWrapper::new("rux_iface", dev_wrapper, ether_addr);

        let ip = IP.parse().expect("invalid IP address");
        let gateway = GATEWAY.parse().expect("invalid gateway IP address");
        //eth0.setup_ip_addr(ip, IP_PREFIX);
        //eth0.setup_gateway(gateway);
        rux_iface.setup_ip_addr(ip, IP_PREFIX);
        rux_iface.setup_gateway(gateway);

        let local_ip = "127.0.0.1".parse().expect("invalid IP address");
        //lo.setup_ip_addr(local_ip, 8);
        rux_iface.setup_ip_addr(local_ip, 8);


        //ETH0.init_by(eth0);
        //LO.init_by(lo);
        RUX_IFACE.init_by(rux_iface);
        SOCKET_SET.init_by(SocketSetWrapper::new());
        LISTEN_TABLE.init_by(ListenTable::new());
        info!("created net interface {:?}:", RUX_IFACE.name());
        info!("  ether:    {}", RUX_IFACE.ethernet_address());
        info!("  ip:       {}/{}", ip, IP_PREFIX);
        info!("  gateway:  {}", gateway);
    }
    RUX_IFACE.dev.lock().inner.lock().add_device(net_dev);
 }