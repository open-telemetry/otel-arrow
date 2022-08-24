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

package config_test

import (
	"testing"

	config2 "otel-arrow-adapter/pkg/air/config"
)

func TestIsDictionary(t *testing.T) {
	t.Parallel()
	config := config2.DictionaryConfig{
		MinRowCount:           10,
		MaxCard:               2,
		MaxCardRatio:          0.5,
		MaxSortedDictionaries: 5,
	}

	if !config.IsDictionary(10, 1, 10) {
		t.Errorf("Expected a dictionary")
	}
	if !config.IsDictionary(10, 2, 10) {
		t.Errorf("Expected a dictionary")
	}

	if config.IsDictionary(5, 1, 5) {
		t.Errorf("Didn't expect a dictionary (too few rows)")
	}
	if config.IsDictionary(10, 3, 10) {
		t.Errorf("Didn't rxpect a dictionary (too many unique values")
	}
}
