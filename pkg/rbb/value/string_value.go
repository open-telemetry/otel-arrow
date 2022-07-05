package value

import (
	"otel-arrow-adapter/pkg/rbb/config"
	"otel-arrow-adapter/pkg/rbb/stats"
)

// StringColumn is a column of optional string values.
type StringColumn struct {
	Name             string
	Config           *config.DictionaryConfig
	FieldPath        []int
	DictId           int
	Dictionary       map[string]bool
	Data             []*string
	TotalValueLength int
	TotalRowCount    int
}

func (c *StringColumn) Push(value *string) {
	// Maintains a dictionary of unique values
	if c.Dictionary != nil {
		if value != nil {
			if _, ok := c.Dictionary[*value]; !ok {
				c.Dictionary[*value] = true
				if len(c.Dictionary) > c.Config.MaxCard {
					c.Dictionary = nil
				}
			}
		}
	}

	c.TotalRowCount++
	if value != nil {
		c.TotalValueLength += len(*value)
	}
	c.Data = append(c.Data, value)
}

func (c *StringColumn) DictionaryStats() *stats.DictionaryStats {
	if c.Dictionary != nil {
		return &stats.DictionaryStats{
			Path:           c.FieldPath,
			Cardinality:    c.DictionaryLen(),
			AvgEntryLength: c.AvgValueLength(),
			TotalEntry:     c.TotalRowCount,
		}
	}
	return nil
}

func (c *StringColumn) DictionaryLen() int {
	if c.Dictionary != nil {
		return len(c.Dictionary)
	} else {
		return 0
	}
}

func (c *StringColumn) AvgValueLength() float64 {
	if c.TotalValueLength == 0 || c.TotalRowCount == 0 {
		return 0.0
	}
	return float64(c.TotalValueLength) / float64(c.TotalRowCount)
}
