// SPDX-License-Identifier: GPL-2.0

//! Rust echo server sample.

use kernel::error::Result;
use kernel::net::addr::*;
use kernel::net::socket::{opts, SockType, Socket};
use kernel::net::tcp::TcpListener;
use kernel::net::udp::UdpSocket;
use kernel::net::*;
use kernel::prelude::*;

module! {
    type: RustEchoServer,
    name: "rust_echo_server",
    author: "Rust for Linux Contributors",
    license: "GPL",
}

struct RustEchoServer {}

impl kernel::Module for RustEchoServer {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        let socket = Socket::new(AddressFamily::Inet, SockType::Datagram, IpProtocol::Udp)?;
        // node01: 192.168.56.101.
        // This sender is meant to be run on node02.
        let peer_addr =
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from([192, 168, 56, 101]), 8000));
        socket.connect(&peer_addr, 0)?;
        pr_info!("Connected!");
        let mut buf = [0u8; 1024];
        let msg = "Hello, world!";
        while let Ok(size) = socket.send(msg.as_bytes(), []) {
            if size == 0 {
                break;
            }
            pr_info!("Sent {} bytes", size);
            let size = socket.receive(&mut buf, [])?;
            pr_info!("Received {} bytes", size);
            pr_info!("Message: {}", core::str::from_utf8(&buf[..size]).unwrap());
        }
        Ok(Self {})
    }
}
