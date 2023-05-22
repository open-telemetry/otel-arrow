/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

// A `related data` is an entity that is attached or related to a another entity.
// For example, `attributes` are related to `resource`, `span`, ...

import (
	"github.com/apache/arrow/go/v12/arrow"

	colarspb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	cfg "github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	config "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	// RelatedRecordBuilder is the common interface for all related record
	// builders.
	RelatedRecordBuilder interface {
		IsEmpty() bool
		Build() (arrow.Record, error)
		SchemaID() string
		Schema() *arrow.Schema
		PayloadType() *PayloadType
		Reset()
		Release()
	}

	// RelatedRecordsManager manages all related record builders for a given
	// main OTel entity.
	RelatedRecordsManager struct {
		cfg   *cfg.Config
		stats *stats.ProducerStats

		builders    []RelatedRecordBuilder
		builderExts []*builder.RecordBuilderExt

		schemas []SchemaWithPayload
	}

	// PayloadType wraps the protobuf payload type generated from the protobuf
	// definition and adds a prefix to it.
	PayloadType struct {
		prefix      string
		payloadType record_message.PayloadType
	}

	// All the payload types currently supported by the adapter.
	payloadTypes struct {
		Metrics *PayloadType
		Logs    *PayloadType
		Spans   *PayloadType

		ResourceAttrs     *PayloadType
		ScopeAttrs        *PayloadType
		Metric            *PayloadType
		IntGauge          *PayloadType
		IntGaugeAttrs     *PayloadType
		DoubleGauge       *PayloadType
		DoubleGaugeAttrs  *PayloadType
		IntSum            *PayloadType
		IntSumAttrs       *PayloadType
		DoubleSum         *PayloadType
		DoubleSumAttrs    *PayloadType
		Summary           *PayloadType
		SummaryAttrs      *PayloadType
		Histogram         *PayloadType
		HistogramAttrs    *PayloadType
		ExpHistogram      *PayloadType
		ExpHistogramAttrs *PayloadType
		LogRecordAttrs    *PayloadType
		SpanAttrs         *PayloadType
		Event             *PayloadType
		EventAttrs        *PayloadType
		Link              *PayloadType
		LinkAttrs         *PayloadType
	}

	SchemaWithPayload struct {
		Schema            *arrow.Schema
		PayloadType       *PayloadType
		ParentPayloadType *PayloadType
	}
)

// All the payload types currently supported by the adapter and their specific
// prefix and protobuf payload type.

var (
	PayloadTypes = payloadTypes{
		Metrics: &PayloadType{
			prefix:      "metrics",
			payloadType: colarspb.OtlpArrowPayloadType_METRICS,
		},
		Logs: &PayloadType{
			prefix:      "logs",
			payloadType: colarspb.OtlpArrowPayloadType_LOGS,
		},
		Spans: &PayloadType{
			prefix:      "spans",
			payloadType: colarspb.OtlpArrowPayloadType_SPANS,
		},
		ResourceAttrs: &PayloadType{
			prefix:      "resource-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS,
		},
		ScopeAttrs: &PayloadType{
			prefix:      "scope-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_SCOPE_ATTRS,
		},
		IntGauge: &PayloadType{
			prefix:      "int_gauge",
			payloadType: colarspb.OtlpArrowPayloadType_INT_GAUGE,
		},
		IntGaugeAttrs: &PayloadType{
			prefix:      "int_gauge-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_INT_GAUGE_ATTRS,
		},
		DoubleGauge: &PayloadType{
			prefix:      "double_gauge",
			payloadType: colarspb.OtlpArrowPayloadType_DOUBLE_GAUGE,
		},
		DoubleGaugeAttrs: &PayloadType{
			prefix:      "double_gauge-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_DOUBLE_GAUGE_ATTRS,
		},
		IntSum: &PayloadType{
			prefix:      "int_sum",
			payloadType: colarspb.OtlpArrowPayloadType_INT_SUM,
		},
		IntSumAttrs: &PayloadType{
			prefix:      "int_sum-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_INT_SUM_ATTRS,
		},
		DoubleSum: &PayloadType{
			prefix:      "double_sum",
			payloadType: colarspb.OtlpArrowPayloadType_DOUBLE_SUM,
		},
		DoubleSumAttrs: &PayloadType{
			prefix:      "double_sum-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_DOUBLE_SUM_ATTRS,
		},
		Summary: &PayloadType{
			prefix:      "summary",
			payloadType: colarspb.OtlpArrowPayloadType_SUMMARIES,
		},
		SummaryAttrs: &PayloadType{
			prefix:      "summary-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_SUMMARY_ATTRS,
		},
		Histogram: &PayloadType{
			prefix:      "histogram",
			payloadType: colarspb.OtlpArrowPayloadType_HISTOGRAMS,
		},
		HistogramAttrs: &PayloadType{
			prefix:      "histogram-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_HISTOGRAM_ATTRS,
		},
		ExpHistogram: &PayloadType{
			prefix:      "exp-histogram",
			payloadType: colarspb.OtlpArrowPayloadType_EXP_HISTOGRAMS,
		},
		ExpHistogramAttrs: &PayloadType{
			prefix:      "exp-histogram-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_EXP_HISTOGRAM_ATTRS,
		},
		LogRecordAttrs: &PayloadType{
			prefix:      "logs-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_LOG_ATTRS,
		},
		SpanAttrs: &PayloadType{
			prefix:      "span-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_SPAN_ATTRS,
		},
		Event: &PayloadType{
			prefix:      "span-event",
			payloadType: colarspb.OtlpArrowPayloadType_SPAN_EVENTS,
		},
		EventAttrs: &PayloadType{
			prefix:      "span-event-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_SPAN_EVENT_ATTRS,
		},
		Link: &PayloadType{
			prefix:      "span-link",
			payloadType: colarspb.OtlpArrowPayloadType_SPAN_LINKS,
		},
		LinkAttrs: &PayloadType{
			prefix:      "span-link-attrs",
			payloadType: colarspb.OtlpArrowPayloadType_SPAN_LINK_ATTRS,
		},
	}
)

