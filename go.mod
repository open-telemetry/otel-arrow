module otel-arrow-adapter

go 1.18

require (
	github.com/apache/arrow/go/v9 v9.0.0-00010101000000-000000000000
	github.com/brianvoe/gofakeit/v6 v6.17.0
	github.com/davecgh/go-spew v1.1.1
	github.com/google/go-cmp v0.5.8
	github.com/klauspost/compress v1.15.8
	github.com/olekukonko/tablewriter v0.0.5
	github.com/pierrec/lz4 v2.0.5+incompatible
	golang.org/x/exp v0.0.0-20211216164055-b2b84827b756
	google.golang.org/protobuf v1.28.0
)

require (
	github.com/andybalholm/brotli v1.0.4 // indirect
	github.com/apache/thrift v0.15.0 // indirect
	github.com/goccy/go-json v0.9.6 // indirect
	github.com/golang/snappy v0.0.4 // indirect
	github.com/google/flatbuffers v2.0.5+incompatible // indirect
	github.com/klauspost/asmfmt v1.3.1 // indirect
	github.com/klauspost/cpuid/v2 v2.0.9 // indirect
	github.com/mattn/go-runewidth v0.0.9 // indirect
	github.com/minio/asm2plan9s v0.0.0-20200509001527-cdd76441f9d8 // indirect
	github.com/minio/c2goasm v0.0.0-20190812172519-36a3d3bbc4f3 // indirect
	github.com/pierrec/lz4/v4 v4.1.15 // indirect
	github.com/zeebo/xxh3 v1.0.1 // indirect
	golang.org/x/mod v0.6.0-dev.0.20220106191415-9b9b3d81d5e3 // indirect
	golang.org/x/sys v0.0.0-20220412211240-33da011f77ad // indirect
	golang.org/x/tools v0.1.10 // indirect
	golang.org/x/xerrors v0.0.0-20220411194840-2f41105eb62f // indirect
)

replace github.com/apache/arrow/go/v9 => github.com/lquerel/arrow/go/v9 v9.0.0-20220708002903-441b5440ea47
