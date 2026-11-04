// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::Channel;
use crate::error::SendError;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};

// Wake requires Send + Sync; each test owns its counters and polls locally.
#[derive(Default)]
struct WakeCount(AtomicUsize);
impl WakeCount {
    fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        let _ = self.0.fetch_add(1, Ordering::Relaxed);
    }
}
fn poll<F: Future>(f: &mut Pin<Box<F>>, c: &Arc<WakeCount>) -> Poll<F::Output> {
    f.as_mut()
        .poll(&mut Context::from_waker(&Waker::from(Arc::clone(c))))
}

/// Scenario: a granted send is canceled while another sender is queued.
/// Guarantees: the released grant transfers without another receive and remains protected from immediate sends.
#[test]
fn canceling_grant_transfers_it() {
    let (tx, rx) = Channel::new(1);
    tx.send(0).unwrap();
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(1));
    let mut fb = Box::pin(tx.send_async(2));
    assert!(poll(&mut fa, &a).is_pending());
    assert!(poll(&mut fb, &b).is_pending());
    let _ = rx.try_recv().unwrap();
    assert_eq!((a.get(), b.get()), (1, 0));
    drop(fa);
    assert_eq!(b.get(), 1);
    assert!(matches!(tx.send(99), Err(SendError::Full(99))));
    assert!(matches!(poll(&mut fb, &b), Poll::Ready(Ok(()))));
    assert_eq!(rx.try_recv().unwrap(), 2);
}

/// Scenario: a non-granted waiter is spuriously polled with a replacement waker.
/// Guarantees: it cannot consume another send's grant and only its latest waker is notified on transfer.
#[test]
fn spurious_poll_preserves_ownership_and_refreshes_waker() {
    let (tx, rx) = Channel::new(1);
    tx.send(0).unwrap();
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let latest = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(1));
    let mut fb = Box::pin(tx.send_async(2));
    assert!(poll(&mut fa, &a).is_pending());
    assert!(poll(&mut fb, &b).is_pending());
    let _ = rx.try_recv().unwrap();
    assert!(poll(&mut fb, &latest).is_pending());
    drop(fa);
    assert_eq!((b.get(), latest.get()), (0, 1));
    assert!(matches!(poll(&mut fb, &latest), Poll::Ready(Ok(()))));
}

/// Scenario: a canceled queued waiter leaves a stale key whose slot is reused.
/// Guarantees: the replacement waiter receives exactly one valid grant and canceled generations cannot reclaim it.
#[test]
fn stale_keys_do_not_duplicate_grants() {
    let (tx, rx) = Channel::new(1);
    tx.send(0).unwrap();
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(1));
    assert!(poll(&mut fa, &a).is_pending());
    drop(fa);
    let mut fb = Box::pin(tx.send_async(2));
    assert!(poll(&mut fb, &b).is_pending());
    let _ = rx.try_recv().unwrap();
    assert_eq!((a.get(), b.get()), (0, 1));
    assert!(matches!(poll(&mut fb, &b), Poll::Ready(Ok(()))));
    assert!(matches!(tx.send(99), Err(SendError::Full(99))));
    assert_eq!(rx.try_recv().unwrap(), 2);
    tx.send(3).unwrap();
}

/// Scenario: a completed granted future is retained and later dropped after another grant is issued.
/// Guarantees: completion releases its bookkeeping once, and dropping the old future cannot release a newer grant.
#[test]
fn completed_future_cannot_release_a_later_grant() {
    let (tx, rx) = Channel::new(1);
    tx.send(0).unwrap();
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(1));
    let mut fb = Box::pin(tx.send_async(2));
    assert!(poll(&mut fa, &a).is_pending());
    assert!(poll(&mut fb, &b).is_pending());
    let _ = rx.try_recv().unwrap();
    assert!(matches!(poll(&mut fa, &a), Poll::Ready(Ok(()))));
    let _ = rx.try_recv().unwrap();
    assert_eq!(b.get(), 1);
    drop(fa);
    assert!(matches!(tx.send(99), Err(SendError::Full(99))));
    assert!(matches!(poll(&mut fb, &b), Poll::Ready(Ok(()))));
}

/// Scenario: shutdown occurs with both granted and queued sends.
/// Guarantees: explicit close and receiver drop release both classes with their original values and no new admission.
#[test]
fn close_rejects_granted_and_queued_sends() {
    for drop_receiver in [false, true] {
        let (tx, rx) = Channel::new(1);
        tx.send(0).unwrap();
        let a = Arc::new(WakeCount::default());
        let b = Arc::new(WakeCount::default());
        let mut fa = Box::pin(tx.send_async(1));
        let mut fb = Box::pin(tx.send_async(2));
        assert!(poll(&mut fa, &a).is_pending());
        assert!(poll(&mut fb, &b).is_pending());
        let _ = rx.try_recv().unwrap();
        if drop_receiver {
            drop(rx);
        } else {
            tx.close();
        }
        assert!(a.get() >= 1 && b.get() >= 1);
        assert!(matches!(
            poll(&mut fb, &b),
            Poll::Ready(Err(SendError::Closed(2)))
        ));
        assert!(matches!(
            poll(&mut fa, &a),
            Poll::Ready(Err(SendError::Closed(1)))
        ));
        assert!(matches!(tx.send(99), Err(SendError::Closed(99))));
    }
}

