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

package oteltest

import (
	"bytes"
	"encoding/json"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/metrics"
	"sort"

	"github.com/google/go-cmp/cmp"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
)

type otelCopier[T any] interface {
	metrics.DataPoint
	CopyTo(T)
}

type otelPointSlice[PT metrics.DataPoint, ST otelCopier[PT]] interface {
	At(int) ST
	Len() int
	RemoveIf(func(PT) bool)
	AppendEmpty() PT
}

func sortPoints[PT metrics.DataPoint, ST otelCopier[PT]](slice otelPointSlice[PT, ST]) {
	num := slice.Len()
	var tmp []ST
	for i := 0; i < num; i++ {
		tmp = append(tmp, slice.At(i))
		slice.At(i).Attributes().Sort()
	}
	sort.Slice(tmp, func(i, j int) bool {
		a := tmp[i]
		b := tmp[j]

		// Note: no multikey is passed into the data point signature because we
		// are not about to remove that key.  DataPointSig takes that argument
		// to compute the schema that will result after ignoring the multivariate
		// key.
		asig := metrics.DataPointSig(a, "")
		bsig := metrics.DataPointSig(b, "")

		return bytes.Compare(asig, bsig) < 0
	})

	slice.RemoveIf(func(PT) bool { return true })

	for _, item := range tmp {
		item.CopyTo(slice.AppendEmpty())
	}
}

func sortMetrics(x pmetric.Metrics) {
	for ri := 0; ri < x.ResourceMetrics().Len(); ri++ {
		rm := x.ResourceMetrics().At(ri)
		rm.Resource().Attributes().Sort()
		for si := 0; si < rm.ScopeMetrics().Len(); si++ {
			sm := rm.ScopeMetrics().At(si)
			sm.Scope().Attributes().Sort()
			for mi := 0; mi < sm.Metrics().Len(); mi++ {
				m := sm.Metrics().At(mi)
				switch m.DataType() {
				case pmetric.MetricDataTypeGauge:
					sortPoints[pmetric.NumberDataPoint, pmetric.NumberDataPoint](
						m.Gauge().DataPoints(),
					)
				case pmetric.MetricDataTypeSum:
					sortPoints[pmetric.NumberDataPoint, pmetric.NumberDataPoint](
						m.Sum().DataPoints(),
					)
				case pmetric.MetricDataTypeHistogram,
					pmetric.MetricDataTypeExponentialHistogram,
					pmetric.MetricDataTypeSummary:
					panic("unsupported case")
				}
			}
		}
	}
}

func DiffMetrics(x, y pmetric.Metrics) string {
	sortMetrics(x)
	sortMetrics(y)

	rx := pmetricotlp.NewRequestFromMetrics(x)
	ry := pmetricotlp.NewRequestFromMetrics(y)

	dx, err := rx.MarshalJSON()
	if err != nil {
		panic(err)
	}
	dy, err := ry.MarshalJSON()
	if err != nil {
		panic(err)
	}

	var gx interface{}
	var gy interface{}

	if err = json.Unmarshal(dx, &gx); err != nil {
		panic(err)
	}
	if err = json.Unmarshal(dy, &gy); err != nil {
		panic(err)
	}

	return cmp.Diff(gx, gy)
}
