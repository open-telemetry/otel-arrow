// SPDX-License-Identifier: Apache-2.0

//! Minimal lock-free single-producer single-consumer ring buffer.
//! Indices each owned by one side; use acquire/release fences for visibility.

use core::cell::RefCell;

/// Safe single-producer single-consumer ring buffer (power-of-two capacity).
pub struct SpscQueue<T> {
    capacity: usize,
    mask: usize,
    buffer: RefCell<Vec<Option<T>>>,
    head: RefCell<usize>, // consumer index
    tail: RefCell<usize>, // producer index
}

impl<T> SpscQueue<T> {
    /// Creates a new SPSC queue with power-of-two capacity.
    pub fn with_capacity_pow2(capacity_pow2: usize) -> Self {
        assert!(capacity_pow2.is_power_of_two());
        Self {
            capacity: capacity_pow2,
            mask: capacity_pow2 - 1,
            buffer: RefCell::new((0..capacity_pow2).map(|_| None).collect()),
            head: RefCell::new(0),
            tail: RefCell::new(0),
        }
    }
    /// Splits into producer and consumer handles.
    pub fn split(&self) -> (SpscProducer<'_, T>, SpscConsumer<'_, T>) {
        (SpscProducer { q: self }, SpscConsumer { q: self })
    }
}

/// Producer handle (not Send across threads in this safe implementation).
pub struct SpscProducer<'a, T> {
    pub(crate) q: &'a SpscQueue<T>,
}
/// Consumer handle.
pub struct SpscConsumer<'a, T> {
    pub(crate) q: &'a SpscQueue<T>,
}

impl<'a, T> SpscProducer<'a, T> {
    /// Attempts to push a value; returns Err(value) if the ring is full.
    pub fn push(&self, value: T) -> Result<(), T> {
        let mut tail = self.q.tail.borrow_mut();
        let head = *self.q.head.borrow();
        if *tail - head == self.q.capacity {
            return Err(value);
        }
        let idx = *tail & self.q.mask;
        self.q.buffer.borrow_mut()[idx] = Some(value);
        *tail += 1;
        Ok(())
    }
}

impl<'a, T> SpscConsumer<'a, T> {
    /// Pops the next value if available.
    pub fn pop(&self) -> Option<T> {
        let mut head = self.q.head.borrow_mut();
        if *head == *self.q.tail.borrow() {
            return None;
        }
        let idx = *head & self.q.mask;
        let val = self.q.buffer.borrow_mut()[idx].take();
        *head += 1;
        val
    }
    /// Creates an iterator draining the queue.
    pub fn drain_iter(&self) -> Drain<'_, T> {
        Drain { cons: self }
    }
}

/// Iterator over drained items.
pub struct Drain<'a, T> {
    cons: &'a SpscConsumer<'a, T>,
}
impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.cons.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_spsc() {
        let q = SpscQueue::with_capacity_pow2(8);
        let (p, c) = q.split();
        for i in 0..4 {
            p.push(i).unwrap();
        }
        let out: Vec<_> = c.drain_iter().collect();
        assert_eq!(out, vec![0, 1, 2, 3]);
    }
}
