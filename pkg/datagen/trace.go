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

	"golang.org/x/exp/rand"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

var EVENT_NAMES = []string{"empty", "dns-lookup", "tcp-connect", "tcp-handshake", "tcp-send", "tcp-receive", "tcp-close", "http-send", "http-receive", "http-close", "message-send", "message-receive", "message-close", "grpc-send", "grpc-receive", "grpc-close", "grpc-status", "grpc-trailers", "unknown"}
var TRACE_STATES = []string{"started", "ended", "unknown"}

type TraceGenerator struct {
	*DataGenerator
}

type Span = ptrace.Span

func shuffleSpans(sl ptrace.SpanSlice, fs ...func(Span)) {
	span := sl.AppendEmpty()
	rand.Shuffle(len(fs), func(i, j int) {
		fs[i], fs[j] = fs[j], fs[i]
	})
	for _, f := range fs {
		f(span)
	}
}

func NewTraceGenerator(resourceAttributes []pcommon.Map, instrumentationScopes []pcommon.InstrumentationScope) *TraceGenerator {
	return &TraceGenerator{
		DataGenerator: NewDataGenerator(uint64(time.Now().UnixNano()/int64(time.Millisecond)), resourceAttributes, instrumentationScopes),
	}
}

func (tg *TraceGenerator) Generate(batchSize int, collectInterval time.Duration) ptrace.Traces {
	result := ptrace.NewTraces()

	resourceSpans := result.ResourceSpans().AppendEmpty()
	pick(tg.resourceAttributes).CopyTo(resourceSpans.Resource().Attributes())

	scopeSpans := resourceSpans.ScopeSpans().AppendEmpty()
	pick(tg.instrumentationScopes).CopyTo(scopeSpans.Scope())

	resourceSpans.SetSchemaUrl("https://opentelemetry.io/schemas/1.0.0")

	spans := scopeSpans.Spans()

	rand.Seed(uint64(time.Now().UnixNano()))
	for i := 0; i < batchSize; i++ {
		tg.AdvanceTime(collectInterval)
		tg.Spans(spans)
	}

	return result
}

func (dg *DataGenerator) Spans(spans ptrace.SpanSlice) {
	dg.NextId8Bytes()
	dg.NextId16Bytes()

	traceId := dg.Id16Bytes()
	rootSpanId := dg.Id8Bytes()
	rootStartTime := dg.CurrentTime()
	rootEndTime := dg.CurrentTime() + 1 + pcommon.Timestamp(rand.Intn(6))

	dg.AdvanceTime(time.Duration(rand.Intn(10)))

	dg.NextId8Bytes()
	userAccountSpanId := dg.Id8Bytes()
	userAccountStartTime := dg.CurrentTime()
	userAccountEndTime := dg.CurrentTime() + pcommon.Timestamp(rand.Intn(6))

	dg.NextId8Bytes()
	userPreferencesSpanId := dg.Id8Bytes()
	userPreferenceStartTime := dg.CurrentTime()
	userPreferenceEndTime := dg.CurrentTime() + pcommon.Timestamp(rand.Intn(4))

	shuffleSpans(spans,
		func(s Span) {
			s.SetTraceID(traceId)
			s.SetSpanID(rootSpanId)
			s.SetName("GET /user-info")
			s.SetStartTimestamp(rootStartTime)
			s.SetEndTimestamp(rootEndTime)
			s.SetKind(ptrace.SpanKindServer)
			DefaultAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links())
			s.Status().SetCode(ptrace.StatusCodeOk)
			s.Status().SetMessage("OK")
		},
		func(s Span) {
			s.SetTraceID(traceId)
			s.SetSpanID(userAccountSpanId)
			s.SetName("user-account")
			s.SetStartTimestamp(userAccountStartTime)
			s.SetEndTimestamp(userAccountEndTime)
			s.SetKind(ptrace.SpanKindServer)
			DefaultAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links())
			s.Status().SetCode(ptrace.StatusCodeError)
			s.Status().SetMessage("Error")
		},
		func(s Span) {
			s.SetTraceID(traceId)
			s.SetSpanID(userPreferencesSpanId)
			s.SetName("user-preferences")
			s.SetStartTimestamp(userPreferenceStartTime)
			s.SetEndTimestamp(userPreferenceEndTime)
			s.SetKind(ptrace.SpanKindServer)
			DefaultAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links())
			s.Status().SetCode(ptrace.StatusCodeOk)
			s.Status().SetMessage("OK")
		},
	)
}

// events returns a slice of events for the span.
func (dg *DataGenerator) events(ses ptrace.SpanEventSlice) {
	eventCount := rand.Intn(8) + 2

	for i := 0; i < eventCount; i++ {
		name := pick(EVENT_NAMES)
		attributes := DefaultSpanEventAttributes()
		if name == "empty" {
			attributes = pcommon.NewMap()
		}
		event := ses.AppendEmpty()
		event.SetTimestamp(dg.CurrentTime() + pcommon.Timestamp(rand.Intn(5)))
		event.SetName(name)
		attributes.CopyTo(event.Attributes())
	}
}

// links returns a slice of links for the span.
func (dg *DataGenerator) links(sls ptrace.SpanLinkSlice) {
	linkCount := rand.Intn(8) + 2
	dg.NextId16Bytes()

	for i := 0; i < linkCount; i++ {
		dg.NextId8Bytes()
		sl := sls.AppendEmpty()
		sl.SetTraceID(dg.Id16Bytes())
		sl.SetSpanID(dg.Id8Bytes())
		sl.SetTraceState(ptrace.TraceState(pick(TRACE_STATES)))
		DefaultSpanLinkAttributes().CopyTo(sl.Attributes())
	}
}
