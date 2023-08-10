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

	"github.com/apache/arrow/go/v12/arrow"

	cfg "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	events "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/events"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/update"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
)

const DictIdKey = "dictId"

var (
	AllIndexTypes   = []arrow.DataType{arrow.PrimitiveTypes.Uint8, arrow.PrimitiveTypes.Uint16, arrow.PrimitiveTypes.Uint32, arrow.PrimitiveTypes.Uint64}
	AllIndexMaxCard = []uint64{math.MaxUint8, math.MaxUint16, math.MaxUint32, math.MaxUint64}
)

// DictionaryField is a FieldTransform that transforms dictionary fields to
// a given index type.
// If the index type is nil, the dictionary is downgraded to its value type.
type DictionaryField struct {
	path string

	// Dictionary ID
	DictID string

	// cumulativeTotal is the total number of values observed in the dictionary
	// since the creation of the dictionary.
	cumulativeTotal uint64

	// prevCumulativeTotal is the total number of values observed in the dictionary
	// before the last schema update.
	prevCumulativeTotal uint64

	// cardinality of the dictionary used to determine dictionary overflow
	cardinality uint64

	indexMaxCard []uint64
	indexTypes   []arrow.DataType
	currentIndex int

	schemaUpdateRequest *update.SchemaUpdateRequest
	events              *events.Events
}

func NewDictionaryField(
	path string,
	dictID string,
	config *cfg.Dictionary,
	schemaUpdateRequest *update.SchemaUpdateRequest,
	events *events.Events,
) *DictionaryField {
	df := DictionaryField{
		path:                path,
		DictID:              dictID,
		cardinality:         0,
		schemaUpdateRequest: schemaUpdateRequest,
		events:              events,
	}
	df.initIndices(config)
	return &df
}

// RevertCounters resets the cumulative total to the previous cumulative total.
func (t *DictionaryField) RevertCounters() {
	t.cumulativeTotal = t.prevCumulativeTotal
}

func (t *DictionaryField) AddTotal(total int) {
	t.prevCumulativeTotal = t.cumulativeTotal
	t.cumulativeTotal += uint64(total)
}

func (t *DictionaryField) SetCardinality(card uint64, stats *stats.RecordBuilderStats) {
	t.cardinality = card
	t.updateIndexType(stats)
}

func (t *DictionaryField) Cardinality() uint64 {
	return t.cardinality
}

func (t *DictionaryField) CumulativeTotal() uint64 {
	return t.cumulativeTotal
}

func (t *DictionaryField) IndexType() arrow.DataType {
	if t.indexTypes == nil {
		return nil
	}
	return t.indexTypes[t.currentIndex]
}

func (t *DictionaryField) Transform(field *arrow.Field) *arrow.Field {
	if t.indexTypes == nil {
		switch fieldType := field.Type.(type) {
		case *arrow.DictionaryType:
			// No index type defined, so the dictionary is downgraded to its
			// value type.
			return &arrow.Field{Name: field.Name, Type: fieldType.ValueType, Nullable: field.Nullable, Metadata: field.Metadata}
		default:
			// No index type defined, so the field is not transformed.
			return field
		}
	} else {
		// Add the dictionary ID to the metadata to ease the process checking
		// dictionary overflow.
		keys := append(field.Metadata.Keys(), DictIdKey)
		values := append(field.Metadata.Values(), t.DictID)
		metadataWithDictId := arrow.NewMetadata(keys, values)

		switch field.Type.(type) {
		case *arrow.DictionaryType:
			// Index type defined, so the dictionary is upgraded to the given
			// index type.
			dictType := &arrow.DictionaryType{
				IndexType: t.IndexType(),
				ValueType: field.Type.(*arrow.DictionaryType).ValueType,
				Ordered:   false,
			}
			return &arrow.Field{Name: field.Name, Type: dictType, Nullable: field.Nullable, Metadata: metadataWithDictId}
		default:
			// Index type defined, so field is converted to a dictionary.
			dictType := &arrow.DictionaryType{
				IndexType: t.IndexType(),
				ValueType: field.Type,
				Ordered:   false,
			}
			return &arrow.Field{Name: field.Name, Type: dictType, Nullable: field.Nullable, Metadata: metadataWithDictId}
		}
	}
}

func (t *DictionaryField) updateIndexType(stats *stats.RecordBuilderStats) {
	if t.indexTypes == nil {
		return
	}

	currentIndex := t.currentIndex

	for t.currentIndex < len(t.indexTypes) && t.cardinality > t.indexMaxCard[t.currentIndex] {
		t.currentIndex++
	}
	if t.currentIndex >= len(t.indexTypes) {
		t.indexTypes = nil
		t.indexMaxCard = nil
		t.currentIndex = 0
		t.schemaUpdateRequest.Inc()
		t.events.DictionariesWithOverflow[t.path] = true
		stats.DictionaryOverflowDetected++
	} else if t.currentIndex != currentIndex {
		t.schemaUpdateRequest.Inc()
		t.events.DictionariesIndexTypeChanged[t.path] = t.indexTypes[t.currentIndex].Name()
		stats.DictionaryIndexTypeChanged++
	}
}

func (t *DictionaryField) initIndices(config *cfg.Dictionary) {
	t.indexTypes = nil
	t.indexMaxCard = nil
	t.currentIndex = 0

	if config == nil || config.MaxCard == 0 {
		return
	}

	t.indexTypes = indexTypesRange(config.MinCard, config.MaxCard)
	t.indexMaxCard = indexMaxCardRange(config.MinCard, config.MaxCard)
}

func indexTypesRange(minCard uint64, maxCard uint64) []arrow.DataType {
	if minCard > maxCard {
		panic("minCard > maxCard")
	}
	return AllIndexTypes[findIndex(minCard) : findIndex(maxCard)+1]
}

func indexMaxCardRange(minCard uint64, maxCard uint64) []uint64 {
	if minCard > maxCard {
		panic("minCard > maxCard")
	}
	return AllIndexMaxCard[findIndex(minCard) : findIndex(maxCard)+1]
}

func findIndex(card uint64) int {
	if card <= math.MaxUint8 {
		return 0
	}
	if card <= math.MaxUint16 {
		return 1
	}
	if card <= math.MaxUint32 {
		return 2
	}
	return 3
}
