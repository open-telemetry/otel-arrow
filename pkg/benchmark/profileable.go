package benchmark

type ProfileableProtocol interface {
	Name() string
	Tags() []string
	DatasetSize() uint64
	CompressionAlgorithm() CompressionAlgorithm
	InitBatchSize(batchSize uint64)
	PrepareBatch(startAt, size uint64)
	CreateBatch(startAt, size uint64)
	Process() string
	Serialize() ([][]byte, error)
	Deserialize(buffers [][]byte)
	Clear()
	ShowStats()
}
