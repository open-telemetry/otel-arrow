// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package zstd

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

type gint struct {
	value int
	Gen
}

func TestMRUGet(t *testing.T) {
	var m mru[*gint]
	const cnt = 5

	TTL = time.Minute

	v, g := m.Get()
	require.Nil(t, v)

	for i := 0; i < cnt; i++ {
		p := &gint{
			value: i + 1,
			Gen:   g,
		}
		m.Put(p)
	}

	for i := 0; i < cnt; i++ {
		v, _ = m.Get()
		require.Equal(t, 5-i, v.value)
	}

	v, _ = m.Get()
	require.Nil(t, v)
}

func TestMRUPut(t *testing.T) {
	var m mru[*gint]
	const cnt = 5

	TTL = 0

	g := m.Reset()

	for i := 0; i < cnt; i++ {
		p := &gint{
			value: i + 1,
			Gen:   g,
		}
		m.Put(p)
	}
	require.Equal(t, 0, m.Size())
}

// func TestMRUReset(t *testing.T) {
// 	var m mru[*gint]
// 	TTL = time.Minute
// 	m.Put(new(gint))
// 	require.Equal(t, 1, m.Size())
// 	m.Reset()
// 	require.Equal(t, 1, m.Size())
// }
