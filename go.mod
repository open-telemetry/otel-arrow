module github.com/open-telemetry/otel-arrow

go 1.21

toolchain go1.21.4

require (
	github.com/HdrHistogram/hdrhistogram-go v1.1.2
	github.com/apache/arrow/go/v14 v14.0.2
	github.com/axiomhq/hyperloglog v0.0.0-20230201085229-3ddf4bad03dc
	github.com/brianvoe/gofakeit/v6 v6.17.0
	github.com/dustin/go-humanize v1.0.1
	github.com/fxamacker/cbor/v2 v2.4.0
	github.com/klauspost/compress v1.17.8
	github.com/olekukonko/tablewriter v0.0.5
	github.com/pierrec/lz4 v2.0.5+incompatible
	github.com/stretchr/testify v1.9.0
	github.com/zeebo/assert v1.3.0
	go.opentelemetry.io/collector/config/configtelemetry v0.98.0
	go.opentelemetry.io/collector/pdata v1.5.0
	go.opentelemetry.io/otel v1.25.0
	go.opentelemetry.io/otel/metric v1.25.0
	go.uber.org/mock v0.4.0
	golang.org/x/exp v0.0.0-20231110203233-9a3e6036ecaa
	google.golang.org/grpc v1.63.2
	google.golang.org/protobuf v1.33.0
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/dgryski/go-metro v0.0.0-20180109044635-280f6062b5bc // indirect
	github.com/go-logr/logr v1.4.1 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/goccy/go-json v0.10.2 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/google/flatbuffers v23.5.26+incompatible // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/klauspost/cpuid/v2 v2.2.5 // indirect
	github.com/mattn/go-runewidth v0.0.9 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/pierrec/lz4/v4 v4.1.18 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/rogpeppe/go-internal v1.11.0 // indirect
	github.com/x448/float16 v0.8.4 // indirect
	github.com/zeebo/xxh3 v1.0.2 // indirect
	go.opentelemetry.io/otel/trace v1.25.0 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/mod v0.14.0 // indirect
	golang.org/x/net v0.24.0 // indirect
	golang.org/x/sys v0.19.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	golang.org/x/tools v0.15.0 // indirect
	golang.org/x/xerrors v0.0.0-20231012003039-104605ab7028 // indirect
	gonum.org/v1/gonum v0.15.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240401170217-c3f982113cda // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)
replace github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol => ./collector/cmd/otelarrowcol
replace github.com/open-telemetry/otel-arrow/collector/connector/validationconnector => ./collector/connector/validationconnector
replace github.com/open-telemetry/otel-arrow/collector/examples/printer => ./collector/examples/printer
replace github.com/open-telemetry/otel-arrow/collector/exporter/fileexporter => ./collector/exporter/fileexporter
replace github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter => ./collector/exporter/otelarrowexporter
replace github.com/open-telemetry/otel-arrow/collector => ./collector
replace github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor => ./collector/processor/concurrentbatchprocessor
replace github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor => ./collector/processor/experimentprocessor
replace github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor => ./collector/processor/obfuscationprocessor
replace github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver => ./collector/receiver/filereceiver
replace github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver => ./collector/receiver/otelarrowreceiver
