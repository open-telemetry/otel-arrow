package rbb

import (
	"fmt"
	"otel-arrow-adapter/pkg/rbb/value"
)

func GenSimpleRecord(ts int64) *Record {
	return &Record{
		fields: []value.Field{
			{Name: "ts", Value: &value.I64{Value: ts}},
			{Name: "a", Value: &value.String{Value: "a"}},
			{Name: "b", Value: &value.String{Value: "b"}},
			{Name: "c", Value: &value.String{Value: "c"}},
		},
	}
}

func GenComplexRecord(ts int64) *Record {
	return &Record{
		fields: []value.Field{
			{Name: "ts", Value: &value.I64{Value: ts}},
			{Name: "a", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "b", Value: &value.String{Value: "b"}},
					{Name: "c", Value: &value.String{Value: "c"}},
				},
			}},
			{Name: "b", Value: &value.String{Value: "b"}},
			{Name: "c", Value: &value.String{Value: "c"}},
		},
	}
}

func GenRecord(ts int64, value_a, value_b, value_c int) *Record {
	return &Record{
		fields: []value.Field{
			{Name: "ts", Value: &value.I64{Value: ts}},
			{Name: "c", Value: &value.String{Value: fmt.Sprintf("c___%d", value_c)}},
			{Name: "a", Value: &value.String{Value: fmt.Sprintf("a_%d", value_a)}},
			{Name: "b", Value: &value.String{Value: fmt.Sprintf("b__%d", value_b)}},
		},
	}
}
