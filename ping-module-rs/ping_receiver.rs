// SPDX-License-Identifier: GPL-2.0

//! Rust echo server sample.

use kernel::flag_set;
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
        let socket = UdpSocket::new()?;
        // Listen on all interfaces on port 8000.
        socket.bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from([0, 0, 0, 0]),
            8000,
        )))?;
        pr_info!("Listening!");
        let mut buf = [0u8; 1024];
        while let Ok((size, peer)) = socket.receive_from(&mut buf,flag_set!()) {
            if size == 0 {
                break;
            }
            pr_info!("Received {} bytes", size);
            pr_info!("Message: {}", core::str::from_utf8(&buf[..size]).unwrap());
            let sent = socket.send_to(&buf[..size], &peer, flag_set!())?;
            pr_info!("Sent back {} bytes", sent);
        }
        pr_info!("Flush");
        Ok(Self {})
    }
}
