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

type RecordBatchRepository struct {
	config *Config

	// A map of SchemaId to RecordBatchBuilder.
	builders map[string]*RecordBatchBuilder
}

func NewRecordBatchRepository(config *Config) *RecordBatchRepository {
	return &RecordBatchRepository{
		config:   config,
		builders: make(map[string]*RecordBatchBuilder),
	}
}

func (rbr *RecordBatchRepository) AddRecord(record *Record) {
	record.Normalize()
	schemaId := record.SchemaId()

	if rbb, ok := rbr.builders[schemaId]; ok {
		rbb.AddRecord(record)
	} else {
		rbr.builders[schemaId] = NewRecordBatchBuilderWithRecord(record, rbr.config)
	}
}

// RecordBatchBuilderCount returns the number of non-empty RecordBatchBuilder in the repository.
func (rbr *RecordBatchRepository) RecordBatchBuilderCount() int {
	count := 0
	for _, rbb := range rbr.builders {
		if !rbb.IsEmpty() {
			count++
		}
	}
	return count
}

func (rbr *RecordBatchRepository) Optimize() {
	for _, rbb := range rbr.builders {
		rbb.Optimize()
	}
}

func (rbr *RecordBatchRepository) Metadata() []*RecordBatchBuilderMetadata {
	metadata := []*RecordBatchBuilderMetadata{}
	for schemaId, rbb := range rbr.builders {
		if !rbb.IsEmpty() {
			metadata = append(metadata, rbb.Metadata(schemaId))
		}
	}
	return metadata
}
