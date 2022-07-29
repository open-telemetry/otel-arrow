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
	"fmt"
	"time"

	coltracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
	tracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/trace/v1"
)

type TraceGenerator struct {
	resourceAttributes   []*commonpb.KeyValue
	defaultSchemaUrl     string
	instrumentationScope *commonpb.InstrumentationScope
	dataGenerator        *DataGenerator
}

func NewTraceGenerator(resourceAttributes []*commonpb.KeyValue, instrumentationScope *commonpb.InstrumentationScope) *TraceGenerator {
	return &TraceGenerator{
		resourceAttributes:   resourceAttributes,
		defaultSchemaUrl:     "",
		instrumentationScope: instrumentationScope,
		dataGenerator:        NewDataGenerator(uint64(time.Now().UnixNano() / int64(time.Millisecond))),
	}
}

func (lg *TraceGenerator) Generate(batchSize int, collectInterval time.Duration) *coltracepb.ExportTraceServiceRequest {
	var resourceSpans []*tracepb.ResourceSpans

	for i := 0; i < batchSize; i++ {
		lg.dataGenerator.AdvanceTime(collectInterval)

		resourceSpans = append(resourceSpans, &tracepb.ResourceSpans{
			Resource: &resourcepb.Resource{
				Attributes:             lg.resourceAttributes,
				DroppedAttributesCount: 0,
			},
			SchemaUrl: lg.defaultSchemaUrl,
			ScopeSpans: []*tracepb.ScopeSpans{
				{
					Scope:     lg.instrumentationScope,
					Spans:     Spans(lg.dataGenerator),
					SchemaUrl: "",
				},
			},
		})
	}

	return &coltracepb.ExportTraceServiceRequest{
		ResourceSpans: resourceSpans,
	}
}

func Spans(dataGenerator *DataGenerator) []*tracepb.Span {
	dataGenerator.NextId8Bits()
	dataGenerator.NextId16Bits()

	traceId := dataGenerator.Id16Bits()
	rootSpanId := dataGenerator.Id8Bits()
	rootStartTime := dataGenerator.CurrentTime()
	rootEndTime := dataGenerator.CurrentTime() + 1 + 5

	dataGenerator.AdvanceTime(1)

	dataGenerator.NextId8Bits()
	userAccountSpanId := dataGenerator.Id8Bits()
	userAccountStartTime := dataGenerator.CurrentTime()
	userAccountEndTime := dataGenerator.CurrentTime() + 5

	dataGenerator.NextId8Bits()
	userPreferencesSpanId := dataGenerator.Id8Bits()
	userPreferenceStartTime := dataGenerator.CurrentTime()
	userPreferenceEndTime := dataGenerator.CurrentTime() + 3

	return []*tracepb.Span{
		{
			TraceId:                traceId,
			SpanId:                 rootSpanId,
			Name:                   "GET /user-info",
			StartTimeUnixNano:      rootStartTime,
			EndTimeUnixNano:        rootEndTime,
			Kind:                   tracepb.Span_SPAN_KIND_SERVER,
			Attributes:             DefaultAttributes(),
			DroppedAttributesCount: 0,
			Events:                 events(dataGenerator),
			DroppedEventsCount:     0,
			Links:                  links(dataGenerator),
			DroppedLinksCount:      0,
			Status: &tracepb.Status{
				Code:    tracepb.Status_STATUS_CODE_OK,
				Message: "OK",
			},
		},
		{
			TraceId:                traceId,
			SpanId:                 userAccountSpanId,
			Name:                   "user-account",
			StartTimeUnixNano:      userAccountStartTime,
			EndTimeUnixNano:        userAccountEndTime,
			Kind:                   tracepb.Span_SPAN_KIND_SERVER,
			Attributes:             DefaultAttributes(),
			DroppedAttributesCount: 0,
			Events:                 events(dataGenerator),
			DroppedEventsCount:     0,
			Links:                  links(dataGenerator),
			DroppedLinksCount:      0,
			Status: &tracepb.Status{
				Code:    tracepb.Status_STATUS_CODE_OK,
				Message: "OK",
			},
		},
		{
			TraceId:                traceId,
			SpanId:                 userPreferencesSpanId,
			Name:                   "user-preferences",
			StartTimeUnixNano:      userPreferenceStartTime,
			EndTimeUnixNano:        userPreferenceEndTime,
			Kind:                   tracepb.Span_SPAN_KIND_SERVER,
			Attributes:             DefaultAttributes(),
			DroppedAttributesCount: 0,
			Events:                 events(dataGenerator),
			DroppedEventsCount:     0,
			Links:                  links(dataGenerator),
			DroppedLinksCount:      0,
			Status: &tracepb.Status{
				Code:    tracepb.Status_STATUS_CODE_OK,
				Message: "OK",
			},
		},
	}
}

// events returns a slice of events for the span.
func events(dataGenerator *DataGenerator) []*tracepb.Span_Event {
	events := make([]*tracepb.Span_Event, 3)
	for i := 0; i < 3; i++ {
		events[i] = &tracepb.Span_Event{
			TimeUnixNano: dataGenerator.CurrentTime(),
			//Name:         fmt.Sprintf("event-%d", i),			// ToDo fix this bug
			//Attributes:             DefaultSpanEventAttributes(),
			DroppedAttributesCount: 0,
		}
	}
	return events
}

// links returns a slice of links for the span.
func links(dataGenerator *DataGenerator) []*tracepb.Span_Link {
	dataGenerator.NextId8Bits()
	dataGenerator.NextId8Bits()

	links := make([]*tracepb.Span_Link, 3)
	for i := 0; i < 3; i++ {
		links[i] = &tracepb.Span_Link{
			TraceId:    dataGenerator.Id16Bits(),
			SpanId:     dataGenerator.Id8Bits(),
			TraceState: fmt.Sprintf("link-trace-state-%d", i),
			//Attributes:             DefaultSpanLinkAttributes(),
			DroppedAttributesCount: 0,
		}
	}
	return links
}
