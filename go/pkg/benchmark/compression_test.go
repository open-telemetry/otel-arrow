/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package benchmark

import (
	"testing"
)

func TestLz4(t *testing.T) {
	t.Parallel()

	lz4 := Lz4()
	compressed, err := lz4.Compress([]byte("This is an example of text to compress.This is an example of text to compress.This is an example of text to compress."))
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	decompressed, err := lz4.Decompress(compressed)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if string(decompressed) != "This is an example of text to compress.This is an example of text to compress.This is an example of text to compress." {
		t.Errorf("expected decompressed data to be 'test', got %v", string(decompressed))
	}
}

func TestZstd(t *testing.T) {
	t.Parallel()

	zstd := Zstd()
	compressed, err := zstd.Compress([]byte("This is an example of text to compress.This is an example of text to compress.This is an example of text to compress."))
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	decompressed, err := zstd.Decompress(compressed)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if string(decompressed) != "This is an example of text to compress.This is an example of text to compress.This is an example of text to compress." {
		t.Errorf("expected decompressed data to be 'test', got %v", string(decompressed))
	}
}
