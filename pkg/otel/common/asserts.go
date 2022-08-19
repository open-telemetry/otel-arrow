/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package common

import (
	"bytes"
	"fmt"
	"sort"
	"strconv"

	colmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
)

func MetricsRequestAssertEq(req1 *colmetrics.ExportMetricsServiceRequest, req2 *colmetrics.ExportMetricsServiceRequest) {
	if req1 == nil && req2 == nil {
		return
	}
	if req1 == nil {
		panic("The first ExportMetricsServiceRequest is nil")
	}
	if req2 == nil {
		panic("The second ExportMetricsServiceResponse is nil")
	}
	if len(req1.ResourceMetrics) != len(req2.ResourceMetrics) {
		panic(fmt.Sprintf("resource metrics length mismatch (len(req1)=%d, len(r2)=%d)", len(req1.ResourceMetrics), len(req2.ResourceMetrics)))
	}
	for i := range req1.ResourceMetrics {
		ResourceMetricsAssertEq(req1.ResourceMetrics[i], req2.ResourceMetrics[i])
	}
}

func ResourceMetricsAssertEq(rm1 *metricspb.ResourceMetrics, rm2 *metricspb.ResourceMetrics) {
	ResourceAssertEq(rm1.Resource, rm2.Resource, "resource")
	ScopeMetricsSliceAssertEq(rm1.ScopeMetrics, rm2.ScopeMetrics, "resourceMetrics.scopeMetrics")
	if rm1.SchemaUrl != rm2.SchemaUrl {
		panic("resource metrics schema url mismatch (r1.SchemaUrl=" + rm1.SchemaUrl + ", rm2.SchemaUrl=" + rm2.SchemaUrl + ")")
	}
}

func ResourceAssertEq(r1 *resourcepb.Resource, r2 *resourcepb.Resource, context string) {
	if r1 == nil && r2 == nil {
		return
	}
	if r1 == nil {
		panic("The first Resource is nil")
	}
	if r2 == nil {
		panic("The second Resource is nil")
	}
	if r1.DroppedAttributesCount != r2.DroppedAttributesCount {
		panic(fmt.Sprintf("resource dropped attributes count mismatch (r1.DroppedAttributesCount=%d, r2.DroppedAttributesCount=%d)", r1.DroppedAttributesCount, r2.DroppedAttributesCount))
	}
	AttributesAssertEq(r1.Attributes, r2.Attributes, context+".attributes")
}

func AttributesAssertEq(attrs1 []*commonpb.KeyValue, attrs2 []*commonpb.KeyValue, context string) {
	if len(attrs1) != len(attrs2) {
		panic(fmt.Sprintf("attributes length mismatch (len(attrs1)=%d, len(attrs2)=%d)", len(attrs1), len(attrs2)))
	}
	sort.Slice(attrs1, func(i, j int) bool {
		return attrs1[i].Key < attrs1[j].Key
	})
	sort.Slice(attrs2, func(i, j int) bool {
		return attrs2[i].Key < attrs2[j].Key
	})
	for i := range attrs1 {
		KeyValueAssertEq(attrs1[i], attrs2[i], context+".attributes")
	}
}

func KeyValueAssertEq(kv1 *commonpb.KeyValue, kv2 *commonpb.KeyValue, context string) {
	if kv1 == nil && kv2 == nil {
		return
	}
	if kv1 == nil {
		panic("The first KeyValue is nil (context: " + context + ")")
	}
	if kv2 == nil {
		panic("The second KeyValue is nil (context: " + context + ")")
	}
	if kv1.Key != kv2.Key {
		panic("key mismatch (kv1.Key=" + kv1.Key + ", kv2.Key=" + kv2.Key + ", context: " + context + ")")
	}
	AnyValueAssertEq(kv1.Value, kv2.Value, context)
}

