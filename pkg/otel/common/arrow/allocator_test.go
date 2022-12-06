package arrow

import (
	"errors"
	"testing"

	"github.com/apache/arrow/go/v10/arrow/memory"
	"github.com/stretchr/testify/require"
)

func TestLimitedAllocator(t *testing.T) {
	const boundary = 1000000
	check := memory.NewCheckedAllocator(memory.NewGoAllocator())
	limit := NewLimitedAllocator(check, boundary)

	var b []byte
	check.AssertSize(t, 0)

	b = limit.Allocate(boundary)
	require.NotNil(t, b)

	var capture interface{}

	func() {
		defer func() {
			if ret := recover(); ret != nil {
				capture = ret
			}
		}()

		_ = limit.Allocate(1)
		panic("not reached")
	}()
	require.NotNil(t, capture)
	require.True(t, errors.Is(capture.(error), LimitError{}))
	require.Equal(t, "allocation size 1 exceeds limit 1000000", capture.(error).Error())

	limit.Free(b)

	check.AssertSize(t, 0)
}
