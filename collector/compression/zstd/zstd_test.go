// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package zstd

import (
	"bytes"
	"fmt"
	"testing"
	"time"

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

func TestCompressorReset(t *testing.T) {
	TTL = time.Minute

	// Get compressor configs 1 and 2.
	comp1 := encoding.GetCompressor("zstdarrow1").(*combined)
	comp2 := encoding.GetCompressor("zstdarrow2").(*combined)

	// Get an object for level 1
	var buf bytes.Buffer
	wc, err := comp1.Compress(&buf)
	require.NoError(t, err)

	// Put back the once, it will be saved.
	save := wc.(*writer)
	require.NoError(t, wc.Close())
	require.Equal(t, 1, comp1.enc.pool.Size())
	require.Equal(t, 0, comp2.enc.pool.Size())

	// We get the same object pointer again.
	wc, err = comp1.Compress(&buf)
	require.NoError(t, err)
	require.Equal(t, save, wc.(*writer))

	// Modify 1's encoder configuration.
	encCfg1 := comp1.enc.getConfig()
	encCfg2 := comp2.enc.getConfig()
	cpyCfg1 := encCfg1
	cpyCfg1.WindowSizeMiB = 32

	require.Equal(t, Level(1), cpyCfg1.Level)
	require.NotEqual(t, cpyCfg1, encCfg1, "see %v %v", cpyCfg1, encCfg1)

	require.NoError(t, SetEncoderConfig(cpyCfg1))

	// The instances can't have changed.
	require.Equal(t, comp1, encoding.GetCompressor("zstdarrow1").(*combined))
	require.Equal(t, comp2, encoding.GetCompressor("zstdarrow2").(*combined))

	// Level 2 is unchanged
	require.Equal(t, encCfg2, comp2.enc.getConfig())

	// Level 1 is changed
	require.NotEqual(t, encCfg1, comp1.enc.getConfig(), "see %v %v", encCfg1, comp1.enc.getConfig())

	// Put back the saved item, it will not be placed back in the
	// pool due to reset.
	require.NoError(t, wc.Close())
	require.Equal(t, 0, comp1.enc.pool.Size())
	// Explicitly, we get a nil from the pool.
	v, _ := comp1.enc.pool.Get()
	require.Nil(t, v)
}

func TestDecompressorReset(t *testing.T) {
	TTL = time.Minute

	// Get compressor configs 1 and 2.
	comp1 := encoding.GetCompressor("zstdarrow1").(*combined)
	comp2 := encoding.GetCompressor("zstdarrow2").(*combined)

	// Get an object for level 1
	buf := new(bytes.Buffer)
	rd, err := comp1.Decompress(buf)
	require.NoError(t, err)
	_, err = rd.Read([]byte{})
	require.Error(t, err)

	// We get the same object pointer again.
	buf = new(bytes.Buffer)
	rd, err = comp1.Decompress(buf)
	require.NoError(t, err)
	_, err = rd.Read(nil)
	require.Error(t, err)

	// Modify 1's encoder configuration.
	decCfg1 := comp1.dec.getConfig()
	decCfg2 := comp2.dec.getConfig()
	cpyCfg1 := decCfg1
	cpyCfg1.MaxWindowSizeMiB = 128

	require.Equal(t, Level(1), cpyCfg1.Level)
	require.NotEqual(t, cpyCfg1, decCfg1, "see %v %v", cpyCfg1, decCfg1)

	require.NoError(t, SetDecoderConfig(cpyCfg1))

	// The instances can't have changed.
	require.Equal(t, comp1, encoding.GetCompressor("zstdarrow1").(*combined))
	require.Equal(t, comp2, encoding.GetCompressor("zstdarrow2").(*combined))

	// Level 2 is unchanged
	require.Equal(t, decCfg2, comp2.dec.getConfig())

	// Level 1 is changed
	require.NotEqual(t, decCfg1, comp1.dec.getConfig(), "see %v %v", decCfg1, comp1.dec.getConfig())

	// Unlike the encoder test, which has an explicit Close() to its advantage,
	// we aren't testing the behavior of the finalizer that puts back into the MRU.
}
