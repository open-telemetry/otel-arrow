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

package arrow // import "github.com/f5/otel-arrow-adapter/collector/exporter/otlpexporter/internal/arrow"

import "fmt"

// Settings includes whether Arrow is enabled and the number of
// concurrent Arrow streams.
type Settings struct {
	Enabled    bool `mapstructure:"enabled"`
	NumStreams int  `mapstructure:"num_streams"`
}

// Validate returns an error when the number of streams is less than 1.
func (cfg *Settings) Validate() error {
	if cfg.NumStreams < 1 {
		return fmt.Errorf("stream count must be > 0: %d", cfg.NumStreams)
	}

	return nil
}

// NewDefaultSettings returns a default Settings, in which Arrow is disabled.
func NewDefaultSettings() *Settings {
	return &Settings{
		NumStreams: 1,
		Enabled:    false,
	}
}