/// Scenario: the only outstanding reservation is canceled with no other queued sends.
/// Guarantees: the slot becomes immediately usable and can be filled only once.
#[test]
fn canceled_last_grant_restores_capacity() {
    let (tx, rx) = Channel::new(1);
    tx.send(0).unwrap();
    let a = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(1));
    assert!(poll(&mut fa, &a).is_pending());
    let _ = rx.try_recv().unwrap();
    drop(fa);
    tx.send(2).unwrap();
    assert!(matches!(tx.send(3), Err(SendError::Full(3))));
}

/// Scenario: a partial drain frees two slots while two senders are parked.
/// Guarantees: both sends complete through a protected handoff with no additional receive or completed-future drop.
#[test]
fn partial_drain_hands_off_before_future_drop() {
    let (tx, rx) = Channel::new(4);
    for i in 0..4 {
        tx.send(i).unwrap();
    }
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(4));
    let mut fb = Box::pin(tx.send_async(5));
    assert!(poll(&mut fa, &a).is_pending());
    assert!(poll(&mut fb, &b).is_pending());
    let _ = rx.try_recv().unwrap();
    let _ = rx.try_recv().unwrap();
    assert_eq!((a.get(), b.get()), (1, 0));
    assert!(matches!(poll(&mut fa, &a), Poll::Ready(Ok(()))));
    assert_eq!(b.get(), 1);
    assert!(matches!(tx.send(99), Err(SendError::Full(99))));
    assert!(matches!(poll(&mut fb, &b), Poll::Ready(Ok(()))));
    assert!(matches!(tx.send(99), Err(SendError::Full(99))));
    drop((fa, fb));
    for i in 2..6 {
        assert_eq!(rx.try_recv().unwrap(), i);
    }
}

/// Scenario: a full buffer is drained while several senders are parked.
/// Guarantees: at least one sender gets a grant, and immediate sends can use only capacity left unreserved.
#[test]
fn granted_sender_survives_greedy_immediate_sends() {
    let (tx, rx) = Channel::new(4);
    for i in 0..4 {
        tx.send(i).unwrap();
    }
    let a = Arc::new(WakeCount::default());
    let b = Arc::new(WakeCount::default());
    let mut fa = Box::pin(tx.send_async(10));
    let mut fb = Box::pin(tx.send_async(11));
    assert!(poll(&mut fa, &a).is_pending());
    assert!(poll(&mut fb, &b).is_pending());
    for _ in 0..4 {
        let _ = rx.try_recv().unwrap();
    }
    assert_eq!(a.get(), 1);
    let mut filled = 0;
    while tx.send(99).is_ok() {
        filled += 1;
    }
    assert!(filled < 4);
    assert!(matches!(poll(&mut fa, &a), Poll::Ready(Ok(()))));
}

/// Scenario: greedy producers are repeatedly polled in fixed index order and fill all unreserved capacity.
/// Guarantees: every parked producer still completes a send under this adverse ordering; full buffers remain bounded.
#[test]
fn greedy_poll_order_cannot_starve_registered_producers() {
    const N: usize = 16;
    for capacity in [2, 16, 256] {
        let (tx, rx) = Channel::new(capacity);
        for _ in 0..capacity {
            tx.send(99).unwrap();
        }
        let senders: Vec<_> = (0..N).map(|_| tx.clone()).collect();
        let counts: Vec<_> = (0..N).map(|_| Arc::new(WakeCount::default())).collect();
        let mut futures: Vec<_> = senders
            .iter()
            .enumerate()
            .map(|(i, tx)| Box::pin(tx.send_async(i)))
            .collect();
        let mut completed = [0usize; N];
        let mut observed_wakes = [0usize; N];
        for i in 0..N {
            assert!(poll(&mut futures[i], &counts[i]).is_pending());
        }
        for _ in 0..4 * N {
            let mut drained = 0;
            while rx.try_recv().is_ok() {
                drained += 1;
            }
            assert!(drained <= capacity);
            for i in 0..N {
                let wakes = counts[i].get();
                if wakes == observed_wakes[i] {
                    continue;
                }
                observed_wakes[i] = wakes;
                // A notification is a grant, so even an adversarially late poll must succeed.
                assert!(matches!(
                    poll(&mut futures[i], &counts[i]),
                    Poll::Ready(Ok(()))
                ));
                completed[i] += 1;
                while senders[i].send(i).is_ok() {}
                futures[i] = Box::pin(senders[i].send_async(i));
                assert!(poll(&mut futures[i], &counts[i]).is_pending());
            }
        }
        assert!(
            completed.iter().all(|n| *n > 0),
            "capacity={capacity}, completed={completed:?}"
        );
    }
}

