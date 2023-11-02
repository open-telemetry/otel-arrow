package zstd

import (
	"sync"
	"time"
)

// mru is a freelist whose two main benefits compared to sync.Pool are:
//
//   - It doesn't perform any per-CPU caching; it has only a single
//     cache. The cache is modelled as a stack, meaning that the most
//     recently used item is always the next to be used. (Hence the name
//     MRU.)
//
//   - It isn't cleared when GC runs. Instead, items that haven't been used
//     in a long time (1min) are released.
//
// An MRU freelist is most useful when the objects being freelisted are
// sufficiently valuable, or expensive to create, that they are worth keeping
// across GC passes. The drawbacks are that MRU isn't as performant under
// heavy concurrent access as sync.Pool, and that its sizing logic (1min TTL)
// is less sophisticated than sync.Pool's.
//
// A zero-initialized MRU is safe to use. Threadsafe.
type mru[T any] struct {
	mu       sync.Mutex
	freelist []T
	putTimes []time.Time // putTimes[i] is when freelist[i] was Put()
	zero     T
}

// Get returns an object from the freelist. If the list is empty, the return
// value is the zero value of T.
func (mru *mru[T]) Get() T {
	mru.mu.Lock()
	defer mru.mu.Unlock()

	if n := len(mru.freelist); n > 0 {
		ret := mru.freelist[n-1]
		mru.freelist[n-1] = mru.zero // Allow GC to occur.
		mru.freelist = mru.freelist[:n-1]
		mru.putTimes = mru.putTimes[:n-1]
		return ret
	}

	return mru.zero
}

func (mru *mru[T]) Put(item T) {
	mru.mu.Lock()
	defer mru.mu.Unlock()

	mru.freelist = append(mru.freelist, item)
	mru.putTimes = append(mru.putTimes, time.Now())

	// Evict any objects that haven't been touched recently.
	const ttl = time.Minute
	for len(mru.putTimes) > 0 && time.Since(mru.putTimes[0]) > ttl {
		mru.freelist[0] = mru.zero // Allow GC to occur.
		mru.freelist = mru.freelist[1:]
		mru.putTimes = mru.putTimes[1:]
	}
}
