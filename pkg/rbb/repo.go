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

package rbb

import (
	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/memory"
	config2 "otel-arrow-adapter/pkg/rbb/config"
)

type RecordRepository struct {
	config *config2.Config

	// A map of SchemaId to RecordBuilder.
	builders map[string]*RecordBuilder

	// ToDo check if release is called properly
	allocator *memory.GoAllocator
}

func NewRecordRepository(config *config2.Config) *RecordRepository {
	return &RecordRepository{
		config:    config,
		builders:  make(map[string]*RecordBuilder),
		allocator: memory.NewGoAllocator(),
	}
}

func (rbr *RecordRepository) AddRecord(record *Record) {
	record.Normalize()
	schemaId := record.SchemaId()

	if rbb, ok := rbr.builders[schemaId]; ok {
		rbb.AddRecord(record)
	} else {
		rbr.builders[schemaId] = NewRecordBuilderWithRecord(record, rbr.config)
	}
}

// RecordBuilderCount returns the number of non-empty RecordBuilder in the repository.
func (rbr *RecordRepository) RecordBuilderCount() int {
	count := 0
	for _, rbb := range rbr.builders {
		if !rbb.IsEmpty() {
			count++
		}
	}
	return count
}

func (rbr *RecordRepository) Build() (map[string]arrow.Record, error) {
	recordBatches := make(map[string]arrow.Record)

	for schemaId, builder := range rbr.builders {
		if !builder.IsEmpty() {
			record, err := builder.Build(rbr.allocator)
			if err != nil {
				return nil, err
			}
			recordBatches[schemaId] = record
		}
	}

	return recordBatches, nil
}

func (rbr *RecordRepository) Optimize() {
	for _, rbb := range rbr.builders {
		rbb.Optimize()
	}
}

func (rbr *RecordRepository) Metadata() []*RecordBuilderMetadata {
	var metadata []*RecordBuilderMetadata
	for schemaId, rbb := range rbr.builders {
		if !rbb.IsEmpty() {
			metadata = append(metadata, rbb.Metadata(schemaId))
		}
	}
	return metadata
}
