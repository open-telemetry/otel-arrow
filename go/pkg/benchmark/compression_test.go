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
