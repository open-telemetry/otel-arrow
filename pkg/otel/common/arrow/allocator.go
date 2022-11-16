package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/arrow/memory"
)

type LimitedAllocator struct {
	mem   memory.Allocator
	inuse uint64
	limit uint64
}

func NewLimitedAllocator(mem memory.Allocator, limit uint64) *LimitedAllocator {
	return &LimitedAllocator{
		mem:   mem,
		limit: limit,
	}
}

var _ memory.Allocator = &LimitedAllocator{}

type LimitError struct {
	Request uint64
	Inuse   uint64
	Limit   uint64
}

var _ error = LimitError{}

func (le LimitError) Error() string {
	return fmt.Sprintf("allocation size %d exceeds limit %d", le.Request, le.Limit)
}

func (_ LimitError) Is(tgt error) bool {
	_, ok := tgt.(LimitError)
	return ok
}

func (l *LimitedAllocator) Allocate(size int) []byte {
	change := uint64(size)
	if l.inuse+change > l.limit {
		panic(LimitError{
			Request: change,
			Inuse:   l.inuse,
			Limit:   l.limit,
		})
	}

	res := l.mem.Allocate(size)

	// This update will be skipped if Allocate() panics.
	l.inuse += change
	return res
}

func (l *LimitedAllocator) Reallocate(size int, b []byte) []byte {
	change := uint64(size - len(b))
	if l.inuse+change > l.limit {
		panic(LimitError{
			Request: change,
			Inuse:   l.inuse,
			Limit:   l.limit,
		})
	}

	res := l.mem.Reallocate(size, b)

	// This update will be skipped if Reallocate() panics.
	l.inuse += change
	return res
}

func (l *LimitedAllocator) Free(b []byte) {
	l.mem.Free(b)

	// This update will be skipped if Free() panics.
	l.inuse -= uint64(len(b))
}
