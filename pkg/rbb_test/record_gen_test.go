package rbb_test

import (
	"fmt"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/value"
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
	record.StructField("a", value.Struct{
		Fields: []value.Field{
			{Name: "b", Value: &value.String{Value: "b"}},
			{Name: "c", Value: &value.String{Value: "c"}},
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
	return record
}

func GenBasicRecord(ts int64) *rbb.Record {
	record := rbb.NewRecord()
	record.I64Field("ts", ts)
	record.BoolField("c", true)
	record.I32Field("a", 12)
	record.U8Field("b", 100)
	return record
}
