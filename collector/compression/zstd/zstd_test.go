// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package zstd

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/encoding"
)

func TestCompressorNonNil(t *testing.T) {
	for i := 1; i <= 10; i++ {
		require.NotNil(t, encoding.GetCompressor(fmt.Sprint(NamePrefix, i)))
	}
	require.Nil(t, encoding.GetCompressor(fmt.Sprint(NamePrefix, MinLevel-1)))
	require.Nil(t, encoding.GetCompressor(fmt.Sprint(NamePrefix, MaxLevel+1)))
}
