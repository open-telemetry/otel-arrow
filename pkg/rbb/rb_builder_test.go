package rbb

import (
	"github.com/davecgh/go-spew/spew"
	"otel-arrow-adapter/pkg/rbb/value"
	"testing"
)

func TestNewRecordBatchBuilderWithRecord(t *testing.T) {
	config := NewDefaultConfig()
	record := Record{
		fields: []value.Field{
			{Name: "b", Value: &value.String{Value: "bla"}},
			{Name: "a", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "e", Value: &value.String{Value: ""}},
					{Name: "b", Value: &value.String{Value: ""}},
					{Name: "c", Value: &value.Struct{
						Fields: []value.Field{
							{Name: "x", Value: &value.String{Value: ""}},
							{Name: "t", Value: &value.String{Value: ""}},
							{Name: "z", Value: &value.List{
								Values: []value.Value{
									&value.I64{Value: 1},
									&value.I64{Value: 2},
								},
							}},
							{Name: "a", Value: &value.List{
								Values: []value.Value{
									&value.Struct{
										Fields: []value.Field{
											{Name: "f2_3_4_2", Value: &value.String{Value: "f2_3_4_2"}},
											{Name: "f2_3_4_1", Value: &value.String{Value: "f2_3_4_1"}},
										},
									},
								},
							}},
						},
					}},
				},
			}},
		},
	}
	record.Normalize()

	rbb := NewRecordBatchBuilderWithRecord(record, config)
	spew.Dump(rbb)
}
