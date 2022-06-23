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

// Config defines the configuration supported by the RecordBatchRepository.
type Config struct {
	// Configuration for the dictionaries
	Dictionaries DictionariesConfig
}

// DictionariesConfig defines dictionaries configuration (binary and string)
type DictionariesConfig struct {
	// Dictionary options for binary columns
	BinaryColumns DictionaryConfig
	// Dictionary options for string columns
	StringColumns DictionaryConfig
}

// DictionaryConfig defines the dictionary configuration.
type DictionaryConfig struct {
	// The creation of a dictionary will be performed only on columns with more than `min_row_count` elements.
	MinRowCount uint64
	// The creation of a dictionary will be performed only on columns with a cardinality lower than `max_card`.
	MaxCard uint64
	// The creation of a dictionary will only be performed on columns with a ratio `card` / `size` <= `max_card_ratio`.
	MaxCardRatio float64
	// Maximum number of sorted dictionaries (based on cardinality/total_size and avg_data_lenght).
	MaxSortedDictionaries uint64
}
