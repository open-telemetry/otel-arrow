package benchmark

type CompressionAlgorithm uint8

const (
	None CompressionAlgorithm = iota
	Lz4
	Zstd
	Zlib
)
