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

package otlp

import (
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type UnivariateGaugeIds struct {
	DataPoints *UnivariateNdpIds
}

func NewUnivariateGaugeIds(parentDT *arrow.StructType) (*UnivariateGaugeIds, error) {
	dataPoints, err := NewUnivariateNdpIds(parentDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &UnivariateGaugeIds{
		DataPoints: dataPoints,
	}, nil
}

func UpdateUnivariateGaugeFrom(gauge pmetric.Gauge, arr *array.Struct, row int, ids *UnivariateGaugeIds, smdata *SharedData, mdata *SharedData) error {
	los, err := arrowutils.ListOfStructsFromStruct(arr, ids.DataPoints.Id, row)
	if err != nil {
		return werror.Wrap(err)
	}
	return AppendUnivariateNdpInto(gauge.DataPoints(), los, ids.DataPoints, smdata, mdata)
}