func AnyValueAssertEq(v1 *commonpb.AnyValue, v2 *commonpb.AnyValue, context string) {
	if v1 == nil && v2 == nil {
		return
	}
	if v1 == nil {
		panic("The first AnyValue is nil (context: " + context + ")")
	}
	if v2 == nil {
		panic("The second AnyValue is nil (context: " + context + ")")
	}
	switch v := v1.Value.(type) {
	case *commonpb.AnyValue_IntValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_IntValue)
		if !ok {
			panic("type mismatch (v1.Value=int, v2.Value=not int, context: " + context + ")")
		}
		if v2.IntValue != v.IntValue {
			panic("int value mismatch (v1.IntValue=" + fmt.Sprint(v.IntValue) + ", v2.IntValue=" + fmt.Sprint(v2.IntValue) + ", context: " + context + ")")
		}
	case *commonpb.AnyValue_DoubleValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_DoubleValue)
		if !ok {
			panic("type mismatch (v1.Value=double, v2.Value=not double, context: " + context + ")")
		}
		if v2.DoubleValue != v.DoubleValue {
			panic("double value mismatch (v1.DoubleValue=" + fmt.Sprint(v.DoubleValue) + ", v2.DoubleValue=" + fmt.Sprint(v2.DoubleValue) + ", context: " + context + ")")
		}
	case *commonpb.AnyValue_StringValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_StringValue)
		if !ok {
			panic("type mismatch (v1.Value=string, v2.Value=not string, context: " + context + ")")
		}
		if v2.StringValue != v.StringValue {
			panic("string value mismatch (v1.StringValue=" + v.StringValue + ", v2.StringValue=" + v2.StringValue + ", context: " + context + ")")
		}
	case *commonpb.AnyValue_BoolValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_BoolValue)
		if !ok {
			panic("type mismatch (v1.Value=bool, v2.Value=not bool, context: " + context + ")")
		}
		if v2.BoolValue != v.BoolValue {
			panic("bool value mismatch (v1.BoolValue=" + fmt.Sprint(v.BoolValue) + ", v2.BoolValue=" + fmt.Sprint(v2.BoolValue) + ", context: " + context + ")")
		}
	case *commonpb.AnyValue_BytesValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_BytesValue)
		if !ok {
			panic("type mismatch (v1.Value=bytes, v2.Value=not bytes, context: " + context + ")")
		}
		if !bytes.Equal(v2.BytesValue, v.BytesValue) {
			panic("bytes value mismatch (v1.BytesValue=" + fmt.Sprint(v.BytesValue) + ", v2.BytesValue=" + fmt.Sprint(v2.BytesValue) + ", context: " + context + ")")
		}
	case *commonpb.AnyValue_ArrayValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_ArrayValue)
		if !ok {
			panic("type mismatch (v1.Value=array, v2.Value=not array, context: " + context + ")")
		}
		ArrayValueAssertEq(v.ArrayValue, v2.ArrayValue, context)
	case *commonpb.AnyValue_KvlistValue:
		v2, ok := v2.Value.(*commonpb.AnyValue_KvlistValue)
		if !ok {
			panic("type mismatch (v1.Value=kvlist, v2.Value=not kvlist, context: " + context + ")")
		}
		KvlistValueAssertEq(v.KvlistValue, v2.KvlistValue, context)
	default:
		panic("unexpected AnyValue type (context: " + context + ")")
	}
}

func KvlistValueAssertEq(v1 *commonpb.KeyValueList, v2 *commonpb.KeyValueList, context string) {
	if v1 == nil && v2 == nil {
		return
	}
	if v1 == nil {
		panic("The first KvlistValue is nil")
	}
	if v2 == nil {
		panic("The second KvlistValue is nil")
	}
	if len(v1.Values) != len(v2.Values) {
		panic(fmt.Sprintf("kvlist length mismatch (len(v1)=%d, len(v2)=%d)", len(v1.Values), len(v2.Values)))
	}
	sort.Slice(v1.Values, func(i, j int) bool {
		return v1.Values[i].Key < v1.Values[j].Key
	})
	sort.Slice(v2.Values, func(i, j int) bool {
		return v2.Values[i].Key < v2.Values[j].Key
	})
	for i := range v1.Values {
		KeyValueAssertEq(v1.Values[i], v2.Values[i], context+"."+v1.Values[1].Key+"["+strconv.Itoa(i)+"]")
	}
}

func ArrayValueAssertEq(v1 *commonpb.ArrayValue, v2 *commonpb.ArrayValue, context string) {
	if v1 == nil && v2 == nil {
		return
	}
	if v1 == nil {
		panic("The first ArrayValue is nil (context: " + context + ")")
	}
	if v2 == nil {
		panic("The second ArrayValue is nil (context: " + context + ")")
	}
	if len(v1.Values) != len(v2.Values) {
		panic(fmt.Sprintf("array value length mismatch (len(v1)=%d, len(v2)=%d)", len(v1.Values), len(v2.Values)))
	}
	for i := range v1.Values {
		AnyValueAssertEq(v1.Values[i], v2.Values[i], context+"["+strconv.Itoa(i)+"]")
	}
}

