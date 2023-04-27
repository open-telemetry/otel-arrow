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

package builder

import (
	"github.com/apache/arrow/go/v12/arrow/array"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/update"
)

type StringBuilder struct {
	builder       array.Builder
	transformNode *schema.TransformNode
	updateRequest *update.SchemaUpdateRequest
}

// NewStringBuilder creates a new StringBuilder.
func NewStringBuilder(builder array.Builder, transformNode *schema.TransformNode, updateRequest *update.SchemaUpdateRequest) *StringBuilder {
	return &StringBuilder{
		builder:       builder,
		transformNode: transformNode,
		updateRequest: updateRequest,
	}
}

func (b *StringBuilder) AppendNull() {
	if b.builder != nil {
		b.builder.AppendNull()
		return
	}
}

func (b *StringBuilder) Append(value string) {
	if b.builder != nil {
		// This is a workaround for dictionaries that do not support empty
		// strings.
		if value == "" {
			b.builder.AppendNull()
			return
		}

		switch builder := b.builder.(type) {
		case *array.StringBuilder:
			builder.Append(value)
		case *array.BinaryDictionaryBuilder:
			if err := builder.AppendString(value); err != nil {
				// Should never happen.
				panic(err)
			}
		default:
			// Should never happen.
			panic("unknown builder type")
		}

		return
	}

	b.transformNode.RemoveOptional()
	b.updateRequest.Inc()
}

func (b *StringBuilder) AppendNonEmpty(value string) {
	if b.builder != nil {
		if value == "" {
			b.builder.AppendNull()
			return
		}

		switch builder := b.builder.(type) {
		case *array.StringBuilder:
			builder.Append(value)
		case *array.BinaryDictionaryBuilder:
			if err := builder.AppendString(value); err != nil {
				// Should never happen.
				panic(err)
			}
		default:
			// Should never happen.
			panic("unknown builder type")
		}

		return
	}

	if value != "" {
		b.transformNode.RemoveOptional()
		b.updateRequest.Inc()
	}
}
