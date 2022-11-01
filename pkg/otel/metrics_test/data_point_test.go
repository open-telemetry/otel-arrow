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

package metrics

import (
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/otel/metrics"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pmetric"
)

func TestDataPointSig(t *testing.T) {
	t.Parallel()

	ndp := pmetric.NewNumberDataPoint()
	ndp.SetStartTimestamp(1)
	ndp.SetTimestamp(2)
	attrs := ndp.Attributes()

	attrs.PutDouble("k4", 1.)
	attrs.PutInt("k1", 2)
	attrs.PutBool("k3", false)
	attrs.PutStr("k5", "bla")
	attrs.PutEmptyBytes("k2").FromRaw([]byte{1, 2, 3})
	k8val := attrs.PutEmptyMap("k8")
	k8val.PutDouble("k4", 1)
	k8val.PutInt("k1", 2)
	k7val := attrs.PutEmptyMap("k7")
	k7val.PutDouble("k4", 1)
	k7val.PutInt("k1", 2)

	sig := metrics.DataPointSig[pmetric.NumberDataPoint](ndp, "k5")

	expected := []byte{1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 50, 1, 2, 3, 107, 51, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63, 107, 55, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63, 107, 56, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63}

	require.Equal(t, expected, sig)
}
