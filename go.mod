// Deprecated: Terminal version of github.com/open-telemetry/otel-arrow, use github.com/open-telemetry/otel-arrow/go instead.
module github.com/open-telemetry/otel-arrow

go 1.23.0

require (
	github.com/HdrHistogram/hdrhistogram-go v1.1.2
	github.com/apache/arrow-go/v18 v18.2.0
	github.com/axiomhq/hyperloglog v0.0.0-20230201085229-3ddf4bad03dc
	github.com/brianvoe/gofakeit/v6 v6.17.0
	github.com/dustin/go-humanize v1.0.1
	github.com/fxamacker/cbor/v2 v2.4.0
	github.com/klauspost/compress v1.18.0
	github.com/olekukonko/tablewriter v0.0.5
	github.com/pierrec/lz4 v2.0.5+incompatible
	github.com/stretchr/testify v1.10.0
	github.com/zeebo/assert v1.3.1
	go.opentelemetry.io/collector/pdata v1.33.0
	go.opentelemetry.io/otel v1.34.0
	go.opentelemetry.io/otel/metric v1.34.0
	go.uber.org/mock v0.4.0
	golang.org/x/exp v0.0.0-20250531010427-b6e5de432a8b
	google.golang.org/grpc v1.72.2
	google.golang.org/protobuf v1.36.6
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/dgryski/go-metro v0.0.0-20180109044635-280f6062b5bc // indirect
	github.com/go-logr/logr v1.4.2 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/goccy/go-json v0.10.5 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/google/flatbuffers v25.2.10+incompatible // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/klauspost/cpuid/v2 v2.2.10 // indirect
	github.com/mattn/go-runewidth v0.0.16 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/pierrec/lz4/v4 v4.1.22 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/rivo/uniseg v0.4.4 // indirect
	github.com/x448/float16 v0.8.4 // indirect
	github.com/zeebo/xxh3 v1.0.2 // indirect
	go.opentelemetry.io/auto/sdk v1.1.0 // indirect
	go.opentelemetry.io/otel/trace v1.34.0 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/mod v0.24.0 // indirect
	golang.org/x/net v0.40.0 // indirect
	golang.org/x/sync v0.14.0 // indirect
	golang.org/x/sys v0.33.0 // indirect
	golang.org/x/text v0.25.0 // indirect
	golang.org/x/tools v0.33.0 // indirect
	golang.org/x/xerrors v0.0.0-20240903120638-7835f813f4da // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250218202821-56aae31c358a // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

retract (
	v0.37.0 // This version contained an upgrade to Go 1.24 which was incompatible with primary consumers.
	v0.36.0 // This version was released in the midst of a repository restructure causing incompatilibity between module name and source location.
)
