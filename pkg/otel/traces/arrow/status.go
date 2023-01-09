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

package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// StatusDT is the Arrow Data Type describing a span status.
var (
	StatusDT = arrow.StructOf([]arrow.Field{
		{Name: constants.StatusCode, Type: acommon.DefaultDictInt32},
		{Name: constants.StatusMessage, Type: acommon.DefaultDictString},
	}...)
)

type StatusBuilder struct {
	released bool
	builder  *array.StructBuilder
	scb      *acommon.AdaptiveDictionaryBuilder // status code builder
	smb      *acommon.AdaptiveDictionaryBuilder // status message builder
}

func NewStatusBuilder(pool memory.Allocator) *StatusBuilder {
	return StatusBuilderFrom(array.NewStructBuilder(pool, StatusDT))
}

func StatusBuilderFrom(sb *array.StructBuilder) *StatusBuilder {
	return &StatusBuilder{
		released: false,
		builder:  sb,
		scb:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(0)),
		smb:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(1)),
	}
}

// Append appends a new span status to the builder.
func (b *StatusBuilder) Append(status ptrace.Status) error {
	if b.released {
		return fmt.Errorf("status builder already released")
	}

	b.builder.Append(true)
	if err := b.scb.AppendI32(int32(status.Code())); err != nil {
		return err
	}
	message := status.Message()
	if message == "" {
		b.smb.AppendNull()
	} else {
		if err := b.smb.AppendString(message); err != nil {
			return err
		}
	}
	return nil
}

// Build builds the span status array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *StatusBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("status builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *StatusBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
