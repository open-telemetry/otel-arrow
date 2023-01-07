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

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSettingsValidate(t *testing.T) {
	settings := func(enabled bool, numStreams int) *Settings {
		return &Settings{Enabled: enabled, NumStreams: numStreams}
	}
	require.NoError(t, settings(true, 1).Validate())
	require.NoError(t, settings(false, 1).Validate())
	require.NoError(t, settings(true, 2).Validate())
	require.NoError(t, settings(true, math.MaxInt).Validate())

	require.Error(t, settings(true, 0).Validate())
	require.Contains(t, settings(true, 0).Validate().Error(), "stream count must be")
	require.Error(t, settings(false, -1).Validate())
	require.Error(t, settings(true, math.MinInt).Validate())
}

func TestDefaultSettings(t *testing.T) {
	require.NoError(t, NewDefaultSettings().Validate())

}
