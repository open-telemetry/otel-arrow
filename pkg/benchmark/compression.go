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

//go:generate stringer -type=CompressionAlgorithm

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
