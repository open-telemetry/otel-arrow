package rbb

import (
	"github.com/apache/arrow/go/arrow"
	"otel-arrow-adapter/pkg/rbb/stats"
)

type StructColumn struct {
	Name    string
	Type    arrow.DataType
	Columns Columns
}

func (c *StructColumn) DictionaryStats() []*stats.DictionaryStats {
	return c.Columns.DictionaryStats()
}
