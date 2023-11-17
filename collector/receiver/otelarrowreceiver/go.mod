module github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver

go 1.21

require (
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/klauspost/compress v1.17.2
	github.com/open-telemetry/otel-arrow v0.10.0
	github.com/open-telemetry/otel-arrow/collector v0.10.0
	github.com/stretchr/testify v1.8.4
	go.opentelemetry.io/collector v0.89.0
	go.opentelemetry.io/collector/component v0.89.0
	go.opentelemetry.io/collector/config/configauth v0.89.0
	go.opentelemetry.io/collector/config/configgrpc v0.89.0
	go.opentelemetry.io/collector/config/confighttp v0.89.0
	go.opentelemetry.io/collector/config/confignet v0.89.0
	go.opentelemetry.io/collector/config/configtelemetry v0.89.0
	go.opentelemetry.io/collector/config/configtls v0.89.0
	go.opentelemetry.io/collector/confmap v0.89.0
	go.opentelemetry.io/collector/consumer v0.89.0
	go.opentelemetry.io/collector/extension/auth v0.89.0
	go.opentelemetry.io/collector/pdata v1.0.0-rcv0018
	go.opentelemetry.io/collector/receiver v0.89.0
	go.opentelemetry.io/collector/semconv v0.89.0
	go.uber.org/multierr v1.11.0
	go.uber.org/zap v1.26.0
	golang.org/x/net v0.17.0
	google.golang.org/genproto/googleapis/rpc v0.0.0-20231012201019-e917dd12ba7a
	google.golang.org/grpc v1.59.0
	google.golang.org/protobuf v1.31.0
)

require (
	cloud.google.com/go/compute/metadata v0.2.4-0.20230617002413-005d2dfb6b68 // indirect
	contrib.go.opencensus.io/exporter/prometheus v0.4.2 // indirect
	github.com/HdrHistogram/hdrhistogram-go v1.1.2 // indirect
	github.com/andybalholm/brotli v1.0.5 // indirect
	github.com/apache/arrow/go/v12 v12.0.1 // indirect
	github.com/apache/thrift v0.16.0 // indirect
	github.com/axiomhq/hyperloglog v0.0.0-20230201085229-3ddf4bad03dc // indirect
	github.com/beorn7/perks v1.0.1 // indirect
	github.com/cespare/xxhash/v2 v2.2.0 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/dgryski/go-metro v0.0.0-20180109044635-280f6062b5bc // indirect
	github.com/felixge/httpsnoop v1.0.3 // indirect
	github.com/fsnotify/fsnotify v1.7.0 // indirect
	github.com/fxamacker/cbor/v2 v2.4.0 // indirect
	github.com/go-kit/log v0.2.1 // indirect
	github.com/go-logfmt/logfmt v0.5.1 // indirect
	github.com/go-logr/logr v1.3.0 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/goccy/go-json v0.9.11 // indirect
	github.com/golang/groupcache v0.0.0-20210331224755-41bb18bfe9da // indirect
	github.com/golang/protobuf v1.5.3 // indirect
	github.com/golang/snappy v0.0.5-0.20220116011046-fa5810519dcb // indirect
	github.com/google/flatbuffers v2.0.8+incompatible // indirect
	github.com/hashicorp/go-version v1.6.0 // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/klauspost/asmfmt v1.3.2 // indirect
	github.com/klauspost/cpuid/v2 v2.0.9 // indirect
	github.com/knadh/koanf/maps v0.1.1 // indirect
	github.com/knadh/koanf/providers/confmap v0.1.0 // indirect
	github.com/knadh/koanf/v2 v2.0.1 // indirect
	github.com/matttproud/golang_protobuf_extensions/v2 v2.0.0 // indirect
	github.com/minio/asm2plan9s v0.0.0-20200509001527-cdd76441f9d8 // indirect
	github.com/minio/c2goasm v0.0.0-20190812172519-36a3d3bbc4f3 // indirect
	github.com/mitchellh/copystructure v1.2.0 // indirect
	github.com/mitchellh/mapstructure v1.5.1-0.20220423185008-bf980b35cac4 // indirect
	github.com/mitchellh/reflectwalk v1.0.2 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/mostynb/go-grpc-compression v1.2.2 // indirect
	github.com/pierrec/lz4/v4 v4.1.18 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/prometheus/client_golang v1.17.0 // indirect
	github.com/prometheus/client_model v0.5.0 // indirect
	github.com/prometheus/common v0.45.0 // indirect
	github.com/prometheus/procfs v0.11.1 // indirect
	github.com/prometheus/statsd_exporter v0.22.7 // indirect
	github.com/rs/cors v1.10.1 // indirect
	github.com/x448/float16 v0.8.4 // indirect
	github.com/zeebo/xxh3 v1.0.2 // indirect
	go.opencensus.io v0.24.0 // indirect
	go.opentelemetry.io/collector/config/configcompression v0.89.0 // indirect
	go.opentelemetry.io/collector/config/configopaque v0.89.0 // indirect
	go.opentelemetry.io/collector/config/internal v0.89.0 // indirect
	go.opentelemetry.io/collector/exporter v0.89.0 // indirect
	go.opentelemetry.io/collector/extension v0.89.0 // indirect
	go.opentelemetry.io/collector/featuregate v1.0.0-rcv0018 // indirect
	go.opentelemetry.io/collector/service v0.89.0 // indirect
	go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc v0.46.0 // indirect
	go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp v0.45.0 // indirect
	go.opentelemetry.io/otel v1.20.0 // indirect
	go.opentelemetry.io/otel/exporters/prometheus v0.43.0 // indirect
	go.opentelemetry.io/otel/metric v1.20.0 // indirect
	go.opentelemetry.io/otel/sdk v1.20.0 // indirect
	go.opentelemetry.io/otel/sdk/metric v1.20.0 // indirect
	go.opentelemetry.io/otel/trace v1.20.0 // indirect
	golang.org/x/mod v0.11.0 // indirect
	golang.org/x/sys v0.14.0 // indirect
	golang.org/x/text v0.13.0 // indirect
	golang.org/x/tools v0.10.0 // indirect
	golang.org/x/xerrors v0.0.0-20220907171357-04be3eba64a2 // indirect
	gopkg.in/yaml.v2 v2.4.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)











replace github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol => ../../../collector/cmd/otelarrowcol
replace github.com/open-telemetry/otel-arrow/collector/connector/validationconnector => ../../../collector/connector/validationconnector
replace github.com/open-telemetry/otel-arrow/collector/examples/printer => ../../../collector/examples/printer
replace github.com/open-telemetry/otel-arrow/collector/exporter/fileexporter => ../../../collector/exporter/fileexporter
replace github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter => ../../../collector/exporter/otelarrowexporter
replace github.com/open-telemetry/otel-arrow/collector => ../../../collector
replace github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor => ../../../collector/processor/concurrentbatchprocessor
replace github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor => ../../../collector/processor/experimentprocessor
replace github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor => ../../../collector/processor/obfuscationprocessor
replace github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver => ../../../collector/receiver/filereceiver
replace github.com/open-telemetry/otel-arrow => ../../..
