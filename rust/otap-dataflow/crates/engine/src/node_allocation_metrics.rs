// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in allocation activity recorded while a node task is being polled.

use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry_macros::metric_set;
use std::cell::{Cell, RefCell};
use std::future::{Future, poll_fn};
use std::pin::pin;
use std::rc::Rc;

#[metric_set(name = "node.allocator_activity")]
#[derive(Debug, Default)]
pub(crate) struct NodeAllocationMetrics {
    /// Successful allocation calls executed during node polls.
    #[metric(unit = "{call}")]
    allocations: Counter<u64>,
    /// Requested bytes in successful allocation calls during node polls.
    #[metric(unit = "By")]
    allocated_bytes: Counter<u64>,
    /// Deallocation calls executed during node polls.
    #[metric(unit = "{call}")]
    deallocations: Counter<u64>,
    /// Requested bytes in deallocation calls during node polls.
    #[metric(unit = "By")]
    deallocated_bytes: Counter<u64>,
    /// Successful reallocation calls executed during node polls.
    #[metric(unit = "{call}")]
    reallocations: Counter<u64>,
    /// Bytes copied by reallocations during node polls.
    #[metric(unit = "By")]
    realloc_copied_bytes: Counter<u64>,
}

pub(crate) type NodeAllocationHandle = Rc<RefCell<MetricSet<NodeAllocationMetrics>>>;

#[derive(Clone, Copy, Default, PartialEq)]
struct Activity {
    allocations: u64,
    allocated: u64,
    deallocations: u64,
    deallocated: u64,
    reallocations: u64,
    copied: u64,
}

thread_local! {
    static ACTIVITY: Cell<Option<Activity>> = const { Cell::new(None) };
}

#[inline]
fn record(update: impl FnOnce(&mut Activity)) {
    let _ = ACTIVITY.try_with(|counter| {
        if let Some(mut activity) = counter.get() {
            update(&mut activity);
            counter.set(Some(activity));
        }
    });
}

#[inline]
pub(crate) fn allocated(bytes: usize) {
    record(|activity| {
        activity.allocations += 1;
        activity.allocated += bytes as u64;
    });
}

#[inline]
pub(crate) fn freed(bytes: usize) {
    record(|activity| {
        activity.deallocations += 1;
        activity.deallocated += bytes as u64;
    });
}

#[inline]
pub(crate) fn reallocated(copied: usize) {
    record(|activity| {
        activity.reallocations += 1;
        activity.copied += copied as u64;
    });
}

// Suspend any enclosing capture while this task is polled and restore it
// before recording telemetry, including when the poll unwinds.
struct PollActivity<'a> {
    counter: &'a Cell<Option<Activity>>,
    previous: Option<Activity>,
}

impl Drop for PollActivity<'_> {
    fn drop(&mut self) {
        self.counter.set(self.previous);
    }
}

fn capture<T>(poll: impl FnOnce() -> T) -> (T, Activity) {
    ACTIVITY.with(|counter| {
        let scope = PollActivity {
            counter,
            previous: counter.replace(Some(Activity::default())),
        };
        let result = poll();
        let activity = counter.get().unwrap_or_default();
        drop(scope);
        (result, activity)
    })
}

pub(crate) async fn measure_node<F: Future>(metrics: NodeAllocationHandle, future: F) -> F::Output {
    let mut future = pin!(future);
    poll_fn(|cx| {
        let (result, activity) = capture(|| future.as_mut().poll(cx));
        if activity != Activity::default() {
            let mut metrics = metrics.borrow_mut();
            metrics.allocations.add(activity.allocations);
            metrics.allocated_bytes.add(activity.allocated);
            metrics.deallocations.add(activity.deallocations);
            metrics.deallocated_bytes.add(activity.deallocated);
            metrics.reallocations.add(activity.reallocations);
            metrics.realloc_copied_bytes.add(activity.copied);
        }
        result
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline_allocator::PipelineAllocator;
    use std::alloc::{GlobalAlloc, Layout};
    use std::task::{Context, Poll, Waker};

    /// Scenario: A node allocates before suspension and grows and frees its
    /// allocation after resuming, with unrelated worker activity between polls.
    /// Guarantees: Each poll reports only its own activity, and a moving realloc
    /// counts as one realloc with copied bytes instead of a separate alloc/free.
    #[test]
    #[allow(unsafe_code)]
    fn activity_is_scoped_to_each_poll_and_counts_realloc_once() {
        let old = Layout::from_size_align(64, 8).unwrap();
        let new = Layout::from_size_align(512, 8).unwrap();
        let mut block = std::ptr::null_mut::<u8>();
        let mut task = pin!(poll_fn(|_| {
            // SAFETY: Each successful allocation is used with its actual layout.
            unsafe {
                if block.is_null() {
                    block = PipelineAllocator.alloc(old);
                    assert!(!block.is_null());
                    block.write_bytes(42, old.size());
                    Poll::Pending
                } else {
                    let grown = PipelineAllocator.realloc(block, old, new.size());
                    if grown.is_null() {
                        PipelineAllocator.dealloc(block, old);
                        panic!("reallocation failed");
                    }
                    PipelineAllocator.dealloc(grown, new);
                    Poll::Ready(())
                }
            }
        }));
        let mut cx = Context::from_waker(Waker::noop());

        let (pending, first) = capture(|| task.as_mut().poll(&mut cx));
        assert!(pending.is_pending());
        assert_eq!((first.allocations, first.allocated), (1, 64));
        assert_eq!((first.deallocations, first.reallocations), (0, 0));

        allocated(128);
        freed(128);
        let (completed, second) = capture(|| task.as_mut().poll(&mut cx));
        assert!(completed.is_ready());
        assert_eq!(second.allocations, 0);
        assert_eq!((second.reallocations, second.copied), (1, 64));
        assert_eq!((second.deallocations, second.deallocated), (1, 512));
    }
}
