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

// Attributes record builder for 16-bit IDs.
// ToDo move this to the common package once the metrics and logs builders are refactored.

import (
	"errors"

	"github.com/apache/arrow/go/v12/arrow"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Schema is the Arrow schema for the OTLP Arrow Traces record.
var (
	AttrsSchema32 = arrow.NewSchema([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.AttrsRecordKey, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.AttrsRecordValue, Type: AnyValueDT},
	}, nil)
)

type (
	Attrs32Builder struct {
		released bool

		builder *builder.RecordBuilderExt // Record builder

		ib *builder.Uint32Builder
		kb *builder.StringBuilder
		ab *AnyValueBuilder

		accumulator *Attributes32Accumulator
	}
)

func NewAttrs32Builder(rBuilder *builder.RecordBuilderExt) (*Attrs32Builder, error) {
	b := &Attrs32Builder{
		released:    false,
		builder:     rBuilder,
		accumulator: NewAttributes32Accumulator(),
	}
	if err := b.init(); err != nil {
		return nil, werror.Wrap(err)
	}
	return b, nil
}

func (b *Attrs32Builder) init() error {
	b.ib = b.builder.Uint32Builder("id")
	b.kb = b.builder.StringBuilder("key")
	b.ab = AnyValueBuilderFrom(b.builder.SparseUnionBuilder("value"))
	return nil
}

func (b *Attrs32Builder) Accumulator() *Attributes32Accumulator {
	return b.accumulator
}

func (b *Attrs32Builder) IsEmpty() bool {
	return b.accumulator.IsEmpty()
}

func (b *Attrs32Builder) TryBuild() (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(ErrBuilderAlreadyReleased)
	}

	for _, attr := range b.accumulator.SortedAttrs() {
		b.ib.Append(attr.ID)
		b.kb.Append(attr.Key)
		if err := b.ab.Append(attr.Value); err != nil {
			return nil, werror.Wrap(err)
		}
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		initErr := b.init()
		if initErr != nil {
			err = werror.Wrap(initErr)
		}
	} else {
		//PrintRecord(record)
	}

	return
}

func (b *Attrs32Builder) Build() (arrow.Record, error) {
	schemaNotUpToDateCount := 0

	var record arrow.Record
	var err error

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		record, err = b.TryBuild()
		if err != nil {
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return nil, werror.Wrap(err)
			}
		} else {
			break
		}
	}
	return record, werror.Wrap(err)
}

func (b *Attrs32Builder) SchemaID() string {
	return b.builder.SchemaID()
}

// Release releases the memory allocated by the builder.
func (b *Attrs32Builder) Release() {
	if !b.released {
		b.builder.Release()
		b.released = true
	}
}

func (b *Attrs32Builder) ShowSchema() {
	b.builder.ShowSchema()
}
