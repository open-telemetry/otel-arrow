package rbb

import (
	"github.com/davecgh/go-spew/spew"
	"testing"
)

func TestNewRecordBatchBuilderWithRecord(t *testing.T) {
	config := NewDefaultConfig()
	record := Record{
		fields: []Field{
			{Name: "b", Value: &String{Value: "bla"}},
			{Name: "a", Value: &Struct{
				fields: []Field{
					{Name: "e", Value: &String{Value: ""}},
					{Name: "b", Value: &String{Value: ""}},
					{Name: "c", Value: &Struct{
						fields: []Field{
							{Name: "x", Value: &String{Value: ""}},
							{Name: "t", Value: &String{Value: ""}},
							{Name: "z", Value: &List{
								values: []Value{
									&I64{Value: 1},
									&I64{Value: 2},
								},
							}},
							{Name: "a", Value: &List{
								values: []Value{
									&Struct{
										fields: []Field{
											{Name: "f2_3_4_2", Value: &String{Value: "f2_3_4_2"}},
											{Name: "f2_3_4_1", Value: &String{Value: "f2_3_4_1"}},
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
