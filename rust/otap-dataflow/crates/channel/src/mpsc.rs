// SPDX-License-Identifier: Apache-2.0

//! Multiple-producer, single-consumer channel implementation optimized for single-threaded async.

use crate::error::{RecvError, SendError};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

struct ChannelState<T> {
    buffer: VecDeque<T>,
    capacity: usize,
    is_closed: bool,
    senders: usize,
    has_receiver: bool,
    receiver_waker: Option<Waker>,
    sender_wakers: VecDeque<Waker>,
}

/// A single-threaded MPSC channel.
pub struct Channel<T> {
    state: RefCell<ChannelState<T>>,
}

impl<T> Channel<T> {
    /// Creates a new channel with the given capacity.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let channel = Rc::new(Channel {
            state: RefCell::new(ChannelState {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
                is_closed: false,
                senders: 1,
                has_receiver: true,
                receiver_waker: None,
                sender_wakers: VecDeque::new(),
            }),
        });

        (
            Sender {
                channel: channel.clone(),
            },
            Receiver { channel },
        )
    }
}

/// A sender for the channel.
pub struct Sender<T> {
    channel: Rc<Channel<T>>,
}

/// A receiver for the channel.
pub struct Receiver<T> {
    channel: Rc<Channel<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        {
            let mut state = self.channel.state.borrow_mut();
            state.senders += 1;
        }
        Sender {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = self.channel.state.borrow_mut();
        state.senders -= 1;

        if state.senders == 0 {
            state.is_closed = true;
            if let Some(waker) = state.receiver_waker.take() {
                waker.wake();
            }
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut state = self.channel.state.borrow_mut();
        state.has_receiver = false;
        state.is_closed = true;
        // Wake all senders to handle the closure
        for waker in state.sender_wakers.drain(..) {
            waker.wake();
        }
    }
}

impl<T> Sender<T> {
    /// Sends a value to the channel.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut state = self.channel.state.borrow_mut();

        if state.is_closed || !state.has_receiver {
            return Err(SendError::Closed(value));
        }

        if state.buffer.len() >= state.capacity {
            return Err(SendError::Full(value));
        }

        state.buffer.push_back(value);

        if let Some(waker) = state.receiver_waker.take() {
            waker.wake();
        }

        Ok(())
    }

    /// Sends a value to the channel asynchronously.
    pub async fn send_async(&self, value: T) -> Result<(), SendError<T>> {
        SendFuture {
            sender: self.clone(),
            value: Some(value),
        }
        .await
    }

    /// Closes the channel.
    pub fn close(&self) {
        let mut state = self.channel.state.borrow_mut();
        state.is_closed = true;
        // Wake the receiver if it's waiting
        if let Some(waker) = state.receiver_waker.take() {
            waker.wake();
        }
        // Wake all waiting senders
        for waker in state.sender_wakers.drain(..) {
            waker.wake();
        }
    }
}

impl<T> Receiver<T> {
    /// Tries to receive a value from the channel.
    pub fn try_recv(&self) -> Result<T, RecvError> {
        let mut state = self.channel.state.borrow_mut();

        if let Some(value) = state.buffer.pop_front() {
            // Wake one sender if channel was full
            if state.buffer.len() == state.capacity - 1 {
                if let Some(waker) = state.sender_wakers.pop_front() {
                    waker.wake();
                }
            }
            Ok(value)
        } else if state.is_closed {
            Err(RecvError::Closed)
        } else {
            Err(RecvError::Empty)
        }
    }

    /// Receives a value from the channel asynchronously.
    pub async fn recv(&self) -> Result<T, RecvError> {
        RecvFuture { receiver: self }.await
    }
}

struct SendFuture<T> {
    sender: Sender<T>,
    value: Option<T>,
}

impl<T> Unpin for SendFuture<T> {}

impl<T> Future for SendFuture<T> {
    type Output = Result<(), SendError<T>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = self
            .value
            .take()
            .expect("SendFuture polled after completion");

        match self.sender.send(value) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(SendError::Full(value)) => {
                self.value = Some(value);
                let mut state = self.sender.channel.state.borrow_mut();
                state.sender_wakers.push_back(cx.waker().clone());
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

struct RecvFuture<'a, T> {
    receiver: &'a Receiver<T>,
}

impl<T> Future for RecvFuture<'_, T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<T, RecvError>> {
        match self.receiver.try_recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(RecvError::Empty) => {
                let mut state = self.receiver.channel.state.borrow_mut();
                state.receiver_waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tokio::time::{Duration, timeout};

    // Helper function to create a test runtime
    fn create_test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn test_basic_channel_operations() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(2);

            // Test send and receive
            let result = tx.send(1);
            assert!(result.is_ok());

            let result = tx.send(2);
            assert!(result.is_ok());
            assert_eq!(rx.try_recv().unwrap(), 1);
            assert_eq!(rx.try_recv().unwrap(), 2);

            // Test empty channel
            assert!(matches!(rx.try_recv(), Err(RecvError::Empty)));
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_channel_capacity() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, _rx) = Channel::new(1);

