/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package value

import (
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
)

func TestListDataType(t *testing.T) {
	t.Parallel()

	list := rfield.List{
		Values: []rfield.Value{
			rfield.NewStruct([]*rfield.Field{
				rfield.NewU64Field("start_time", 123),
				rfield.NewU64Field("end_time", 124),
				rfield.NewStringField("name", "bla"),
				rfield.NewI32Field("flags", 1),
				rfield.NewStringField("trace_state", "abc"),
				rfield.NewBinaryField("parent_span_id", []byte("123")),
				rfield.NewBinaryField("span_id", []byte("123")),
			}),
			rfield.NewStruct([]*rfield.Field{
				rfield.NewU64Field("start_time", 123),
				rfield.NewU64Field("end_time", 124),
				rfield.NewStringField("name", "bla"),
				rfield.NewI32Field("flags", 1),
				rfield.NewStringField("trace_state", "abc"),
				rfield.NewBinaryField("span_id", []byte("123")),
			}),
			rfield.NewStruct([]*rfield.Field{
				rfield.NewU64Field("start_time", 123),
				rfield.NewU64Field("end_time", 124),
				rfield.NewStringField("name", "bla"),
				rfield.NewI32Field("flags", 1),
				rfield.NewStringField("trace_state", "abc"),
				rfield.NewBinaryField("span_id", []byte("123")),
				rfield.NewListField("events", rfield.List{
					Values: []rfield.Value{
						rfield.NewStruct([]*rfield.Field{
							rfield.NewStringField("a", "a"),
							rfield.NewStringField("b", "b"),
							rfield.NewU64Field("ts", 12),
							rfield.NewStringField("c", "c"),
						}),
						rfield.NewStruct([]*rfield.Field{
							rfield.NewStringField("a", "a"),
							rfield.NewStringField("b", "b"),
							rfield.NewStringField("c", "c"),
						}),
						rfield.NewStruct([]*rfield.Field{
							rfield.NewStringField("a", "a"),
							rfield.NewU64Field("ts", 12),
							rfield.NewStringField("d", "d"),
						}),
					},
				}),
			}),
		},
	}
	listType := list.DataType()
	sig := rfield.DataTypeSignature(listType)
	if sig != "[{end_time:U64,events:[{a:Str,b:Str,c:Str,d:Str,ts:U64}],flags:I32,name:Str,parent_span_id:Bin,span_id:Bin,start_time:U64,trace_state:Str}]" {
		t.Errorf("invalid signature")
	}
}
