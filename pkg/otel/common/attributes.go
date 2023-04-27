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

package common

import (
	"sort"

	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/pdata"
)

// SharedAttributes is a data structure representing the shared attributes of a set of metrics.
type SharedAttributes struct {
	Attributes map[string]pcommon.Value
}

// NewSharedAttributesFrom creates a new SharedAttributes from a [pcommon.Map] of attributes.
func NewSharedAttributesFrom(attrs pcommon.Map) *SharedAttributes {
	attributes := make(map[string]pcommon.Value)

	attrs.Range(func(k string, v pcommon.Value) bool {
		attributes[k] = v
		return true
	})

	return &SharedAttributes{
		Attributes: attributes,
	}
}

// CopyTo copies the current SharedAttributes to a [pcommon.Map] of attributes.
// The attributes are sorted by key before being copied to make the tests
// deterministic.
func (sa *SharedAttributes) CopyTo(attrs pcommon.Map) {
	for k, v := range sa.Attributes {
		v.CopyTo(attrs.PutEmpty(k))
	}
}

func (sa *SharedAttributes) Clone() *SharedAttributes {
	attributes := make(map[string]pcommon.Value)
	for k, v := range sa.Attributes {
		attributes[k] = v
	}

	return &SharedAttributes{
		Attributes: attributes,
	}
}

// IntersectWithMap intersects the current SharedAttributes with a [pcommon.Map] of attributes
// and returns the number of shared attributes after the intersection.
func (sa *SharedAttributes) IntersectWithMap(attrs pcommon.Map) int {
	for k, v := range sa.Attributes {
		if otherV, ok := attrs.Get(k); ok {
			if !pdata.ValuesEqual(v, otherV) {
				delete(sa.Attributes, k)
			}
		} else {
			delete(sa.Attributes, k)
		}
	}

	return len(sa.Attributes)
}

func (sa *SharedAttributes) IntersectWith(other *SharedAttributes) int {
	for k, v := range sa.Attributes {
		if otherV, ok := other.Attributes[k]; ok {
			if !pdata.ValuesEqual(v, otherV) {
				delete(sa.Attributes, k)
			}
		} else {
			delete(sa.Attributes, k)
		}
	}
	return len(sa.Attributes)
}

// Has returns true if the current SharedAttributes has the given attribute.
func (sa *SharedAttributes) Has(k string) bool {
	_, ok := sa.Attributes[k]
	return ok
}

// Len returns the number of attributes in the current SharedAttributes.
func (sa *SharedAttributes) Len() int {
	return len(sa.Attributes)
}

type (
	AttributeEntry struct {
		Key   string
		Value pcommon.Value
	}

	AttributeEntries []AttributeEntry
)

var _ sort.Interface = AttributeEntries{}

func (ae AttributeEntries) Len() int {
	return len(ae)
}

func (ae AttributeEntries) Swap(i, j int) {
	ae[i], ae[j] = ae[j], ae[i]
}

func (ae AttributeEntries) Less(i, j int) bool {
	return ae[i].Key < ae[j].Key
}
