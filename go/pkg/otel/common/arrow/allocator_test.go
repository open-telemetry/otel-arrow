/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow

import (
	"errors"
	"fmt"
	"testing"

	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
)

func TestLimitedAllocatorUnformatted(t *testing.T) {
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
	require.Equal(t, "memory limit exceeded: requested 1 out of 1000000 (in-use=1000000)", capture.(error).Error())

	limit.Free(b)

	check.AssertSize(t, 0)
}

func TestLimitedAllocatorFormatted(t *testing.T) {
	// Arrow does not wrap the error, so the consumer sees its
	// formatted version.
	expect := LimitError{
		Request: 1000,
		Inuse:   9900,
		Limit:   10000,
	}

	unwrap, ok := NewLimitErrorFromError(fmt.Errorf("some sort of prefix %w some sort of suffix", expect))
	require.Error(t, unwrap)
	require.True(t, ok)
	require.Equal(t, expect, unwrap)

	require.True(t, errors.Is(unwrap, LimitError{}))
}
