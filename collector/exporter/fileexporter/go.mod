module github.com/open-telemetry/otel-arrow/collector/exporter/fileexporter

go 1.21

toolchain go1.21.4

require (
	github.com/klauspost/compress v1.17.7
	github.com/open-telemetry/otel-arrow/collector v0.18.0
	github.com/stretchr/testify v1.8.4
	go.opentelemetry.io/collector/component v0.96.0
	go.opentelemetry.io/collector/config/configretry v0.96.0
	go.opentelemetry.io/collector/confmap v0.96.0
	go.opentelemetry.io/collector/consumer v0.96.0
	go.opentelemetry.io/collector/exporter v0.96.0
	go.opentelemetry.io/collector/pdata v1.3.0
	go.uber.org/multierr v1.11.0
	go.uber.org/zap v1.27.0
	gopkg.in/natefinch/lumberjack.v2 v2.2.1
)

require (
	github.com/beorn7/perks v1.0.1 // indirect
	github.com/cenkalti/backoff/v4 v4.2.1 // indirect
	github.com/cespare/xxhash/v2 v2.2.0 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/go-logr/logr v1.4.1 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/go-viper/mapstructure/v2 v2.0.0-alpha.1 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/protobuf v1.5.3 // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/knadh/koanf/maps v0.1.1 // indirect
	github.com/knadh/koanf/providers/confmap v0.1.0 // indirect
	github.com/knadh/koanf/v2 v2.1.0 // indirect
	github.com/mitchellh/copystructure v1.2.0 // indirect
	github.com/mitchellh/reflectwalk v1.0.2 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/prometheus/client_golang v1.19.0 // indirect
	github.com/prometheus/client_model v0.6.0 // indirect
	github.com/prometheus/common v0.48.0 // indirect
	github.com/prometheus/procfs v0.12.0 // indirect
	go.opentelemetry.io/collector v0.96.0 // indirect
	go.opentelemetry.io/collector/config/configtelemetry v0.96.0 // indirect
	go.opentelemetry.io/collector/extension v0.96.0 // indirect
	go.opentelemetry.io/collector/receiver v0.96.0 // indirect
	go.opentelemetry.io/otel v1.24.0 // indirect
	go.opentelemetry.io/otel/exporters/prometheus v0.46.0 // indirect
	go.opentelemetry.io/otel/metric v1.24.0 // indirect
	go.opentelemetry.io/otel/sdk v1.24.0 // indirect
	go.opentelemetry.io/otel/sdk/metric v1.24.0 // indirect
	go.opentelemetry.io/otel/trace v1.24.0 // indirect
	golang.org/x/net v0.21.0 // indirect
	golang.org/x/sys v0.17.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240221002015-b0ce06bbee7c // indirect
	google.golang.org/grpc v1.62.0 // indirect
	google.golang.org/protobuf v1.32.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol => ../../../collector/cmd/otelarrowcol

replace github.com/open-telemetry/otel-arrow/collector/connector/validationconnector => ../../../collector/connector/validationconnector

replace github.com/open-telemetry/otel-arrow/collector/examples/printer => ../../../collector/examples/printer

replace github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter => ../../../collector/exporter/otelarrowexporter

replace github.com/open-telemetry/otel-arrow/collector => ../../../collector

replace github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor => ../../../collector/processor/concurrentbatchprocessor

replace github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor => ../../../collector/processor/experimentprocessor

replace github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor => ../../../collector/processor/obfuscationprocessor

replace github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver => ../../../collector/receiver/filereceiver

replace github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver => ../../../collector/receiver/otelarrowreceiver

replace github.com/open-telemetry/otel-arrow => ../../..
