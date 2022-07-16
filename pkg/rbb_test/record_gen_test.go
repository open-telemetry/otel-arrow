package rbb_test

import (
	"fmt"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/rfield"
)

func GenSimpleRecord(ts int64) *rbb.Record {
	record := rbb.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("a", "a")
	record.StringField("b", "b")
	record.StringField("c", "c")
	return record
}

func GenComplexRecord(ts int64) *rbb.Record {
	record := rbb.NewRecord()
	record.I64Field("ts", ts)
	record.StructField("a", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "b", Value: &rfield.String{Value: "b"}},
			{Name: "c", Value: &rfield.String{Value: "c"}},
		},
	})
	record.StringField("b", "b")
	record.StringField("c", "c")
	return record
}

func GenRecord(ts int64, value_a, value_b, value_c int) *rbb.Record {
	record := rbb.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("c", fmt.Sprintf("c___%d", value_c))
	record.StringField("a", fmt.Sprintf("a_%d", value_a))
	record.StringField("b", fmt.Sprintf("b__%d", value_b))
	record.StructField("d", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "a", Value: &rfield.String{Value: fmt.Sprintf("a_%d", value_a)}},
			{Name: "b", Value: &rfield.String{Value: fmt.Sprintf("b_%d", value_b)}},
		},
	})
	return record
}