/// Scenario: the oldest granted future is retained without being polled while the receiver keeps draining other work.
/// Guarantees: the drain-boundary grant lets later waiters make progress without waiting for that future.
#[test]
fn coalesced_progress_bypasses_a_delayed_grant_holder() {
    let (tx, rx) = Channel::new(4);
    for i in 0..4 {
        tx.send(i).unwrap();
    }
    let mut a = Box::pin(tx.send_async(10));
    let mut b = Box::pin(tx.send_async(11));
    let mut c = Box::pin(tx.send_async(12));
    let b_count = Arc::new(WakeCount::default());
    let c_count = Arc::new(WakeCount::default());
    assert!(
        a.as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    assert!(poll(&mut b, &b_count).is_pending());
    assert!(poll(&mut c, &c_count).is_pending());
    for _ in 0..4 {
        let _ = rx.try_recv().unwrap();
    }
    // A keeps its grant but is deliberately not polled; B must be notified independently.
    assert_eq!(b_count.get(), 1);
    assert_eq!(c_count.get(), 0);
    assert!(matches!(poll(&mut b, &b_count), Poll::Ready(Ok(()))));
    assert_eq!(rx.try_recv().unwrap(), 11);
    assert_eq!(c_count.get(), 1);
    assert!(matches!(poll(&mut c, &c_count), Poll::Ready(Ok(()))));
    assert_eq!(rx.try_recv().unwrap(), 12);
    assert!(matches!(
        a.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Ok(()))
    ));
}

/// Scenario: seeded sends, receives, cancellations, and spurious polls interleave across several capacities.
/// Guarantees: grants have live owners and stay bounded, queued work retains a grant holder when space is free, and accepted data is conserved.
#[test]
fn seeded_reservation_accounting_and_data_conservation() {
    for capacity in [1, 2, 4, 16] {
        let (tx, rx) = Channel::new(capacity);
        let mut pending: Vec<_> = (0..8).map(|_| None).collect();
        let mut rng = 4569u64;
        let mut next_value = 0u64;
        let mut accepted = HashSet::new();
        let mut received = HashSet::new();
        let mut cx = Context::from_waker(Waker::noop());
        for _ in 0..10_000 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let index = (rng as usize >> 8) % pending.len();
            match rng % 5 {
                0 => {
                    if pending[index].is_none() {
                        pending[index] = Some((next_value, Box::pin(tx.send_async(next_value))));
                        next_value += 1;
                    }
                }
                1 => {
                    if let Some((value, future)) = pending[index].as_mut() {
                        match future.as_mut().poll(&mut cx) {
                            Poll::Ready(Ok(())) => {
                                assert!(accepted.insert(*value));
                                pending[index] = None;
                            }
                            Poll::Pending => {}
                            Poll::Ready(Err(e)) => panic!("unexpected send failure: {e:?}"),
                        }
                    }
                }
                2 => {
                    pending[index] = None;
                }
                3 => {
                    if let Ok(value) = rx.try_recv() {
                        assert!(accepted.contains(&value));
                        assert!(received.insert(value));
                    }
                }
                _ => {
                    if tx.send(next_value).is_ok() {
                        assert!(accepted.insert(next_value));
                    }
                    next_value += 1;
                }
            }
            let state = tx.channel.state.borrow();
            assert!(state.buffer.len() + state.reserved_slots <= capacity);
            assert_eq!(accepted.len(), received.len() + state.buffer.len());
            for value in &state.buffer {
                assert!(accepted.contains(value) && !received.contains(value));
            }
            if let Some(waiters) = &state.sender_waiters {
                let reserved = waiters
                    .slots
                    .iter()
                    .filter(|slot| slot.in_use && slot.reserved)
                    .count();
                assert_eq!(state.reserved_slots, reserved);
                for slot in &waiters.slots {
                    if slot.reserved {
                        assert!(slot.in_use && !slot.queued && slot.waker.is_none());
                    }
                }
                if waiters.slots.iter().any(|slot| slot.in_use && slot.queued) {
                    assert!(
                        state.buffer.len() + state.reserved_slots == capacity
                            || state.reserved_slots > 0
                    );
                }
            } else {
                assert_eq!(state.reserved_slots, 0);
            }
        }
        drop(pending);
        assert_eq!(tx.channel.state.borrow().reserved_slots, 0);
        while let Ok(value) = rx.try_recv() {
            assert!(received.insert(value));
        }
        assert_eq!(accepted, received);
    }
}
