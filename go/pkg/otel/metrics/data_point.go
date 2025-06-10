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

package metrics

import (
	"encoding/binary"
	"math"
	"sort"

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

type DataPoint interface {
	Attributes() pcommon.Map
	Timestamp() pcommon.Timestamp
	StartTimestamp() pcommon.Timestamp
}

func DataPointSig[N DataPoint](dataPoint N, multivariateKey string) []byte {
	sig := make([]byte, 16, 128)

	// Serialize times and attributes to build the signature.
	binary.LittleEndian.PutUint64(sig[0:], uint64(dataPoint.StartTimestamp()))
	binary.LittleEndian.PutUint64(sig[8:], uint64(dataPoint.Timestamp()))
	return MapSig(sig, dataPoint.Attributes(), multivariateKey)
}

func MapSig(sig []byte, kvs pcommon.Map, multivariateKey string) []byte {
	// Sort KeyValue slice by key to make the signature deterministic.
	tmp := make(mapEntries, 0, kvs.Len())
	kvs.Range(func(k string, v pcommon.Value) bool {
		// Skip the multivariate key.
		if k == multivariateKey {
			return true
		}

		tmp = append(tmp, mapEntry{
			key:   k,
			value: v,
		})
		return true
	})
	sort.Stable(tmp)

	for _, me := range tmp {
		// Serialize attribute name
		sig = append(sig, []byte(me.key)...)

		// Serialize attribute value
		sig = ValueSig(sig, me.value)
	}

	return sig
}

func SliceSig(sig []byte, sl pcommon.Slice) []byte {
	for i := 0; i < sl.Len(); i++ {
		val := sl.At(i)

		sig = ValueSig(sig, val)
	}

	return sig
}

func ValueSig(sig []byte, value pcommon.Value) []byte {
	switch value.Type() {
	case pcommon.ValueTypeBool:
		sig = append(sig, BoolToByte(value.Bool()))
	case pcommon.ValueTypeInt:
		var buf [8]byte
		binary.LittleEndian.PutUint64(buf[:], uint64(value.Int()))
		sig = append(sig, buf[:]...)
	case pcommon.ValueTypeDouble:
		var buf [8]byte
		binary.LittleEndian.PutUint64(buf[:], math.Float64bits(value.Double()))
		sig = append(sig, buf[:]...)
	case pcommon.ValueTypeBytes:
		sig = append(sig, value.Bytes().AsRaw()...)
	case pcommon.ValueTypeStr:
		sig = append(sig, []byte(value.Str())...)
	case pcommon.ValueTypeSlice:
		sig = SliceSig(sig, value.Slice())
	case pcommon.ValueTypeMap:
		sig = MapSig(sig, value.Map(), "")
	case pcommon.ValueTypeEmpty:
		// Do nothing.
	default:
		panic("unsupported value type")
	}
	return sig
}

func BoolToByte(b bool) byte {
	if b {
		return 1
	}
	return 0
}
