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
	"fmt"

	"github.com/klauspost/compress/zstd"
	"github.com/pierrec/lz4"
)

type CompressionAlgorithm uint8

const (
	None CompressionAlgorithm = iota
	Lz4
	Zstd
	//Zlib
)

func (c CompressionAlgorithm) String() string {
	switch c {
	case None:
		return "None"
	case Lz4:
		return "Lz4"
	case Zstd:
		return "Zstd"
	default:
		return "Unknown"
	}
}

func Compress(algorithm CompressionAlgorithm, data []byte) ([]byte, error) {
	switch algorithm {
	case None:
		return data, nil
	case Lz4:
		buf := make([]byte, len(data))
		ht := make([]int, 64<<10)
		n, err := lz4.CompressBlock(data, buf, ht)
		if err != nil {
			return nil, err
		}
		if n == 0 {
			// uncompressible
			buf = data
		} else {
			buf = buf[:n]
		}
		return buf, nil
	case Zstd:
		var encoder, err = zstd.NewWriter(nil)
		if err != nil {
			return nil, err
		}
		compressed := encoder.EncodeAll(data, nil)
		return compressed, nil
	default:
		return nil, fmt.Errorf("Unknown compression algorithm: %v", algorithm)
	}
}

func Decompress(algorithm CompressionAlgorithm, data []byte) ([]byte, error) {
	switch algorithm {
	case None:
		return data, nil
	case Lz4:
		decompressed := make([]byte, 10*len(data))
		n, err := lz4.UncompressBlock(data, decompressed)
		if err != nil {
			return nil, err
		}
		decompressed = decompressed[:n]
		return decompressed, nil
	case Zstd:
		var decoder, err = zstd.NewReader(nil)
		if err != nil {
			return nil, err
		}
		decompressed, err := decoder.DecodeAll(data, nil)
		if err != nil {
			return nil, err
		}
		return decompressed, nil
	default:
		return nil, fmt.Errorf("Unknown decompression algorithm: %v", algorithm)
	}
}
