// SPDX-License-Identifier: GPL-2.0

//! Rust echo server sample.

use core::fmt::{Debug, Formatter};
use kernel::error::Result;
use kernel::net::addr::*;
use kernel::net::socket::kasync::AsyncSocket;
use kernel::net::socket::opts::SocketOptions;
use kernel::net::socket::{opts, ShutdownCmd, SockType, Socket};
use kernel::net::tcp::TcpListener;
use kernel::net::udp::UdpSocket;
use kernel::prelude::*;
use kernel::{bindings, net::*};

module! {
    type: RustEchoServer,
    name: "rust_echo_server",
    author: "Rust for Linux Contributors",
    license: "GPL",
}

struct RustEchoServer {}

fn udp_sock() -> Result<RustEchoServer> {
    let mut address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOOPBACK, 8000));
    let socket = UdpSocket::new()?;
    socket.bind(address)?;
    pr_info!("Listening!");
    let mut buf = [0u8; 1024];
    while let Ok((len, peer)) = socket.receive(&mut buf, true) {
        if len == 0 {
            break;
        }
        pr_info!("Received {} bytes", len);
        pr_info!("Message: {}", core::str::from_utf8(&buf[..len]).unwrap());
        socket.send(&buf[..len], &peer)?;
    }
    pr_info!("Flush");
    Ok(RustEchoServer {})
}

fn tcp_sock() -> Result<RustEchoServer> {
    let listener = TcpListener::new(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOOPBACK, 8000)))?;
    pr_info!("Listening!");
    while let Ok(stream) = listener.accept() {
        pr_info!("Accepted!");
        let mut buf = [0u8; 1024];
        while let Ok(len) = stream.receive(&mut buf) {
            if len == 0 {
                break;
            }
            pr_info!("Received {} bytes", len);
            pr_info!("Message: {}", core::str::from_utf8(&buf[..len]).unwrap());
            stream.send(&buf[..len])?;
        }
        pr_info!("Flush");
    }
    Ok(RustEchoServer {})
}

impl kernel::Module for RustEchoServer {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        tcp_sock();
        udp_sock();
        Ok(Self {})
    }
}
