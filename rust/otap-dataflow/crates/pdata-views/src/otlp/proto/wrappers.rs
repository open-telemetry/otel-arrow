// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! To ensure design flexibility, we want to introduce a layer of abstraction over access to OTLP
//! data. That's a bit tricky beacuse OTLP structures are object trees with iterators for different
//! child structures. This module gives you some convenient tools to create indirection types. For
//! example, for the OTLP struct `ResourceSpans`, we can construct a wrapper that holds references
//! to it like this:
//!
//! ```compile_fail
//! // These doctests are marked as failing because we want this module to be private and you can't
//! // doctest non-public items.
//! pub type ObjResourceSpans<'a> = GenericObj<'a, ResourceSpans>;
//! ```
//!
//! We can then create an iterator struct that consumes slice iterators of `ResourceSpans` and
//! yields elements of `ObjResourceSpans` like so:
//! ```compile_fail
//! pub type ResourceIter<'a> = GenericIterator<'a, ResourceSpans, ObjResourceSpans<'a>>;
//! ```
use std::{marker::PhantomData, slice};

/// A generic iterator that can wrap the `std::slice::Iter` for any type `Inner` with lifetime
/// `'a`. This iterator yields elements of type `Outer`. Given that it starts with an `Inner` and
/// must yield `Outer` elements, how does that work? Well, it only works because `Inner` and `Outer`
/// are related by the `Wraps` trait which ensures that given an instance of type `Inner`, we can
/// always construct a corresponding instance of type `Outer`.
#[derive(Clone)]
pub struct GenericIterator<'a, Inner, Outer> {
    it: slice::Iter<'a, Inner>,
    _outer: PhantomData<Outer>,
}

impl<'a, Inner, Outer> GenericIterator<'a, Inner, Outer> {
    /// Make a new `GenericIterator` given a slice iterator over `Inner`s.
    #[must_use]
    pub fn new(it: slice::Iter<'a, Inner>) -> Self {
        Self {
            it,
            _outer: PhantomData,
        }
    }
}

/// Any type `T` that implements `Wraps<'a, Inner>` guarantees that given a reference with lifetime
/// `'a` to a `Inner`, we can make an instance of `T` that stores that reference.
pub trait Wraps<'a, Inner> {
    /// Construct a new instance of `Self` given a reference to a value of type `Inner`.
    fn new(inner: &'a Inner) -> Self;
}

// Now we have all the pieces needed to make `GenericIterator` an iterator.
impl<'a, Inner, Outer> Iterator for GenericIterator<'a, Inner, Outer>
where
    Outer: Wraps<'a, Inner>,
{
    type Item = Outer;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(Outer::new)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    // FIXME: would there be any performance benefit to using a macro crate to delegate all the
    // default `Iterator` methods directly to `self.it`? We'd get the same results either way, but
    // maybe the optimizer would do better if we didn't have to continually dereference
    // `self.it.next()`?
}

/// `GenericObj` is a wrapper for storing references to some type `Inner`.
#[derive(Clone)]
pub struct GenericObj<'a, Inner> {
    /// The reference that we're wrapping
    pub inner: &'a Inner,
}

// Of course, `GenericObj` implements `Wraps` for, well, everything.
impl<'a, Inner> Wraps<'a, Inner> for GenericObj<'a, Inner> {
    fn new(inner: &'a Inner) -> Self {
        GenericObj { inner }
    }
}
