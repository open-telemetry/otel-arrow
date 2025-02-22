use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
    thread::JoinHandle,
};

use color_eyre::eyre::Result;
use engine::{Command, CommandHandler};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Just a basic message.
///
/// Note: a message must be at the minimum 'static + Clone + Send.
#[derive(Clone, Debug)]
pub struct Message {
    #[allow(dead_code)]
    pub origin: String,
    pub payload: String,
}

/// Try to bind to a port.
fn try_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<u16> {
    Some(std::net::TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
}

/// Check if the given port is a free TCP port
pub fn is_free_tcp(port: u16) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

    try_bind_tcp(ipv6).is_some() && try_bind_tcp(ipv4).is_some()
}

/// Initializes logs and traces
pub fn init() -> Result<()> {
    color_eyre::install()?;

    let subscriber = FmtSubscriber::builder().with_max_level(Level::ERROR).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(())
}

/// Returns a tuple of available TCP ports.
pub fn available_tcp_ports(first_port_to_scan: u16) -> (u16, u16) {
    // Finds an available port for the receiver TCP listener.
    let mut receiver_tcp_port: u16 = first_port_to_scan;
    while !is_free_tcp(receiver_tcp_port) {
        receiver_tcp_port += 1;
        if receiver_tcp_port > 65534 {
            panic!("Could not find an available port for the receiver TCP listener");
        }
    }

    // Finds an available port for the test TCP server.
    let mut exporter_tcp_port: u16 = receiver_tcp_port + 1;
    while !is_free_tcp(exporter_tcp_port) {
        exporter_tcp_port += 1;
        if exporter_tcp_port > 65534 {
            panic!("Could not find an available port for the test TCP server");
        }
    }

    (receiver_tcp_port, exporter_tcp_port)
}

/// Starts a TCP client that sends 10 messages to the server (one message per
/// connection).
#[allow(dead_code)]
pub fn run_tcp_client(msg_count: usize, tcp_port: u16) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Wait for the TCP server to be ready.
            // Panic if the server is not ready after 5 seconds.
            let started_at = std::time::Instant::now();
            loop {
                if TcpStream::connect(format!("127.0.0.1:{}", tcp_port)).await.is_ok() {
                    break;
                }
                if started_at.elapsed() > std::time::Duration::from_secs(5) {
                    panic!("TCP server not ready after 5 seconds (port: {})", tcp_port);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            // Send 10 messages to the TCP server.
            // The client sends alternately the messages "test/0" and "test/1".
            for i in 0..msg_count {
                let stream = TcpStream::connect(format!("127.0.0.1:{}", tcp_port)).await;
                match stream {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write(format!("test/{}", i % 2).as_bytes()).await {
                            panic!(
                                "Error while sending message to the TCP server: {} (port: {})",
                                e, tcp_port
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Connection failed {:?}", e);
                        return;
                    }
                }
            }
        });
}

/// Starts a TCP client that sends 10 messages to the server (one message per
/// connection).
#[allow(dead_code)]
pub async fn async_run_tcp_client(msg_count: usize, tcp_port: u16) {
    // Wait for the TCP server to be ready.
    // Panic if the server is not ready after 5 seconds.
    let started_at = std::time::Instant::now();
    loop {
        if TcpStream::connect(format!("127.0.0.1:{}", tcp_port)).await.is_ok() {
            break;
        }
        if started_at.elapsed() > std::time::Duration::from_secs(5) {
            panic!("TCP server not ready after 5 seconds (port: {})", tcp_port);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Send 10 messages to the TCP server.
    // The client sends alternately the messages "test/0" and "test/1".
    for i in 0..msg_count {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", tcp_port)).await;
        match stream {
            Ok(mut stream) => {
                if let Err(e) = stream.write(format!("test/{}", i % 2).as_bytes()).await {
                    panic!(
                        "Error while sending message to the TCP server: {} (port: {})",
                        e, tcp_port
                    );
                }
            }
            Err(e) => {
                tracing::error!("Connection failed {:?}", e);
                return;
            }
        }
    }
}

/// Starts a TCP test server that will receive the messages sent by pipeline
/// instances.
pub fn run_tcp_server(msg_count: usize, tcp_port: u16, command_handler: CommandHandler) -> JoinHandle<()> {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let addr = format!("127.0.0.1:{}", tcp_port);

                eprintln!("TCP test server binding on: {}", addr);
                let listener = TcpListener::bind(&addr).await.expect("Could not bind TCP listener");
                tracing::info!("TCP test server listening on: {}", addr);

                let mut msg_received = 0;

                // Accept connections and process messages.
                loop {
                    let (mut socket, _) = listener.accept().await.expect("Could not accept TCP connection");
                    let mut buf = vec![0; 1024];

                    // Reads data from the socket.
                    let n = socket.read(&mut buf).await.expect("failed to read data from socket");

                    match std::str::from_utf8(&buf[0..n]) {
                        Ok(message) => {
                            msg_received += 1;
                            tracing::info!(message=%message, total_msg_received=%msg_received, "TCP server received a new message");
                        }
                        Err(e) => {
                            tracing::error!("Invalid UTF-8 sequence: {}", e);
                        }
                    };

                    if msg_received == msg_count {
                        tracing::info!("TCP test server received all expected messages");
                        assert!(command_handler.send(Command::StopAll).is_ok());
                        break;
                    }
                }
                tracing::info!("TCP test server finished");
            });
    })
}
