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

// Infrastructure to manage related records.

import (
	"math"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	cfg "github.com/f5/otel-arrow-adapter/pkg/config"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	config "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	// RelatedData is a collection of related/dependent data to log record
	// entities.
	RelatedData struct {
		logRecordCount uint64

		attrsBuilders       *AttrsBuilders
		attrsRecordBuilders *AttrsRecordBuilders
	}

	// AttrsBuilders groups together AttrsBuilder instances used to build related
	// data attributes (i.e. resource attributes, scope attributes, and log record
	// attributes.
	AttrsBuilders struct {
		resource  *carrow.Attrs16Builder
		scope     *carrow.Attrs16Builder
		logRecord *carrow.Attrs16Builder
	}

	// AttrsRecordBuilders is a collection of RecordBuilderExt instances used
	// to build related data records (i.e. resource attributes, scope attributes,
	// and log record attributes.
	AttrsRecordBuilders struct {
		resource  *builder.RecordBuilderExt
		scope     *builder.RecordBuilderExt
		logRecord *builder.RecordBuilderExt
	}
)

func NewRelatedData(cfg *cfg.Config, stats *stats.ProducerStats) (*RelatedData, error) {
	attrsResourceRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsScopeRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsLogRecordRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)

	attrsResourceBuilder, err := carrow.NewAttrs16Builder(attrsResourceRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsScopeBuilder, err := carrow.NewAttrs16Builder(attrsScopeRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsLogRecordBuilder, err := carrow.NewAttrs16Builder(attrsLogRecordRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &RelatedData{
		attrsBuilders: &AttrsBuilders{
			resource:  attrsResourceBuilder,
			scope:     attrsScopeBuilder,
			logRecord: attrsLogRecordBuilder,
		},
		attrsRecordBuilders: &AttrsRecordBuilders{
			resource:  attrsResourceRB,
			scope:     attrsScopeRB,
			logRecord: attrsLogRecordRB,
		},
	}, nil
}

func (r *RelatedData) Release() {
	if r.attrsBuilders != nil {
		r.attrsBuilders.Release()
	}
	if r.attrsRecordBuilders != nil {
		r.attrsRecordBuilders.Release()
	}
}

func (r *RelatedData) AttrsBuilders() *AttrsBuilders {
	return r.attrsBuilders
}

func (r *RelatedData) AttrsRecordBuilders() *AttrsRecordBuilders {
	return r.attrsRecordBuilders
}

func (r *RelatedData) Reset() {
	r.logRecordCount = 0
	r.attrsBuilders.Reset()
}

func (r *RelatedData) LogRecordCount() uint16 {
	return uint16(r.logRecordCount)
}

func (r *RelatedData) NextSpanID() uint16 {
	c := r.logRecordCount

	if c == math.MaxUint16 {
		panic("maximum number of log records reached per batch, please reduce the batch size to a maximum of 65535 log records")
	}

	r.logRecordCount++
	return uint16(c)
}

func (r *RelatedData) BuildRecordMessages() ([]*record_message.RecordMessage, error) {
	recordMessages := make([]*record_message.RecordMessage, 0, 6)

	if !r.attrsBuilders.resource.IsEmpty() {
		attrsResRec, err := r.attrsBuilders.resource.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "resource_attrs:" + r.attrsBuilders.resource.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsResRec, colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS))
	}

	if !r.attrsBuilders.scope.IsEmpty() {
		attrsScopeRec, err := r.attrsBuilders.scope.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "scope_attrs:" + r.attrsBuilders.scope.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsScopeRec, colarspb.OtlpArrowPayloadType_SCOPE_ATTRS))
	}

	if !r.attrsBuilders.logRecord.IsEmpty() {
		attrsLogRecordRec, err := r.attrsBuilders.logRecord.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "log_attrs:" + r.attrsBuilders.logRecord.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsLogRecordRec, colarspb.OtlpArrowPayloadType_LOG_ATTRS))
	}

	return recordMessages, nil
}

func (ab *AttrsBuilders) Release() {
	ab.resource.Release()
	ab.scope.Release()
	ab.logRecord.Release()
}

func (ab *AttrsBuilders) Resource() *carrow.Attrs16Builder {
	return ab.resource
}

func (ab *AttrsBuilders) Scope() *carrow.Attrs16Builder {
	return ab.scope
}

func (ab *AttrsBuilders) LogRecord() *carrow.Attrs16Builder {
	return ab.logRecord
}

func (ab *AttrsBuilders) Reset() {
	ab.resource.Accumulator().Reset()
	ab.scope.Accumulator().Reset()
	ab.logRecord.Accumulator().Reset()
}

func (arb *AttrsRecordBuilders) Release() {
	arb.resource.Release()
	arb.scope.Release()
	arb.logRecord.Release()
}

func (arb *AttrsRecordBuilders) Resource() *builder.RecordBuilderExt {
	return arb.resource
}

func (arb *AttrsRecordBuilders) Scope() *builder.RecordBuilderExt {
	return arb.scope
}

func (arb *AttrsRecordBuilders) LogRecord() *builder.RecordBuilderExt {
	return arb.logRecord
}
