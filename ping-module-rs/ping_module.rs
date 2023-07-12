// SPDX-License-Identifier: GPL-2.0

//! Rust echo server sample.

use kernel::{
    kasync::executor::{workqueue::Executor as WqExecutor, Executor},
    net::{self, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    prelude::*,
    spawn_task,
    sync::ArcBorrow,
};

module! {
    type: RustEchoServer,
    name: "rust_echo_server",
    author: "Rust for Linux Contributors",
    description: "Rust tcp echo sample",
    license: "GPL",
    params: {
        ADDRESS: ArrayParam<u8, 4> {
            default: [127, 0, 0, 1],
            permissions: 0,
            description: "Example of array",
        },
        PORT: u16 {
            default: 8080,
            permissions: 0,
            description: "The port to bind the TCP server to",
        },
    },
}

async fn echo_server(sock: UdpSocket) -> Result {
    let mut buf = [0u8; 1024];
    loop {
        pr_info!("1");
        let (n, addr) = sock.read(&mut buf, true)?;
        match addr {
            SocketAddr::V4(addrv4) => {
                pr_info!("Packet from port {} {}", addrv4.address().as_str(), addrv4.port());
            }
            _ => {}
        }
        pr_info!("Received {} bytes", n);
        if n == 0 {
            pr_info!("Breaking!");
            return Ok(());
        }
        pr_info!("2");
        sock.write(&mut buf, true)?;
        pr_info!("3");
    }
}

//
// async fn accept_loop(listener: TcpListener, executor: Arc<impl Executor>) {
//     loop {
//         if let Ok(stream) = listener.accept().await {
//             let _ = spawn_task!(executor.as_arc_borrow(), echo_server(stream));
//         }
//     }
// }
//
fn start_listener(
    ex: ArcBorrow<'_, impl Executor + Send + Sync + 'static>,
    ip_parts: &[u8; 4],
    port: &u16,
) -> Result {
    let ip_addr: Ipv4Addr = Ipv4Addr::from(ip_parts);
    let addr = SocketAddr::V4(SocketAddrV4::new(ip_addr, *port));

    let udp_sock = UdpSocket::new(net::init_ns(), &addr)?;
    pr_info!("Created UDP socket!");
    spawn_task!(ex, echo_server(udp_sock)).unwrap();

    //let listener = TcpListener::try_new(net::init_ns(), &addr)?;
    //spawn_task!(ex, accept_loop(listener, ex.into()))?;
    Ok(())
}

struct RustEchoServer {
    //_handle: AutoStopHandle<dyn Executor>,
}

impl kernel::Module for RustEchoServer {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        let handle = WqExecutor::try_new(kernel::workqueue::system())?;
        let server_addr: &[u8; 4] = ADDRESS.read().try_into().unwrap();
        let server_port: &u16 = PORT.read();
        pr_info!("Starting server...");

        //pr_crit!("Starting server on {}:{}", server_addr, server_port);

        start_listener(handle.executor(), server_addr, server_port)?;
        Ok(Self {
            //_handle: handle.into(),
        })
    }
}
