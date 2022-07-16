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

package metrics

import (
	"fmt"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/otel/metrics"
	"testing"
)

func TestDataPointSig(t *testing.T) {
	t.Parallel()

	ndp := v1.NumberDataPoint{
		StartTimeUnixNano: 1,
		TimeUnixNano:      2,
		Attributes: []*commonpb.KeyValue{
			{
				Key: "k4",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{
					DoubleValue: 1.0,
				}},
			},
			{
				Key: "k1",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{
					IntValue: 2,
				}},
			},
			{
				Key: "k3",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{
					BoolValue: false,
				}},
			},
			{
				Key: "k5",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{
					StringValue: "bla",
				}},
			},
			{
				Key: "k2",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{
					BytesValue: []byte{1, 2, 3},
				}},
			},
			{
				Key: "k8",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{
					KvlistValue: &commonpb.KeyValueList{
						Values: []*commonpb.KeyValue{
							{
								Key: "k4",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{
									DoubleValue: 1.0,
								}},
							},
							{
								Key: "k1",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{
									IntValue: 2,
								}},
							},
						},
					},
				}},
			},
			{
				Key: "k7",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{
					KvlistValue: &commonpb.KeyValueList{
						Values: []*commonpb.KeyValue{
							{
								Key: "k4",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{
									DoubleValue: 1.0,
								}},
							},
							{
								Key: "k1",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{
									IntValue: 2,
								}},
							},
						},
					},
				}},
			},
		},
	}

	sig := metrics.DataPointSig(&ndp, "k5")
	expected := "[1 0 0 0 0 0 0 0 2 0 0 0 0 0 0 0 107 49 2 0 0 0 0 0 0 0 107 50 1 2 3 107 51 0 107 52 0 0 0 0 0 0 240 63 107 55 107 49 2 0 0 0 0 0 0 0 107 52 0 0 0 0 0 0 240 63 107 56 107 49 2 0 0 0 0 0 0 0 107 52 0 0 0 0 0 0 240 63]"
	observed := fmt.Sprintf("%v", sig)
	if expected != observed {
		t.Errorf("expected %v, observed %v", expected, observed)
	}
}
