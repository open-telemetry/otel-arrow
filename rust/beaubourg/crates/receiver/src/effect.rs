//! EffectHandler for the receivers.

use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::Error;

/// Handler used by a receiver to act on the pipeline to which it is connected.
///
/// Note: The struct EffectHandler is used to make opaque the inner enum as the
/// different variants should not be exposed publicly. This could be removed if
/// Rust ever supports opaque enums or some form of private variants.
#[derive(Clone)]
pub struct EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    effect_handler: PrivateEffectHandler<Msg>,
}

impl<Msg> EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Creates a new EffectHandler with a channel implementation.
    pub fn with_channel(receiver_name: String, sender: flume::Sender<Vec<Msg>>, reusable_tcp_listener: bool) -> Self {
        EffectHandler {
            effect_handler: PrivateEffectHandler::Channel {
                receiver_name,
                sender,
                reusable_tcp_listener,
            },
        }
    }

    /// Send a collection of messages to the chain of processors connected to
    /// this receiver.
    pub async fn send_messages(&self, messages: Vec<Msg>) -> Result<(), Error> {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                receiver_name, sender, ..
            } => Ok(sender.send_async(messages).await.map_err(|e| Error::Receiver {
                receiver: receiver_name.clone(),
                error: e.to_string(),
                context: Default::default(),
            })?),
        }
    }

    /// Returns a std TCP listener that can be used to accept async connections
    /// from the specified address. Depending on the engine used, this may
    /// be a:
    /// * normal TCP listener for the multi-threaded engine.
    /// * or a SO_REUSEADDR + SO_REUSEPORT listener for the thread-per-core
    ///   engine.
    pub async fn std_tcp_listener(&self, addr: SocketAddr) -> Result<std::net::TcpListener, Error> {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                receiver_name,
                reusable_tcp_listener,
                ..
            } => {
                if *reusable_tcp_listener {
                    // Create a SO_REUSEADDR + SO_REUSEPORT listener.
                    let sock = socket2::Socket::new(
                        match addr {
                            SocketAddr::V4(_) => socket2::Domain::IPV4,
                            SocketAddr::V6(_) => socket2::Domain::IPV6,
                        },
                        socket2::Type::STREAM,
                        None,
                    )
                    .map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;

                    // Configure TCP socket with SO_REUSEADDR and SO_REUSEPORT
                    sock.set_reuse_address(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.set_reuse_port(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.set_nonblocking(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.bind(&addr.into()).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.listen(8192).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;

                    let tcp_listener: std::net::TcpListener = sock.into();
                    Ok(tcp_listener)
                } else {
                    Ok(std::net::TcpListener::bind(&addr).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?)
                }
            }
        }
    }

    /// Returns a Tokio TCP listener that can be used to accept async
    /// connections from the specified address. Depending on the engine
    /// used, this may be a:
    /// * normal TCP listener for the multi-threaded engine.
    /// * or a SO_REUSEADDR + SO_REUSEPORT listener for the thread-per-core
    ///   engine.
    pub async fn tokio_tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error> {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                receiver_name,
                reusable_tcp_listener,
                ..
            } => {
                if *reusable_tcp_listener {
                    // Create a SO_REUSEADDR + SO_REUSEPORT listener.
                    let sock = socket2::Socket::new(
                        match addr {
                            SocketAddr::V4(_) => socket2::Domain::IPV4,
                            SocketAddr::V6(_) => socket2::Domain::IPV6,
                        },
                        socket2::Type::STREAM,
                        None,
                    )
                    .map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;

                    // Configure TCP socket with SO_REUSEADDR and SO_REUSEPORT
                    sock.set_reuse_address(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.set_reuse_port(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.set_nonblocking(true).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.bind(&addr.into()).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;
                    sock.listen(8192).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?;

                    Ok(TcpListener::from_std(sock.into()).map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?)
                } else {
                    Ok(TcpListener::bind(&addr).await.map_err(|e| Error::TcpListener {
                        receiver: receiver_name.clone(),
                        error: e.to_string(),
                    })?)
                }
            }
        }
    }
}

/// Handler used by a receiver to act on the pipeline to which it is connected.
#[derive(Clone)]
pub enum PrivateEffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// An effect handler based on a tokio mpsc channel sender.
    Channel {
        /// The receiver name.
        receiver_name: String,
        /// The sender to the pipeline.
        sender: flume::Sender<Vec<Msg>>,
        /// Flag indicating whether the TCP listener should be reusable or not
        /// (SO_REUSE_PORT, SO_REUSE_ADDR).
        reusable_tcp_listener: bool,
    },
}