func NewRelatedRecordsManager(cfg *cfg.Config, stats *stats.ProducerStats) *RelatedRecordsManager {
	return &RelatedRecordsManager{
		cfg:         cfg,
		stats:       stats,
		builders:    make([]RelatedRecordBuilder, 0),
		builderExts: make([]*builder.RecordBuilderExt, 0),
	}
}

func (m *RelatedRecordsManager) Declare(payloadType *PayloadType, parentPayloadType *PayloadType, schema *arrow.Schema, rrBuilder func(b *builder.RecordBuilderExt) RelatedRecordBuilder) RelatedRecordBuilder {
	builderExt := builder.NewRecordBuilderExt(m.cfg.Pool, schema, config.NewDictionary(m.cfg.LimitIndexSize), m.stats)
	builderExt.SetLabel(payloadType.SchemaPrefix())
	rBuilder := rrBuilder(builderExt)
	m.builders = append(m.builders, rBuilder)
	m.builderExts = append(m.builderExts, builderExt)
	m.schemas = append(m.schemas, SchemaWithPayload{
		Schema:            schema,
		PayloadType:       payloadType,
		ParentPayloadType: parentPayloadType,
	})
	return rBuilder
}

func (m *RelatedRecordsManager) BuildRecordMessages() ([]*record_message.RecordMessage, error) {
	recordMessages := make([]*record_message.RecordMessage, 0, len(m.builders))
	for _, b := range m.builders {
		if b.IsEmpty() {
			continue
		}
		record, err := b.Build()
		if err != nil {
			return nil, werror.WrapWithContext(
				err,
				map[string]interface{}{"schema_prefix": b.PayloadType().SchemaPrefix()},
			)
		}
		schemaID := b.PayloadType().SchemaPrefix() + ":" + b.SchemaID()
		relatedDataMessage := record_message.NewRelatedDataMessage(schemaID, record, b.PayloadType().PayloadType())
		recordMessages = append(recordMessages, relatedDataMessage)
	}
	return recordMessages, nil
}

func (m *RelatedRecordsManager) Schemas() []SchemaWithPayload {
	return m.schemas
}

func (m *RelatedRecordsManager) Reset() {
	for _, b := range m.builders {
		b.Reset()
	}
}

func (m *RelatedRecordsManager) Release() {
	for _, b := range m.builders {
		b.Release()
	}
	for _, b := range m.builderExts {
		b.Release()
	}
}

func (m *RelatedRecordsManager) RecordBuilderExt(payloadType *PayloadType) *builder.RecordBuilderExt {
	for i, b := range m.builders {
		if b.PayloadType() == payloadType {
			return m.builderExts[i]
		}
	}
	return nil
}

func (p *PayloadType) SchemaPrefix() string {
	return p.prefix
}

func (p *PayloadType) PayloadType() record_message.PayloadType {
	return p.payloadType
}