            // First send should succeed
            let result = tx.send(1);
            assert!(result.is_ok());

            // Second send should fail with Full error
            match tx.send(2) {
                Err(SendError::Full(2)) => (),
                _ => panic!("Expected Full error"),
            }
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_multiple_producers() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx1, rx) = Channel::new(4);
            let tx2 = tx1.clone();

            // Send from both producers
            let result = tx1.send(1);
            assert!(result.is_ok());
            let result = tx2.send(2);
            assert!(result.is_ok());

            // Verify order
            assert_eq!(rx.try_recv().unwrap(), 1);
            assert_eq!(rx.try_recv().unwrap(), 2);
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_async_send_receive() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(1);
            let received = Rc::new(RefCell::new(vec![]));
            let received_clone = received.clone();

            // Spawn consumer
            let consumer = tokio::task::spawn_local(async move {
                while let Ok(value) = rx.recv().await {
                    received_clone.borrow_mut().push(value);
                    if value == 2 {
                        break;
                    }
                }
            });

            // Send values
            let result = tx.send_async(1).await;
            assert!(result.is_ok());
            let result = tx.send_async(2).await;
            assert!(result.is_ok());

            consumer.await.unwrap();
            assert_eq!(*received.borrow(), vec![1, 2]);
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_channel_closing() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(1);

            // Send a value
            let result = tx.send(1);
            assert!(result.is_ok());

            // Close the channel
            tx.close();

            // Should still be able to receive existing value
            assert_eq!(rx.try_recv().unwrap(), 1);

            // Further receives should indicate closed channel
            assert!(matches!(rx.try_recv(), Err(RecvError::Closed)));

            // Sends should fail with Closed error
            match tx.send(2) {
                Err(SendError::Closed(2)) => (),
                _ => panic!("Expected Closed error"),
            }
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_sender_drop() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(1);

            let result = tx.send(1);
            assert!(result.is_ok());
            drop(tx);

            // Should still receive last value
            assert_eq!(rx.recv().await.unwrap(), 1);

            // Next receive should indicate closed
            assert!(matches!(rx.recv().await, Err(RecvError::Closed)));
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_backpressure() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(1);
            let send_completed = Rc::new(RefCell::new(false));
            let send_completed_clone = send_completed.clone();

            // Spawn producer task that will block
            let producer = tokio::task::spawn_local(async move {
                let result = tx.send(1);
                assert!(result.is_ok());

                let result = tx.send_async(2).await;
                assert!(result.is_ok());
                *send_completed_clone.borrow_mut() = true;
            });

            // Allow some time for the second send to block
            tokio::task::yield_now().await;
            assert!(!*send_completed.borrow());

            // Receive value, allowing blocked send to complete
            assert_eq!(rx.recv().await.unwrap(), 1);

            // Wait for producer to complete
            timeout(Duration::from_millis(100), producer)
                .await
                .expect("producer should complete")
                .unwrap();

            assert!(*send_completed.borrow());
            assert_eq!(rx.recv().await.unwrap(), 2);
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_fairness_in_waking_senders() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(1);
            let received = Rc::new(RefCell::new(vec![]));
            let received_clone = received.clone();

            // Send a value to fill the channel to its capacity
            let result = tx.send_async(1).await;
            assert!(result.is_ok());

            // Spawn senders that wait for the reciver to be process items and wake them up
            let sender_clone1 = tx.clone();
            let sender_clone2 = tx.clone();

            let pending_sender_1 = tokio::task::spawn_local(async move {
                let result = sender_clone1.send_async(2).await;
                assert!(result.is_ok());
            });

            let pending_sender_2 = tokio::task::spawn_local(async move {
                let result = sender_clone2.send_async(3).await;
                assert!(result.is_ok());
            });

            // Spawn consumer
            let consumer = tokio::task::spawn_local(async move {
                let mut count_of_items_processed = 0;
                const MAX_ITEMS_TO_RECEIVE: usize = 3;
                while let Ok(value) = rx.recv().await {
                    received_clone.borrow_mut().push(value);
                    count_of_items_processed += 1;
                    if count_of_items_processed >= MAX_ITEMS_TO_RECEIVE {
                        break;
                    }
                }
            });

            pending_sender_1.await.unwrap();
            pending_sender_2.await.unwrap();
            consumer.await.unwrap();
            assert_eq!(*received.borrow(), vec![1, 2, 3]); // Wake the sender in FIFO order. We should receive 1 -> 2 -> 3 and not 1 -> 3 -> 2
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }
}
