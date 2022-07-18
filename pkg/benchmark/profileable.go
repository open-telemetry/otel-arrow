package benchmark

type ProfileableSystem interface {
	Name() string
	Tags() []string
	DatasetSize() int
	CompressionAlgorithm() CompressionAlgorithm
	InitBatchSize(batchSize int)
	PrepareBatch(startAt, size int)
	CreateBatch(startAt, size int)
	Process() string
	Serialize() ([][]byte, error)
	Deserialize(buffers [][]byte)
	Clear()
	ShowStats()
}
