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

package air_test

import (
	"fmt"

	"github.com/lquerel/otel-arrow-adapter/pkg/air"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/rfield"
)

func GenSimpleRecord(ts int64) *air.Record {
	record := air.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("a", "a")
	record.StringField("b", "b")
	record.StringField("c", "c")
	return record
}

func GenComplexRecord(ts int64) *air.Record {
	record := air.NewRecord()
	record.I64Field("ts", ts)
	record.StructField("a", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "b", Value: rfield.NewString("b")},
			{Name: "c", Value: rfield.NewString("c")},
		},
	})
	record.StringField("b", "b")
	record.StringField("c", "c")
	return record
}

func GenRecord(ts int64, value_a, value_b, value_c int) *air.Record {
	record := air.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("c", fmt.Sprintf("c_%d", value_c))
	record.StringField("a", fmt.Sprintf("a___%d", value_a))
	record.StringField("b", fmt.Sprintf("b__%d", value_b))
	record.StructField("d", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "a", Value: rfield.NewString(fmt.Sprintf("a_%d", value_a))},
			{Name: "b", Value: rfield.NewString(fmt.Sprintf("b_%d", value_b))},
			{Name: "c", Value: &rfield.List{Values: []rfield.Value{
				rfield.NewI64(1),
				rfield.NewI64(2),
				rfield.NewI64(3),
			}}},
			{Name: "d", Value: &rfield.List{Values: []rfield.Value{
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 1),
					rfield.NewF64Field("b", 2.0),
					rfield.NewStringField("c", "3"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 4),
					rfield.NewF64Field("b", 5.0),
					rfield.NewStringField("c", "6"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 7),
					rfield.NewF64Field("b", 8.0),
					rfield.NewStringField("c", "9"),
				}},
			}}},
		},
	})
	return record
}
