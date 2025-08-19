/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package metrics

import (
	"testing"

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

	sig := DataPointSig[pmetric.NumberDataPoint](ndp, "k5")

	expected := []byte{1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 50, 1, 2, 3, 107, 51, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63, 107, 55, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63, 107, 56, 107, 49, 2, 0, 0, 0, 0, 0, 0, 0, 107, 52, 0, 0, 0, 0, 0, 0, 240, 63}

	require.Equal(t, expected, sig)
}
