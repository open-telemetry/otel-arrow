package rbb

import "otel-arrow-adapter/pkg/rbb/value"

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
