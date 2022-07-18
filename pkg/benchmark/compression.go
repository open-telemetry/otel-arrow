package benchmark

type CompressionAlgorithm uint8

const (
	None CompressionAlgorithm = iota
	Lz4
	Zstd
	Zlib
)

//go:generate stringer -type=CompressionAlgorithm

func Compress(algorithm CompressionAlgorithm, data []byte) ([]byte, error) {
	//switch algorithm {
	//case None:
	//	return data, nil
	//case Lz4:
	//	return lz4.Compress(data)
	//case Zstd:
	//	return zstd.Compress(data)
	//case Zlib:
	//	return zlib.Compress(data)
	//default:
	//	return nil, fmt.Errorf("Unknown compression algorithm: %v", algorithm)
	//}

	println("ToDo implement Compress function")
	return data, nil
}

func Decompress(algorithm CompressionAlgorithm, data []byte) ([]byte, error) {
	//switch algorithm {
	//case None:
	//	return data, nil
	//case Lz4:
	//	return lz4.Decompress(data)
	//case Zstd:
	//	return zstd.Decompress(data)
	//case Zlib:
	//	return zlib.Decompress(data)
	//default:
	//	return nil, fmt.Errorf("Unknown decompression algorithm: %v", algorithm)
	//}

	println("ToDo implement Decompress function")
	return data, nil
}
