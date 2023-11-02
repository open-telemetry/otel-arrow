// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package zstd

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestMRUGet(t *testing.T) {
	var m mru[*int]
	const cnt = 5

	TTL = time.Minute

	require.Nil(t, m.Get())

	for i := 0; i < cnt; i++ {
		p := new(int)
		*p += i + 1
		m.Put(p)
	}

	for i := 0; i < cnt; i++ {
		require.Equal(t, 5-i, *m.Get())
	}

	require.Nil(t, m.Get())
}

func TestMRUPut(t *testing.T) {
	var m mru[*int]
	const cnt = 5

	TTL = 0

	for i := 0; i < cnt; i++ {
		p := new(int)
		*p += i + 1
		m.Put(p)
	}
	// Up to 1 can be kept, w/ 0 TTL, or 1 in case the
	// time.Since(time.Now()) returns 0.
	require.LessOrEqual(t, m.Size(), 1)
}
