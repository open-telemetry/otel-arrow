// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_stream::{Stream, wrappers::TcpListenerStream};
use std::sync::{Arc, Mutex};

/// A wrapper around TcpListenerStream that can be shut down
pub struct ShutdownableTcpListenerStream {
    stream: TcpListenerStream,
    shutdown_rx: oneshot::Receiver<()>,
    shutdown_triggered: Arc<Mutex<bool>>,
}

impl Stream for ShutdownableTcpListenerStream {
    type Item = std::io::Result<TcpStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If we've already triggered shutdown, return None to end the stream
        if *self.shutdown_triggered.lock().unwrap() {
            return Poll::Ready(None);
        }

        // Poll the shutdown channel
        let shutdown_poll = Pin::new(&mut self.shutdown_rx).poll(cx);
        
        // If shutdown has been signaled, mark as shutdown and end the stream
        if let Poll::Ready(_) = shutdown_poll {
            eprintln!("Shutdown signal received, closing TCP listener stream");
            *self.shutdown_triggered.lock().unwrap() = true;
            return Poll::Ready(None);
        }
        
        // Otherwise, poll the underlying stream
        Pin::new(&mut self.stream).poll_next(cx)
    }
}

pub fn create_shutdownable_tcp_listener(
    listener: TcpListener
) -> (ShutdownableTcpListenerStream, oneshot::Sender<()>) {
    // Create a shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    
    // Create the standard TcpListenerStream
    let stream = TcpListenerStream::new(listener);
    
    let shutdownable_stream = ShutdownableTcpListenerStream {
        stream,
        shutdown_rx,
        shutdown_triggered: Arc::new(Mutex::new(false)),
    };
    
    (shutdownable_stream, shutdown_tx)
}
