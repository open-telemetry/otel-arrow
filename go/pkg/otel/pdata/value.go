/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package pdata

// pdata utilities

import "go.opentelemetry.io/collector/pdata/pcommon"

// ValuesEqual compares two pdata.Values for equality.
func ValuesEqual(v, av pcommon.Value) bool {
	// Note: copied from the function removed in
	// https://github.com/open-telemetry/opentelemetry-collector/pull/6860
	if v.Type() != av.Type() {
		return false
	}

	switch v.Type() {
	case pcommon.ValueTypeEmpty:
		return true
	case pcommon.ValueTypeStr:
		return v.Str() == av.Str()
	case pcommon.ValueTypeBool:
		return v.Bool() == av.Bool()
	case pcommon.ValueTypeInt:
		return v.Int() == av.Int()
	case pcommon.ValueTypeDouble:
		return v.Double() == av.Double()
	case pcommon.ValueTypeSlice:
		vs := v.Slice()
		avs := av.Slice()
		if vs.Len() != avs.Len() {
			return false
		}

		for i := 0; i < vs.Len(); i++ {
			if !ValuesEqual(vs.At(i), avs.At(i)) {
				return false
			}
		}
		return true
	case pcommon.ValueTypeMap:
		vm := v.Map()
		avm := av.Map()
		if vm.Len() != avm.Len() {
			return false
		}

		ne := false
		vm.Range(func(k string, vv pcommon.Value) bool {
			// Note: This is not an equivalent function as there is no way
			// to look up values by position; as long as the sender respects
			// Map semantics, this will get the same result and is
			// unfortunately an O(N) lookup.
			if av, ok := avm.Get(k); !ok || !ValuesEqual(vv, av) {
				ne = true
				return false
			}
			return true
		})
		return !ne
	case pcommon.ValueTypeBytes:
		vb := v.Bytes()
		avb := av.Bytes()
		if vb.Len() != avb.Len() {
			return false
		}
		for i := 0; i < vb.Len(); i++ {
			if vb.At(i) != avb.At(i) {
				return false
			}
		}
		return true
	default:
		return false
	}
}
