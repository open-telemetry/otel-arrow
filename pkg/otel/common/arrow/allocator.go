// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package arrow

import (
	"fmt"
	"os"

	"github.com/apache/arrow/go/v12/arrow/memory"
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
	return fmt.Sprintf("allocation size %d exceeds limit %d (in-use=%d)", le.Request, le.Limit, le.Inuse)
}

func (_ LimitError) Is(tgt error) bool {
	_, ok := tgt.(LimitError)
	return ok
}

func (l *LimitedAllocator) Allocate(size int) []byte {
	change := uint64(size)
	if l.inuse+change > l.limit {
		err := LimitError{
			Request: change,
			Inuse:   l.inuse,
			Limit:   l.limit,
		}
		// Write the error to stderr so that it is visible even if the
		// panic is caught.
		os.Stderr.WriteString(err.Error() + "\n")
		panic(err)
	}

	res := l.mem.Allocate(size)

	// This update will be skipped if Allocate() panics.
	l.inuse += change
	return res
}

func (l *LimitedAllocator) Reallocate(size int, b []byte) []byte {
	change := uint64(size - len(b))
	if l.inuse+change > l.limit {
		err := LimitError{
			Request: change,
			Inuse:   l.inuse,
			Limit:   l.limit,
		}
		// Write the error to stderr so that it is visible even if the
		// panic is caught.
		os.Stderr.WriteString(err.Error() + "\n")
		panic(err)
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
