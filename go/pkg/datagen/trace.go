/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package datagen

import (
	"fmt"
	"time"

	"github.com/brianvoe/gofakeit/v6"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

var EventNames = []string{"empty", "dns-lookup", "tcp-connect", "tcp-handshake", "tcp-send", "tcp-receive", "tcp-close", "http-send", "http-receive", "http-close", "message-send", "message-receive", "message-close", "grpc-send", "grpc-receive", "grpc-close", "grpc-status", "grpc-trailers", "unknown"}
var TraceStates = []string{"started", "ended", "unknown"}

type TraceGenerator struct {
	*DataGenerator
}

type Span = ptrace.Span

func (te TestEntropy) shuffleSpans(sl ptrace.SpanSlice, fs ...func(Span)) {
	span := sl.AppendEmpty()
	te.rng.Shuffle(len(fs), func(i, j int) {
		fs[i], fs[j] = fs[j], fs[i]
	})
	for _, f := range fs {
		f(span)
	}
}

func NewTracesGenerator(entropy TestEntropy, resourceAttributes []pcommon.Map, instrumentationScopes []pcommon.InstrumentationScope) *TraceGenerator {
	return &TraceGenerator{
		DataGenerator: NewDataGenerator(entropy, resourceAttributes, instrumentationScopes),
	}
}

func (tg *TraceGenerator) Generate(batchSize int, collectInterval time.Duration) ptrace.Traces {
	result := ptrace.NewTraces()

	resourceSpans := result.ResourceSpans().AppendEmpty()
	pick(tg.TestEntropy, tg.resourceAttributes).CopyTo(resourceSpans.Resource().Attributes())

	scopeSpans := resourceSpans.ScopeSpans().AppendEmpty()
	pick(tg.TestEntropy, tg.instrumentationScopes).CopyTo(scopeSpans.Scope())

	resourceSpans.SetSchemaUrl("https://opentelemetry.io/schemas/1.0.0")

	spans := scopeSpans.Spans()

	for i := 0; i < batchSize; i++ {
		tg.AdvanceTime(collectInterval)
		tg.Spans(spans)
	}

	return result
}

func (tg *TraceGenerator) GenerateRandomTraces(batchSize int, collectInterval time.Duration) ptrace.Traces {
	result := ptrace.NewTraces()

	// Generate 4 resource spans per batch.
	for i := 0; i < 4; i++ {
		resourceSpans := result.ResourceSpans().AppendEmpty()
		// ~50% of the time, generate a random resource
		if tg.GenBool() {
			pick(tg.TestEntropy, tg.resourceAttributes).CopyTo(resourceSpans.Resource().Attributes())
		}

		// Generate 4 scope spans per resource span.
		for j := 0; j < 4; j++ {
			scopeSpans := resourceSpans.ScopeSpans().AppendEmpty()
			// ~50% of the time, generate a random scope
			if tg.GenBool() {
				pick(tg.TestEntropy, tg.instrumentationScopes).CopyTo(scopeSpans.Scope())
			}

			if tg.GenBool() {
				scopeSpans.SetSchemaUrl(fmt.Sprintf("https://opentelemetry.io/schemas/1.%d.%d", i, j))
			}

			spans := scopeSpans.Spans()

			spanCount := tg.rng.Intn(batchSize) + 1
			for i := 0; i < spanCount; i++ {
				tg.AdvanceTime(time.Duration(tg.rng.Intn(int(collectInterval))))
				tg.AddRandomSpansTo(spans)
			}
		}
		if tg.GenBool() {
			resourceSpans.SetSchemaUrl(fmt.Sprintf("https://opentelemetry.io/schemas/1.0.%d", i))
		}
	}

	return result
}

func (dg *DataGenerator) Spans(spans ptrace.SpanSlice) {
	dg.NextId16Bytes()
	traceId := dg.Id16Bytes()

	dg.NextId8Bytes()
	rootSpanId := dg.Id8Bytes()
	rootStartTime := dg.CurrentTime()
	rootEndTime := dg.CurrentTime() + 1 + pcommon.Timestamp(dg.rng.Intn(6))

	dg.AdvanceTime(time.Duration(dg.rng.Intn(10)))

	dg.NextId8Bytes()
	userAccountSpanId := dg.Id8Bytes()
	userAccountStartTime := dg.CurrentTime()
	userAccountEndTime := dg.CurrentTime() + pcommon.Timestamp(dg.rng.Intn(6))

	dg.NextId8Bytes()
	userPreferencesSpanId := dg.Id8Bytes()
	userPreferenceStartTime := dg.CurrentTime()
	userPreferenceEndTime := dg.CurrentTime() + pcommon.Timestamp(dg.rng.Intn(4))

	dg.shuffleSpans(spans,
		func(s Span) {
			s.SetTraceID(traceId)
			s.SetSpanID(rootSpanId)
			s.SetName("GET /user-info")
			s.SetStartTimestamp(rootStartTime)
			s.SetEndTimestamp(rootEndTime)
			s.SetKind(ptrace.SpanKindServer)
			dg.NewStandardAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links(), traceId, rootSpanId)
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
			dg.NewStandardAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links(), traceId, userAccountSpanId)
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
			dg.NewStandardAttributes().CopyTo(s.Attributes())
			dg.events(s.Events())
			dg.links(s.Links(), traceId, userPreferencesSpanId)
			s.Status().SetCode(ptrace.StatusCodeOk)
			s.Status().SetMessage("OK")
		},
	)
}

