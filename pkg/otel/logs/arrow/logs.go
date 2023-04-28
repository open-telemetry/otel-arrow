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

import (
	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/plog"

	"github.com/f5/otel-arrow-adapter/pkg/config"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Schema is the Arrow schema for the OTLP Arrow Logs record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceLogs, Type: arrow.ListOf(ResourceLogsDT)},
	}, nil)
)

// LogsBuilder is a helper to build a list of resource logs.
type LogsBuilder struct {
	released bool

	builder *builder.RecordBuilderExt // Record builder

	rlb *builder.ListBuilder // ResourceLogs list builder
	rlp *ResourceLogsBuilder // resource logs builder

	optimizer *LogsOptimizer
	analyzer  *LogsAnalyzer

	relatedData *RelatedData
}

// NewLogsBuilder creates a new LogsBuilder with a given allocator.
func NewLogsBuilder(
	recordBuilder *builder.RecordBuilderExt,
	cfg *config.Config,
	stats *stats.ProducerStats,
) (*LogsBuilder, error) {
	var optimizer *LogsOptimizer
	var analyzer *LogsAnalyzer

	relatedData, err := NewRelatedData(cfg, stats)
	if err != nil {
		panic(err)
	}

	if stats.SchemaStatsEnabled {
		optimizer = NewLogsOptimizer(acommon.WithStats(), acommon.WithSort())
		analyzer = NewLogsAnalyzer()
	} else {
		optimizer = NewLogsOptimizer(acommon.WithSort())
	}

	b := &LogsBuilder{
		released:    false,
		builder:     recordBuilder,
		optimizer:   optimizer,
		analyzer:    analyzer,
		relatedData: relatedData,
	}

	if err := b.init(); err != nil {
		return nil, werror.Wrap(err)
	}

	return b, nil
}

func (b *LogsBuilder) init() error {
	rlb := b.builder.ListBuilder(constants.ResourceLogs)
	b.rlb = rlb
	b.rlp = ResourceLogsBuilderFrom(rlb.StructBuilder())
	return nil
}

func (b *LogsBuilder) RelatedData() *RelatedData {
	return b.relatedData
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *LogsBuilder) Build() (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		initErr := b.init()
		if initErr != nil {
			err = werror.Wrap(initErr)
		}
	}

	return
}

// Append appends a new set of resource logs to the builder.
func (b *LogsBuilder) Append(logs plog.Logs) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	optimLogs := b.optimizer.Optimize(logs)
	if b.analyzer != nil {
		b.analyzer.Analyze(optimLogs)
		b.analyzer.ShowStats("")
	}

	rc := len(optimLogs.ResourceLogs)
	return b.rlb.Append(rc, func() error {
		for _, resLogGroup := range optimLogs.ResourceLogs {
			if err := b.rlp.Append(resLogGroup, b.relatedData); err != nil {
				return werror.Wrap(err)
			}
		}
		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *LogsBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.released = true

		b.relatedData.Release()
	}
}

func (b *LogsBuilder) ShowSchema() {
	b.builder.ShowSchema()
}
