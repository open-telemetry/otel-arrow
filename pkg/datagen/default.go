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
	"golang.org/x/exp/rand"

	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
)

var HOSTNAMES = []string{"host1.mydomain.com", "host2.org", "host3.thedomain.edu", "host4.gov", "host5.retailer.com"}
var UPS = []bool{true, false}
var STATUS = []int64{200, 300, 400, 404, 500, 503}
var VERSIONS = []string{"1.0.0", "1.0.2", "2.0", "1.9.9"}
var STATES = []string{"running", "ready", "maintenance", "degraded", "unavailable", "unknown"}
var TRACE_IDS = []string{"trace1", "trace2", "trace3", "trace4", "trace5"}

func DefaultAttributes() []*commonpb.KeyValue {
	attributes := []*commonpb.KeyValue{
		{
			Key:   "hostname",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: HOSTNAMES[rand.Intn(len(HOSTNAMES))]}},
		},
		{
			Key:   "up",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: UPS[rand.Intn(len(UPS))]}},
		},
		{
			Key:   "status",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: STATUS[rand.Intn(len(STATUS))]}},
		},
		{
			Key:   "version",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: VERSIONS[rand.Intn(len(VERSIONS))]}},
		},
		// ToDo reintroduce tags_arrays once list are fully supported
		//{
		//	Key: "tags_array",
		//	Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_ArrayValue{ArrayValue: &commonpb.ArrayValue{
		//		Values: []*commonpb.AnyValue{
		//			{Value: &commonpb.AnyValue_StringValue{StringValue: "tag1"}},
		//			{Value: &commonpb.AnyValue_StringValue{StringValue: "tag2"}},
		//		},
		//	}}},
		//},
		{
			Key: "tags_kv_list",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{
				KvlistValue: &commonpb.KeyValueList{
					Values: []*commonpb.KeyValue{
						{
							Key:   "state",
							Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: STATES[rand.Intn(len(STATES))]}},
						},
						{
							Key:   "duration",
							Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: int64(rand.Intn(100))}},
						},
					},
				},
			}},
		},
		{
			Key: "trace_id",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{
				BytesValue: []byte(TRACE_IDS[rand.Intn(len(TRACE_IDS))]),
			}},
		},
	}

	rand.Shuffle(len(attributes), func(i, j int) {
		attributes[i], attributes[j] = attributes[j], attributes[i]
	})
	return attributes
}

func DefaultResourceAttributes() [][]*commonpb.KeyValue {
	return [][]*commonpb.KeyValue{
		{
			{
				Key:   "hostname",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "host1.mydomain.com"}},
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
		},
		{
			{
				Key:   "hostname",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "host2.mydomain.com"}},
			},
			{
				Key:   "ip",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "192.168.0.2"}},
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
		},
		{
			{
				Key:   "hostname",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "host3.mydomain.com"}},
			},
			{
				Key:   "ip",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "192.168.0.3"}},
			},
			{
				Key:   "up",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: false}},
			},
			{
				Key:   "status",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: 500}},
			},
			{
				Key:   "version",
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{DoubleValue: 1.5}},
			},
		}}
}

func DefaultInstrumentationScopes() []*commonpb.InstrumentationScope {
	return []*commonpb.InstrumentationScope{
		{
			Name:    "fake_generator",
			Version: "1.0.0",
		},
		{
			Name:    "fake_generator",
			Version: "1.0.1",
		},
	}
}

func DefaultSpanEventAttributes() []*commonpb.KeyValue {
	return []*commonpb.KeyValue{
		{
			Key:   "hostname",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: HOSTNAMES[rand.Intn(len(HOSTNAMES))]}},
		},
		{
			Key:   "version",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: VERSIONS[rand.Intn(len(VERSIONS))]}},
		},
		{
			Key:   "up",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: UPS[rand.Intn(len(UPS))]}},
		},
		{
			Key:   "status",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: STATUS[rand.Intn(len(STATUS))]}},
		},
	}
}

func DefaultSpanLinkAttributes() []*commonpb.KeyValue {
	return []*commonpb.KeyValue{
		{
			Key:   "hostname",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: HOSTNAMES[rand.Intn(len(HOSTNAMES))]}},
		},
		{
			Key:   "up",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: UPS[rand.Intn(len(UPS))]}},
		},
		{
			Key:   "status",
			Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: STATUS[rand.Intn(len(STATUS))]}},
		},
	}
}