func ScopeMetricsSliceAssertEq(sm1 []*metricspb.ScopeMetrics, sm2 []*metricspb.ScopeMetrics, context string) {
	if sm1 == nil && sm2 == nil {
		return
	}
	if sm1 == nil {
		panic("The first scope metrics is nil")
	}
	if sm2 == nil {
		panic("The second scope metrics is nil")
	}
	if len(sm1) != len(sm2) {
		panic(fmt.Sprintf("scope metrics length mismatch (len(sm1)=%d, len(sm2)=%d, context=%s)", len(sm1), len(sm2), context))
	}
	for i := range sm1 {
		ScopeMetricsAssertEq(sm1[i], sm2[i], context+"["+strconv.Itoa(i)+"]")
	}
}

func ScopeMetricsAssertEq(sm1 *metricspb.ScopeMetrics, sm2 *metricspb.ScopeMetrics, context string) {
	if sm1 == nil && sm2 == nil {
		return
	}
	if sm1 == nil {
		panic("The first scope metrics is nil (context: " + context + ")")
	}
	if sm2 == nil {
		panic("The second scope metrics is nil (context: " + context + ")")
	}

	InstrumentationScopeAssertEq(sm1.Scope, sm2.Scope, context+".Scope")
	MetricsAssertEq(sm1.Metrics, sm2.Metrics, context+".Metrics")

	if sm1.SchemaUrl != sm2.SchemaUrl {
		panic("schema url mismatch (sm1.SchemaUrl=" + sm1.SchemaUrl + ", sm2.SchemaUrl=" + sm2.SchemaUrl + ", context: " + context + ")")
	}
}

func MetricsAssertEq(metrics1 []*metricspb.Metric, metrics2 []*metricspb.Metric, context string) {
	if metrics1 == nil && metrics2 == nil {
		return
	}
	if metrics1 == nil {
		panic("The first metrics is nil (context: " + context + ")")
	}
	if metrics2 == nil {
		panic("The second metrics is nil (context: " + context + ")")
	}
	if len(metrics1) != len(metrics2) {
		panic(fmt.Sprintf("metrics length mismatch (len(metrics1)=%d, len(metrics2)=%d, context=%s)", len(metrics1), len(metrics2), context))
	}
	for i := range metrics1 {
		MetricAssertEq(metrics1[i], metrics2[i], context+"["+strconv.Itoa(i)+"]")
	}
}

func MetricAssertEq(m1 *metricspb.Metric, m2 *metricspb.Metric, context string) {
	if m1 == nil && m2 == nil {
		return
	}
	if m1 == nil {
		panic("The first metric is nil (context: " + context + ")")
	}
	if m2 == nil {
		panic("The second metric is nil (context: " + context + ")")
	}
	if m1.Name != m2.Name {
		panic("metric name mismatch (m1.Name=" + m1.Name + ", m2.Name=" + m2.Name + ", context: " + context + ")")
	}
	if m1.Description != m2.Description {
		panic("metric description mismatch (m1.Description=" + m1.Description + ", m2.Description=" + m2.Description + ", context: " + context + ")")
	}
	if m1.Unit != m2.Unit {
		panic("metric unit mismatch (m1.Unit=" + m1.Unit + ", m2.Unit=" + m2.Unit + ", context: " + context + ")")
	}
	MetricDataAssertEq(m1.Data, m2.Data, context+".Data")
}

func MetricDataAssertEq(d1 interface{}, d2 interface{}, context string) {
	// ToDo
}

func InstrumentationScopeAssertEq(sc1 *commonpb.InstrumentationScope, sc2 *commonpb.InstrumentationScope, context string) {
	if sc1 == nil && sc2 == nil {
		return
	}
	if sc1 == nil {
		panic("The first instrumentation scope is nil (context: " + context + ")")
	}
	if sc2 == nil {
		panic("The second instrumentation scope is nil (context: " + context + ")")
	}
	if sc1.Name != sc2.Name {
		panic("instrumentation scope name mismatch (sc1.Name=" + sc1.Name + ", sc2.Name=" + sc2.Name + ", context: " + context + ")")
	}
	if sc1.Version != sc2.Version {
		panic("instrumentation scope version mismatch (sc1.Version=" + sc1.Version + ", sc2.Version=" + sc2.Version + ", context: " + context + ")")
	}
	AttributesAssertEq(sc1.Attributes, sc2.Attributes, context+".attributes")
	if sc1.DroppedAttributesCount != sc2.DroppedAttributesCount {
		panic(fmt.Sprintf("instrumentation scope dropped attributes count mismatch (sc1.DroppedAttributesCount=%d, sc2.DroppedAttributesCount=%d, context: %s)", sc1.DroppedAttributesCount, sc2.DroppedAttributesCount, context))
	}
}
