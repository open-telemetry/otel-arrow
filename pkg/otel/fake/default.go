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

package fake

import (
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
)

func DefaultAttributes() []*commonpb.KeyValue {
	return []*commonpb.KeyValue{
		{
			Key:   "hostname",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "localhost"}},
		},
		{
			Key:   "up",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: true}},
		},
		{
			Key:   "status",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: 200}},
		},
		{
			Key:   "version",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{DoubleValue: 1.0}},
		},
		{
			Key: "tags_array",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_ArrayValue{ArrayValue: &commonpb.ArrayValue{
				Values: []*commonpb.AnyValue{
					{Value: &commonpb.AnyValue_StringValue{StringValue: "tag1"}},
					{Value: &commonpb.AnyValue_StringValue{StringValue: "tag2"}},
				},
			}}},
		},
		{
			Key: "tags_kv_list",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{
				KvlistValue: &commonpb.KeyValueList{
					Values: []*commonpb.KeyValue{
						{
							Key:   "attribute_1",
							Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "abc"}},
						},
						{
							Key:   "attribute_2",
							Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: 192}},
						},
					},
				},
			}},
		},
		{
			Key: "version_binary",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{
				BytesValue: []byte("binary"),
			}},
		},
	}
}

func DefaultResourceAttributes() []*commonpb.KeyValue {
	return []*commonpb.KeyValue{
		{
			Key:   "hostname",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "localhost"}},
		},
		{
			Key:   "ip",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "192.168.0.1"}},
		},
		{
			Key:   "up",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: true}},
		},
		{
			Key:   "status",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: 200}},
		},
		{
			Key:   "version",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{DoubleValue: 1.0}},
		},
	}
}

func DefaultInstrumentationScope() *commonpb.InstrumentationScope {
	return &commonpb.InstrumentationScope{
		Name:    "fake_generator",
		Version: "1.0.0",
	}
}
