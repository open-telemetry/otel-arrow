// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package datagen

import (
	"time"

	"github.com/brianvoe/gofakeit/v6"

	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/logs/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
)

type LogsGenerator struct {
	resourceAttributes    [][]*commonpb.KeyValue
	defaultSchemaUrl      string
	instrumentationScopes []*commonpb.InstrumentationScope
	dataGenerator         *DataGenerator
	generation            int
}

func NewLogsGenerator(resourceAttributes [][]*commonpb.KeyValue, instrumentationScopes []*commonpb.InstrumentationScope) *LogsGenerator {
	return &LogsGenerator{
		resourceAttributes:    resourceAttributes,
		defaultSchemaUrl:      "",
		instrumentationScopes: instrumentationScopes,
		dataGenerator:         NewDataGenerator(uint64(time.Now().UnixNano() / int64(time.Millisecond))),
		generation:            0,
	}
}

func (lg *LogsGenerator) Generate(batchSize int, collectInterval time.Duration) *collogspb.ExportLogsServiceRequest {
	var resourceLogs []*logspb.ResourceLogs

	for i := 0; i < batchSize; i++ {
		var logRecords []*logspb.LogRecord

		lg.dataGenerator.AdvanceTime(collectInterval)
		lg.dataGenerator.NextId8Bits()
		lg.dataGenerator.NextId16Bits()

		logRecords = append(logRecords, LogDebugRecord(lg.dataGenerator))
		logRecords = append(logRecords, LogInfoRecord(lg.dataGenerator))
		logRecords = append(logRecords, LogWarnRecord(lg.dataGenerator))
		logRecords = append(logRecords, LogErrorRecord(lg.dataGenerator))

		resourceLogs = append(resourceLogs, &logspb.ResourceLogs{
			Resource: &resourcepb.Resource{
				Attributes:             lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)],
				DroppedAttributesCount: 0,
			},
			SchemaUrl: lg.defaultSchemaUrl,
			ScopeLogs: []*logspb.ScopeLogs{
				{
					Scope:      lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)],
					LogRecords: logRecords,
					SchemaUrl:  "",
				},
			},
		})
	}

	return &collogspb.ExportLogsServiceRequest{
		ResourceLogs: resourceLogs,
	}
}

func LogDebugRecord(dataGenerator *DataGenerator) *logspb.LogRecord {
	return &logspb.LogRecord{
		TimeUnixNano:         dataGenerator.CurrentTime(),
		ObservedTimeUnixNano: dataGenerator.CurrentTime(),
		SeverityNumber:       logspb.SeverityNumber_SEVERITY_NUMBER_DEBUG,
		SeverityText:         "DEBUG",
		Body: &commonpb.AnyValue{
			Value: &commonpb.AnyValue_StringValue{
				StringValue: gofakeit.LoremIpsumSentence(10),
			},
		},
		Attributes:             DefaultAttributes(),
		DroppedAttributesCount: 0,
		Flags:                  0,
		TraceId:                dataGenerator.Id16Bits(),
		SpanId:                 dataGenerator.Id8Bits(),
	}
}

func LogInfoRecord(dataGenerator *DataGenerator) *logspb.LogRecord {
	return &logspb.LogRecord{
		TimeUnixNano:         dataGenerator.CurrentTime(),
		ObservedTimeUnixNano: dataGenerator.CurrentTime(),
		SeverityNumber:       logspb.SeverityNumber_SEVERITY_NUMBER_INFO,
		SeverityText:         "INFO",
		Body: &commonpb.AnyValue{
			Value: &commonpb.AnyValue_StringValue{
				StringValue: gofakeit.LoremIpsumSentence(10),
			},
		},
		Attributes:             DefaultAttributes(),
		DroppedAttributesCount: 0,
		Flags:                  0,
		TraceId:                dataGenerator.Id16Bits(),
		SpanId:                 dataGenerator.Id8Bits(),
	}
}

func LogWarnRecord(dataGenerator *DataGenerator) *logspb.LogRecord {
	return &logspb.LogRecord{
		TimeUnixNano:         dataGenerator.CurrentTime(),
		ObservedTimeUnixNano: dataGenerator.CurrentTime(),
		SeverityNumber:       logspb.SeverityNumber_SEVERITY_NUMBER_WARN,
		SeverityText:         "WARN",
		Body: &commonpb.AnyValue{
			Value: &commonpb.AnyValue_StringValue{
				StringValue: gofakeit.LoremIpsumSentence(10),
			},
		},
		Attributes:             DefaultAttributes(),
		DroppedAttributesCount: 0,
		Flags:                  0,
		TraceId:                dataGenerator.Id16Bits(),
		SpanId:                 dataGenerator.Id8Bits(),
	}
}

func LogErrorRecord(dataGenerator *DataGenerator) *logspb.LogRecord {
	return &logspb.LogRecord{
		TimeUnixNano:         dataGenerator.CurrentTime(),
		ObservedTimeUnixNano: dataGenerator.CurrentTime(),
		SeverityNumber:       logspb.SeverityNumber_SEVERITY_NUMBER_ERROR,
		SeverityText:         "ERROR",
		Body: &commonpb.AnyValue{
			Value: &commonpb.AnyValue_StringValue{
				StringValue: gofakeit.LoremIpsumSentence(10),
			},
		},
		Attributes:             DefaultAttributes(),
		DroppedAttributesCount: 0,
		Flags:                  0,
		TraceId:                dataGenerator.Id16Bits(),
		SpanId:                 dataGenerator.Id8Bits(),
	}
}
