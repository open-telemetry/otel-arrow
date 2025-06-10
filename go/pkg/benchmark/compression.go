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
	"bytes"
	"fmt"

	"github.com/klauspost/compress/zstd"
	"github.com/pierrec/lz4"
)

const CompressionTypeZstd = "zstd"
const CompressionTypeNone = ""

type CompressionAlgorithm interface {
	fmt.Stringer
	Compress(data []byte) ([]byte, error)
	Decompress(data []byte) ([]byte, error)
}

type Lz4CompressionAlgo struct{}

func Lz4() CompressionAlgorithm {
	return &Lz4CompressionAlgo{}
}

func (c *Lz4CompressionAlgo) Compress(data []byte) ([]byte, error) {
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
}

func (c *Lz4CompressionAlgo) Decompress(data []byte) ([]byte, error) {
	decompressed := make([]byte, 10*len(data))

	n, err := lz4.UncompressBlock(data, decompressed)
	if err != nil {
		return nil, err
	}

	decompressed = decompressed[:n]

	return decompressed, nil
}

func (c *Lz4CompressionAlgo) String() string {
	return "Lz4"
}

type ZstdCompressionAlgo struct {
	encoder *zstd.Encoder
	decoder *zstd.Decoder
}

func Zstd() CompressionAlgorithm {
	encoder, err := zstd.NewWriter(nil, zstd.WithEncoderLevel(zstd.SpeedDefault))
	if err != nil {
		panic(err)
	}

	decoder, err := zstd.NewReader(nil)
	if err != nil {
		panic(err)
	}

	return &ZstdCompressionAlgo{encoder: encoder, decoder: decoder}
}

func (c *ZstdCompressionAlgo) Compress(data []byte) ([]byte, error) {
	var buf bytes.Buffer
	c.encoder.Reset(&buf)

	compressed := c.encoder.EncodeAll(data, nil)

	return compressed, nil
}

func (c *ZstdCompressionAlgo) Decompress(data []byte) ([]byte, error) {
	err := c.decoder.Reset(nil)
	if err != nil {
		return nil, err
	}

	decompressed, err := c.decoder.DecodeAll(data, nil)
	if err != nil {
		return nil, err
	}
	return decompressed, nil
}

func (c *ZstdCompressionAlgo) String() string {
	return "Zstd"
}

type NoCompressionAlgo struct{}

func NoCompression() CompressionAlgorithm {
	return &NoCompressionAlgo{}
}

func (c *NoCompressionAlgo) Compress(data []byte) ([]byte, error) {
	return data, nil
}

func (c *NoCompressionAlgo) Decompress(data []byte) ([]byte, error) {
	return data, nil
}

func (c *NoCompressionAlgo) String() string {
	return ""
}
