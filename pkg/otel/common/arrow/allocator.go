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
	"regexp"
	"strconv"
	"strings"

	"github.com/apache/arrow/go/v16/arrow/memory"
)

// MemoryErrorStringPrefix is a prefix used to recognize memory limit errors.
//
// Note: the arrow/go package (as of v16) has a panic recovery
// mechanism which formats the error object raised through panic in
// the code below.  The formatting uses a "%v" which means we lose the
// error wrapping facility that would let us easily extract the
// object.  Therefore, we use a regexp to unpack memory limit errors.
const MemoryErrorStringPrefix = "allocation size exceeds limit"

type LimitedAllocator struct {
	Allocator memory.Allocator
	inuse     uint64
	limit     uint64
}

func NewLimitedAllocator(allocator memory.Allocator, limit uint64) *LimitedAllocator {
	return &LimitedAllocator{
		Allocator: allocator,
		limit:     limit,
	}
}

var _ memory.Allocator = &LimitedAllocator{}

type LimitError struct {
	Request uint64
	Inuse   uint64
	Limit   uint64
}

var _ error = LimitError{}

var limitRegexp = regexp.MustCompile(`requested (\d+) out of (\d+) \(in-use=(\d+)\)`)

// NewLimitErrorFromError extracts a formatted limit error.  See
// MemoryErrorStringPrefix for an explanation.
func NewLimitErrorFromError(err error) (error, bool) {
	msg := err.Error()
	if !strings.Contains(msg, MemoryErrorStringPrefix) {
		fmt.Println("CASE A", err)
		return err, false
	}
	matches := limitRegexp.FindStringSubmatch(msg)
	if len(matches) != 4 {
		fmt.Println("CASE B", err)
		return err, false
	}

	req, _ := strconv.ParseUint(matches[1], 10, 64)
	lim, _ := strconv.ParseUint(matches[2], 10, 64)
	inuse, _ := strconv.ParseUint(matches[3], 10, 64)

	fmt.Println("CASE C")
	return LimitError{
		Request: req,
		Inuse:   inuse,
		Limit:   lim,
	}, true
}

func (le LimitError) Error() string {
	return fmt.Sprintf("%s: requested %d out of %d (in-use=%d)", MemoryErrorStringPrefix, le.Request, le.Limit, le.Inuse)
}

func (_ LimitError) Is(tgt error) bool {
	if _, ok := tgt.(LimitError); ok {
		return true
	}
	_, ok := tgt.(MemoryLimitClassifier)
	return ok
}

type MemoryLimitClassifier struct {
}

var _ error = MemoryLimitClassifier{}

func (MemoryLimitClassifier) Error() string {
	return "error class: memory limit exceeded"
}

func (l *LimitedAllocator) Inuse() uint64 {
	return l.inuse
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

	res := l.Allocator.Allocate(size)

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

	res := l.Allocator.Reallocate(size, b)

	// This update will be skipped if Reallocate() panics.
	l.inuse += change
	return res
}

func (l *LimitedAllocator) Free(b []byte) {
	l.Allocator.Free(b)

	// This update will be skipped if Free() panics.
	l.inuse -= uint64(len(b))
}
