// SPDX-License-Identifier: Apache-2.0

//! Multiple-producer, multiple-consumer channel implementation optimized for single-threaded async
//! runtime.

use crate::error::{RecvError, SendError};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

struct ChannelState<T> {
    buffer: VecDeque<T>,
    capacity: NonZeroUsize,
    is_closed: bool,
    senders: usize,
    receiver_wakers: VecDeque<Waker>,
    sender_wakers: VecDeque<Waker>,
}

struct Channel<T> {
    state: RefCell<ChannelState<T>>,
}

impl<T> Channel<T> {
    #[allow(clippy::new_ret_no_self)]
    #[allow(dead_code)]
    fn new(capacity: NonZeroUsize) -> (Sender<T>, Receiver<T>) {
        let channel = Rc::new(Channel {
            state: RefCell::new(ChannelState {
                buffer: VecDeque::with_capacity(capacity.get()),
                capacity,
                is_closed: false,
                senders: 1,
                receiver_wakers: VecDeque::new(),
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

#[derive(Clone)]
struct Sender<T> {
    channel: Rc<Channel<T>>,
}

#[derive(Clone)]
struct Receiver<T> {
    channel: Rc<Channel<T>>,
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = self.channel.state.borrow_mut();
        state.senders -= 1;

        // If this was the last sender, wake all receivers
        if state.senders == 0 {
            state.is_closed = true;
            // Drain all wakers in FIFO order
            while let Some(waker) = state.receiver_wakers.pop_front() {
                waker.wake();
            }
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut state = self.channel.state.borrow_mut();

        // Check if this is the last receiver by comparing the Rc count
        // against the number of senders plus one (for this receiver)
        if Rc::strong_count(&self.channel) == state.senders + 1 {
            state.is_closed = true;
            // Wake all blocked senders in FIFO order
            while let Some(waker) = state.sender_wakers.pop_front() {
                waker.wake();
            }
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut state = self.channel.state.borrow_mut();

        if state.is_closed {
            return Err(SendError::Closed(value));
        }

        if state.buffer.len() >= state.capacity.get() {
            return Err(SendError::Full(value));
        }

        state.buffer.push_back(value);

        // Wake the receiver that has been waiting the longest
        if let Some(waker) = state.receiver_wakers.pop_front() {
            waker.wake();
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send_async(&self, value: T) -> Result<(), SendError<T>> {
        SendFuture {
            sender: self.clone(),
            value: Some(value),
        }
        .await
    }

    #[allow(dead_code)]
    pub fn close(&self) {
        let mut state = self.channel.state.borrow_mut();
        state.is_closed = true;
        // Wake all receivers in FIFO order
        while let Some(waker) = state.receiver_wakers.pop_front() {
            waker.wake();
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> Self {
        let mut state = self.channel.state.borrow_mut();
        state.senders += 1;
        Sender {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, RecvError> {
        let mut state = self.channel.state.borrow_mut();

        if let Some(value) = state.buffer.pop_front() {
            // Wake one sender if channel was full
            if state.buffer.len() == state.capacity.get() - 1 {
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

    #[allow(dead_code)]
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
                // Store both the order and the waker
                state.receiver_wakers.push_back(cx.waker().clone());
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
            let (tx, rx) = Channel::new(NonZeroUsize::new(2).unwrap());

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
            let (tx, _rx) = Channel::new(NonZeroUsize::new(1).unwrap());

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
            let (tx1, rx) = Channel::new(NonZeroUsize::new(4).unwrap());
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
    fn test_multiple_receivers() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let (tx, rx) = Channel::new(NonZeroUsize::new(2).unwrap());

        // Shared state to track all received values
        let all_received = Rc::new(RefCell::new(Vec::new()));

        let mut handles = vec![];

        for i in 1..=3 {
            let received = all_received.clone();
            let rx = rx.clone();
            let handle = local.spawn_local(async move {
                while let Ok(value) = rx.recv().await {
                    println!("Receiver {i}: Received value {value}");
                    received.borrow_mut().push(value);
                }
            });
            handles.push(handle);
        }

        let msg_to_send_count = 10;

        let handle = local.spawn_local(async move {
            // Send several values
            for i in 1..=msg_to_send_count {
                let result = tx.send_async(i).await;
                assert!(result.is_ok());
            }

            // Close the channel to let receivers finish
            tx.close();
        });
        handles.push(handle);

        rt.block_on(local);
        for handle in handles {
            rt.block_on(handle).expect("Test task failed");
        }

        // Verify that all values were received exactly once
        let all_values = all_received.borrow();
        assert_eq!(
            all_values.len(),
            msg_to_send_count,
            "Should receive exactly msg_to_send_count values in total"
        );

        // Check that each value from 1 to 6 appears exactly once
        let mut sorted_values = all_values.clone();
        sorted_values.sort_unstable();
        let expected_values = (1..=msg_to_send_count).collect::<Vec<_>>();
        assert_eq!(
            sorted_values, expected_values,
            "Each value should appear exactly once"
        );
    }

    #[test]
    fn test_consumer_fairness() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());
            let receive_order = Rc::new(RefCell::new(Vec::new()));

            // Spawn multiple consumers that will be blocked
            let mut consumer_tasks = vec![];
            for i in 1..=3 {
                let rx = rx.clone();
                let receive_order = receive_order.clone();
                let consumer = tokio::task::spawn_local(async move {
                    let _val = rx.recv().await.unwrap();
                    receive_order.borrow_mut().push(i);
                });
                consumer_tasks.push(consumer);
            }

            // Let consumers get blocked
            tokio::task::yield_now().await;

            // Send values one by one, allowing blocked receives to complete in order
            for i in 1..=3 {
                let result = tx.send(i);
                assert!(result.is_ok());
                tokio::task::yield_now().await;
            }

            // Wait for all consumers to complete
            for task in consumer_tasks {
                task.await.unwrap();
            }

            // Verify receives completed in FIFO order
            let final_order = receive_order.borrow().clone();
            assert_eq!(
                final_order,
                vec![1, 2, 3],
                "Consumers were not unblocked in FIFO order"
            );
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_producer_fairness() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());
            let send_order = Rc::new(RefCell::new(Vec::new()));

            // Fill the channel
            let result = tx.send(0);
            assert!(result.is_ok());

            // Spawn multiple producers that will be blocked
            let mut producer_tasks = vec![];
            for i in 1..=3 {
                let tx = tx.clone();
                let send_order = send_order.clone();
                let producer = tokio::task::spawn_local(async move {
                    let result = tx.send_async(i).await;
                    assert!(result.is_ok());
                    send_order.borrow_mut().push(i);
                });
                producer_tasks.push(producer);
            }

            // Let producers get blocked
            tokio::task::yield_now().await;

            // Receive values one by one, allowing blocked sends to complete in order
            assert_eq!(rx.recv().await.unwrap(), 0);

            for _ in 0..3 {
                let val = rx.recv().await.unwrap();
                assert!((1..=3).contains(&val), "Received unexpected value");
            }

            // Wait for all producers to complete
            for task in producer_tasks {
                task.await.unwrap();
            }

            // Verify sends completed in FIFO order
            let final_order = send_order.borrow().clone();
            for window in final_order.windows(2) {
                assert!(
                    window[0] < window[1],
                    "Producers were not unblocked in FIFO order: {final_order:?}"
                );
            }
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_mixed_operations() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(2).unwrap());

            // Mix of sync and async sends
            let result = tx.send(1);
            assert!(result.is_ok());
            let result = tx.send_async(2).await;
            assert!(result.is_ok());

            // Mix of sync and async receives
            assert_eq!(rx.try_recv().unwrap(), 1);
            assert_eq!(rx.recv().await.unwrap(), 2);
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_receiver_drop() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(2).unwrap());
            let result = tx.send(1);
            assert!(result.is_ok());

            // Clone receiver before dropping original
            let rx2 = rx.clone();
            drop(rx);

            // New sends should still work with remaining receiver
            let result = tx.send(2);
            assert!(result.is_ok());
            assert_eq!(rx2.recv().await.unwrap(), 1);
            assert_eq!(rx2.recv().await.unwrap(), 2);
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_complex_receiver_drop() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(2).unwrap());

            // Create multiple receivers
            let rx2 = rx.clone();
            let rx3 = rx.clone();

            let result = tx.send(1);
            assert!(result.is_ok());
            let result = tx.send(2);
            assert!(result.is_ok());

            // Drop receivers in different order while messages are in flight
            drop(rx);

            // Verify rx2 can still receive
            assert_eq!(rx2.recv().await.unwrap(), 1);

            // Drop another receiver
            drop(rx2);

            // Last receiver should still work
            assert_eq!(rx3.recv().await.unwrap(), 2);

            // Send more data to the last receiver
            let result = tx.send(3);
            assert!(result.is_ok());
            assert_eq!(rx3.recv().await.unwrap(), 3);

            // Drop last receiver
            drop(rx3);

            // Sending should now fail with SendError::Closed
            assert!(matches!(tx.send(4), Err(SendError::Closed(4))));
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_async_send_receive() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());
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
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());

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
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());

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
    fn test_error_propagation() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            // Test 1: SendError::Full propagation
            {
                let (tx, _rx) = Channel::new(NonZeroUsize::new(1).unwrap());

                // Fill the channel
                let result = tx.send(1);
                assert!(result.is_ok());

                // Verify sync send fails with Full
                match tx.send(2) {
                    Err(SendError::Full(2)) => (),
                    other => panic!("Expected SendError::Full, got {other:?}"),
                }

                // Verify async send times out when full
                let send_fut = tx.send_async(3);
                match timeout(Duration::from_millis(100), send_fut).await {
                    Err(_) => (), // Expected timeout
                    Ok(result) => panic!("Expected timeout, got {result:?}"),
                }
            }

            // Test 2: SendError::Closed propagation
            {
                let (tx, _) = Channel::new(NonZeroUsize::new(1).unwrap());
                tx.close();

                // Verify sync send fails with Closed
                match tx.send(1) {
                    Err(SendError::Closed(1)) => (),
                    other => panic!("Expected SendError::Closed, got {other:?}"),
                }

                // Verify async send fails immediately with Closed
                match tx.send_async(2).await {
                    Err(SendError::Closed(2)) => (),
                    other => panic!("Expected SendError::Closed, got {other:?}"),
                }
            }

            // Test 3: RecvError propagation
            {
                let (tx, rx) = Channel::<i32>::new(NonZeroUsize::new(1).unwrap());

                // Verify TryRecvError::Empty
                match rx.try_recv() {
                    Err(RecvError::Empty) => (),
                    other => panic!("Expected TryRecvError::Empty, got {other:?}"),
                }

                // Close empty channel
                tx.close();

                // Verify TryRecvError::Closed
                match rx.try_recv() {
                    Err(RecvError::Closed) => (),
                    other => panic!("Expected TryRecvError::Closed, got {other:?}"),
                }

                // Verify async receive fails with Closed
                match rx.recv().await {
                    Err(RecvError::Closed) => (),
                    other => panic!("Expected RecvError::Closed, got {other:?}"),
                }
            }

            // Test 4: Error propagation with pending operations
            {
                let (tx, rx) = Channel::<i32>::new(NonZeroUsize::new(1).unwrap());

                // Start an async receive
                let recv_fut = rx.recv();

                // Close channel while receive is pending
                tx.close();

                // Verify pending receive gets Closed error
                match recv_fut.await {
                    Err(RecvError::Closed) => (),
                    other => panic!("Expected RecvError::Closed, got {other:?}"),
                }
            }
        });

        rt.block_on(local);
        rt.block_on(handle).expect("Test task failed");
    }

    #[test]
    fn test_backpressure() {
        let rt = create_test_runtime();
        let local = tokio::task::LocalSet::new();

        let handle = local.spawn_local(async {
            let (tx, rx) = Channel::new(NonZeroUsize::new(1).unwrap());
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
}
