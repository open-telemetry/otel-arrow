package benchmark

import (
	"otel-arrow-adapter/pkg/benchmark"
	"testing"
)

func TestLz4(t *testing.T) {
	t.Parallel()

	compressed, err := benchmark.Compress(benchmark.Lz4, []byte("This is an example of text to compress.This is an example of text to compress.This is an example of text to compress."))
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	decompressed, err := benchmark.Decompress(benchmark.Lz4, compressed)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if string(decompressed) != "This is an example of text to compress.This is an example of text to compress.This is an example of text to compress." {
		t.Errorf("expected decompressed data to be 'test', got %v", string(decompressed))
	}
}

func TestZstd(t *testing.T) {
	t.Parallel()

	compressed, err := benchmark.Compress(benchmark.Zstd, []byte("This is an example of text to compress.This is an example of text to compress.This is an example of text to compress."))
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	decompressed, err := benchmark.Decompress(benchmark.Zstd, compressed)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if string(decompressed) != "This is an example of text to compress.This is an example of text to compress.This is an example of text to compress." {
		t.Errorf("expected decompressed data to be 'test', got %v", string(decompressed))
	}
}
