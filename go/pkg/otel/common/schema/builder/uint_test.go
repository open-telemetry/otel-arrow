/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package builder

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
)

// Scenario: Uint16DeltaBuilder.Append receives a value smaller than the
// previously appended value (as happens when a uint16 record-ID counter wraps
// past 65535 back to 0).
// Guarantees: Append returns an error instead of panicking, so callers can
// surface a clean error rather than crashing the process.
func TestUint16DeltaBuilderAppendDecreasingValueReturnsError(t *testing.T) {
	pool := memory.NewGoAllocator()
	b := NewUint16DeltaBuilder(array.NewUint16Builder(pool), nil, nil)

	require.NoError(t, b.Append(65535))

	err := b.Append(0)
	require.Error(t, err)
	require.Contains(t, err.Error(), "less than previous value")
}

// Scenario: Uint16DeltaBuilder.Append receives a value whose delta from the
// previous value exceeds the configured max delta.
// Guarantees: Append returns an error instead of panicking.
func TestUint16DeltaBuilderAppendDeltaTooLargeReturnsError(t *testing.T) {
	pool := memory.NewGoAllocator()
	b := NewUint16DeltaBuilder(array.NewUint16Builder(pool), nil, nil)
	b.SetMaxDelta(1)

	require.NoError(t, b.Append(0))

	err := b.Append(5)
	require.Error(t, err)
	require.Contains(t, err.Error(), "greater than max delta")
}

// Scenario: Uint16DeltaBuilder.Append receives monotonically increasing values
// within the max delta.
// Guarantees: Append succeeds without error for the valid case.
func TestUint16DeltaBuilderAppendIncreasingValueSucceeds(t *testing.T) {
	pool := memory.NewGoAllocator()
	b := NewUint16DeltaBuilder(array.NewUint16Builder(pool), nil, nil)
	b.SetMaxDelta(1)

	require.NoError(t, b.Append(0))
	require.NoError(t, b.Append(1))
	require.NoError(t, b.Append(2))
}

// Scenario: Uint32DeltaBuilder.Append receives a value smaller than the
// previously appended value.
// Guarantees: Append returns an error instead of panicking.
func TestUint32DeltaBuilderAppendDecreasingValueReturnsError(t *testing.T) {
	pool := memory.NewGoAllocator()
	b := NewUint32DeltaBuilder(array.NewUint32Builder(pool), nil, nil)

	require.NoError(t, b.Append(100))

	err := b.Append(0)
	require.Error(t, err)
	require.Contains(t, err.Error(), "less than previous value")
}

// Scenario: Uint32DeltaBuilder.Append receives a value whose delta from the
// previous value exceeds the configured max delta.
// Guarantees: Append returns an error instead of panicking.
func TestUint32DeltaBuilderAppendDeltaTooLargeReturnsError(t *testing.T) {
	pool := memory.NewGoAllocator()
	b := NewUint32DeltaBuilder(array.NewUint32Builder(pool), nil, nil)
	b.SetMaxDelta(1)

	require.NoError(t, b.Append(0))

	err := b.Append(5)
	require.Error(t, err)
	require.Contains(t, err.Error(), "greater than max delta")
}