func (dg *DataGenerator) AddRandomSpansTo(spans ptrace.SpanSlice) {
	dg.NextId16Bytes()
	traceId := dg.Id16Bytes()

	dg.NextId8Bytes()
	rootSpanId := dg.Id8Bytes()
	rootStartTime := dg.CurrentTime()
	rootEndTime := dg.CurrentTime() + 1 + pcommon.Timestamp(dg.rng.Intn(6))

	dg.AdvanceTime(time.Duration(dg.rng.Intn(10)))

	span := spans.AppendEmpty()
	if dg.GenBool() {
		span.SetTraceID(traceId)
	}
	if dg.GenBool() {
		span.SetSpanID(rootSpanId)
	}
	if dg.GenBool() {
		span.SetParentSpanID(rootSpanId)
	}
	if dg.GenBool() {
		span.TraceState().FromRaw(gofakeit.LoremIpsumWord())
	}
	if dg.GenBool() {
		span.SetName("GET /user-info")
	}
	if dg.GenBool() {
		span.SetStartTimestamp(rootStartTime)
	}
	if dg.GenBool() {
		span.SetEndTimestamp(rootEndTime)
	}
	if dg.GenBool() {
		span.SetKind(ptrace.SpanKindServer)
	}
	if dg.GenBool() {
		dg.RandomAttributes().CopyTo(span.Attributes())
	}
	if dg.GenBool() {
		span.SetDroppedAttributesCount(uint32(dg.rng.Intn(10)))
	}
	if dg.GenBool() {
		dg.RandomEvents(span.Events())
	}
	if dg.GenBool() {
		span.SetDroppedEventsCount(uint32(dg.rng.Intn(10)))
	}
	if dg.GenBool() {
		dg.RandomLinks(span.Links(), traceId, rootSpanId)
	}
	if dg.GenBool() {
		span.SetDroppedLinksCount(uint32(dg.rng.Intn(10)))
	}
	if dg.GenBool() {
		span.Status().SetCode(ptrace.StatusCodeOk)
	}
	if dg.GenBool() {
		span.Status().SetMessage("OK")
	}
}

// events returns a slice of events for the span.
func (dg *DataGenerator) events(ses ptrace.SpanEventSlice) {
	eventCount := dg.rng.Intn(8) + 2

	for i := 0; i < eventCount; i++ {
		name := pick(dg.TestEntropy, EventNames)
		attributes := dg.NewStandardSpanEventAttributes()
		if name == "empty" {
			attributes = pcommon.NewMap()
		}
		event := ses.AppendEmpty()
		event.SetTimestamp(dg.CurrentTime() + pcommon.Timestamp(dg.rng.Intn(5)))
		event.SetName(name)
		attributes.CopyTo(event.Attributes())
	}
}

func (dg *DataGenerator) RandomEvents(ses ptrace.SpanEventSlice) {
	eventCount := dg.rng.Intn(10)

	for i := 0; i < eventCount; i++ {
		event := ses.AppendEmpty()
		if dg.GenBool() {
			event.SetTimestamp(dg.CurrentTime() + pcommon.Timestamp(dg.rng.Intn(5)))
		}
		if dg.GenBool() {
			event.SetName(gofakeit.LoremIpsumWord())
		}
		if dg.GenBool() {
			attributes := dg.RandomAttributes()
			attributes.CopyTo(event.Attributes())
		}
		if dg.GenBool() {
			event.SetDroppedAttributesCount(uint32(dg.rng.Intn(10)))
		}
	}
}

// links returns a slice of links for the span.
func (dg *DataGenerator) links(sls ptrace.SpanLinkSlice, traceID pcommon.TraceID, spanID pcommon.SpanID) {
	linkCount := dg.rng.Intn(8) + 2

	for i := 0; i < linkCount; i++ {
		sl := sls.AppendEmpty()
		sl.SetTraceID(traceID)
		sl.SetSpanID(spanID)
		sl.TraceState().FromRaw(pick(dg.TestEntropy, TraceStates))
		dg.NewStandardSpanLinkAttributes().CopyTo(sl.Attributes())
	}
}

func (dg *DataGenerator) RandomLinks(sls ptrace.SpanLinkSlice, traceID pcommon.TraceID, spanID pcommon.SpanID) {
	linkCount := dg.rng.Intn(10)

	for i := 0; i < linkCount; i++ {
		sl := sls.AppendEmpty()
		if dg.GenBool() {
			sl.SetTraceID(traceID)
		}
		if dg.GenBool() {
			sl.SetSpanID(spanID)
		}
		if dg.GenBool() {
			sl.TraceState().FromRaw(gofakeit.LoremIpsumWord())
		}
		if dg.GenBool() {
			attributes := dg.RandomAttributes()
			attributes.CopyTo(sl.Attributes())
		}
		if dg.GenBool() {
			sl.SetDroppedAttributesCount(uint32(dg.rng.Intn(10)))
		}
	}
}
