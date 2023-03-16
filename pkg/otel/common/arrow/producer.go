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

package arrow

import (
	"fmt"
	"sort"
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"
)

type (
	mapEntry struct {
		key   string
		value pcommon.Value
	}

	mapEntries []mapEntry
)

var _ sort.Interface = mapEntries{}

func (me mapEntries) Len() int {
	return len(me)
}

func (me mapEntries) Swap(i, j int) {
	me[i], me[j] = me[j], me[i]
}

func (me mapEntries) Less(i, j int) bool {
	return me[i].key < me[j].key
}

func AttributesId(attrs pcommon.Map) string {
	tmp := make(mapEntries, 0, attrs.Len())
	attrs.Range(func(k string, v pcommon.Value) bool {
		tmp = append(tmp, mapEntry{
			key:   k,
			value: v,
		})
		return true
	})
	sort.Stable(tmp)

	var attrsId strings.Builder
	attrsId.WriteString("{")
	for _, e := range tmp {
		if attrsId.Len() > 1 {
			attrsId.WriteString(",")
		}
		attrsId.WriteString(e.key)
		attrsId.WriteString(":")
		attrsId.WriteString(ValueID(e.value))
	}
	attrsId.WriteString("}")
	return attrsId.String()
}

func ResourceID(r pcommon.Resource) string {
	return AttributesId(r.Attributes()) + "|" + fmt.Sprintf("dac:%d", r.DroppedAttributesCount())
}

func ScopeID(is pcommon.InstrumentationScope) string {
	return "name:" + is.Name() + "|version:" + is.Version() + "|" + AttributesId(is.Attributes()) + "|" + fmt.Sprintf("dac:%d", is.DroppedAttributesCount())
}

func ValueID(v pcommon.Value) string {
	switch v.Type() {
	case pcommon.ValueTypeStr:
		return v.Str()
	case pcommon.ValueTypeInt:
		return fmt.Sprintf("%d", v.Int())
	case pcommon.ValueTypeDouble:
		return fmt.Sprintf("%f", v.Double())
	case pcommon.ValueTypeBool:
		return fmt.Sprintf("%t", v.Bool())
	case pcommon.ValueTypeMap:
		return AttributesId(v.Map())
	case pcommon.ValueTypeBytes:
		return fmt.Sprintf("%x", v.Bytes().AsRaw())
	case pcommon.ValueTypeSlice:
		values := v.Slice()
		valueID := "["
		for i := 0; i < values.Len(); i++ {
			if len(valueID) > 1 {
				valueID += ","
			}
			valueID += ValueID(values.At(i))
		}
		valueID += "]"
		return valueID
	case pcommon.ValueTypeEmpty:
		return ""
	default:
		// includes pcommon.ValueTypeEmpty
		panic("unsupported value type")
	}
}
