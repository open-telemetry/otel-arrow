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

package transform

import (
	"math"
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/stretchr/testify/assert"

	cfg "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/config"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/events"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/update"
	stats "github.com/open-telemetry/otel-arrow/go/pkg/otel/stats"
)

var evts = &events.Events{
	DictionariesWithOverflow:     make(map[string]bool),
	DictionariesIndexTypeChanged: make(map[string]string),
}

func TestNoDictionary(t *testing.T) {
	rbStats := &stats.RecordBuilderStats{}
	schemaUpdateRequest := update.NewSchemaUpdateRequest()

	dict := NewDictionaryField("", "1", nil, schemaUpdateRequest, evts)
	assert.Nil(t, dict.IndexType(), "index type should be nil (no dictionary config)")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8+1, rbStats)
	assert.Nil(t, dict.IndexType(), "index type should be nil (no dictionary config)")
	assert.Equal(t, 0, schemaUpdateRequest.Count())
}

func TestDictUint8Overflow(t *testing.T) {
	rbStats := &stats.RecordBuilderStats{}
	schemaUpdateRequest := update.NewSchemaUpdateRequest()
	dictConfig := cfg.NewDictionary(math.MaxUint8, 0.0)

	dict := NewDictionaryField("", "1", dictConfig, schemaUpdateRequest, evts)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(100, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8+1, rbStats)
	assert.Nil(t, dict.IndexType(), "index type should be nil (overflow)")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
}

func TestDictUint16Overflow(t *testing.T) {
	rbStats := &stats.RecordBuilderStats{}
	schemaUpdateRequest := update.NewSchemaUpdateRequest()
	dictConfig := cfg.NewDictionary(math.MaxUint16, 0.0)

	dict := NewDictionaryField("", "1", dictConfig, schemaUpdateRequest, evts)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(100, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint16, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint16+1, rbStats)
	assert.Nil(t, dict.IndexType(), "index type should be nil (overflow)")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
}

func TestDictUint32Overflow(t *testing.T) {
	rbStats := &stats.RecordBuilderStats{}
	schemaUpdateRequest := update.NewSchemaUpdateRequest()
	dictConfig := cfg.NewDictionary(math.MaxUint32, 0.0)

	dict := NewDictionaryField("", "1", dictConfig, schemaUpdateRequest, evts)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(100, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint16, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint16+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint32, dict.IndexType(), "index type should be uint32")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint32, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint32, dict.IndexType(), "index type should be uint32")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint32+1, rbStats)
	assert.Nil(t, dict.IndexType(), "index type should be nil (overflow)")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
}

func TestDictUint64Overflow(t *testing.T) {
	rbStats := &stats.RecordBuilderStats{}
	schemaUpdateRequest := update.NewSchemaUpdateRequest()
	dictConfig := cfg.NewDictionary(math.MaxUint64, 0.0)

	dict := NewDictionaryField("", "1", dictConfig, schemaUpdateRequest, evts)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(100, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint8, dict.IndexType(), "index type should be uint8")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint8+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint16, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint16, dict.IndexType(), "index type should be uint16")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint16+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint32, dict.IndexType(), "index type should be uint32")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint32, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint32, dict.IndexType(), "index type should be uint32")
	assert.Equal(t, 0, schemaUpdateRequest.Count())

	dict.SetCardinality(math.MaxUint32+1, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint64, dict.IndexType(), "index type should be uint64")
	assert.Equal(t, 1, schemaUpdateRequest.Count())
	schemaUpdateRequest.Reset()

	dict.SetCardinality(math.MaxUint64, rbStats)
	assert.Equal(t, arrow.PrimitiveTypes.Uint64, dict.IndexType(), "index type should be uint64")
	assert.Equal(t, 0, schemaUpdateRequest.Count())
}
