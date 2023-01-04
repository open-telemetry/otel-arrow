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

package datagen

import (
	"go.opentelemetry.io/collector/pdata/pcommon"
)

var HOSTNAMES = []string{"host1.mydomain.com", "host2.org", "host3.thedomain.edu", "host4.gov", "host5.retailer.com"}
var UPS = []bool{true, false}
var STATUS = []int64{200, 300, 400, 404, 500, 503}
var VERSIONS = []string{"1.0.0", "1.0.2", "2.0", "1.9.9"}
var GroupIds = []string{"group1", "group2", "group3", "group4", "group5"}

type Attrs = pcommon.Map
type AttrFunc func(Attrs)

func pick[N any](entropy TestEntropy, from []N) N {
	return from[entropy.rng.Intn(len(from))]
}

func (te TestEntropy) shuffleAttrs(fs ...func(Attrs)) pcommon.Map {
	attrs := pcommon.NewMap()
	te.rng.Shuffle(len(fs), func(i, j int) {
		fs[i], fs[j] = fs[j], fs[i]
	})
	for _, f := range fs {
		f(attrs)
	}
	return attrs
}

func (te TestEntropy) NewStandardAttributes() pcommon.Map {
	return te.shuffleAttrs(
		func(attrs Attrs) { attrs.PutStr("hostname", pick(te, HOSTNAMES)) },
		func(attrs Attrs) { attrs.PutBool("up", pick(te, UPS)) },
		func(attrs Attrs) { attrs.PutInt("status", pick(te, STATUS)) },
		func(attrs Attrs) { attrs.PutStr("version", pick(te, VERSIONS)) },

		// ToDo reintroduce tags_arrays once list are fully supported
		//{
		//	Key: "tags_array",
		//	Value: &pcommon.AnyValue{Value: &pcommon.AnyValue_ArrayValue{ArrayValue: &pcommon.ArrayValue{
		//		Values: []pcommon.Value{
		//			{Value: &pcommon.AnyValue_StringValue{StringValue: "tag1"}},
		//			{Value: &pcommon.AnyValue_StringValue{StringValue: "tag2"}},
		//		},
		//	}}},
		//},
		//{
		//	Key: "tags_kv_list",
		//	Value: &pcommon.AnyValue{Value: &pcommon.AnyValue_KvlistValue{
		//		KvlistValue: &pcommon.KeyValueList{
		//			Values: pcommon.Map{
		//				{
		//					Key:   "state",
		//					Value: &pcommon.AnyValue{Value: &pcommon.AnyValue_StringValue{StringValue: STATES[rand.Intn(len(STATES))]}},
		//				},
		//				{
		//					Key:   "duration",
		//					Value: &pcommon.AnyValue{Value: &pcommon.AnyValue_IntValue{IntValue: int64(rand.Intn(100))}},
		//				},
		//			},
		//		},
		//	}},
		//},

		func(attrs Attrs) {
			attrs.PutEmpty("group_id").
				SetEmptyBytes().
				FromRaw([]byte(pick(te, GroupIds)))
		},
	)
}

func (te TestEntropy) NewStandardResourceAttributes() []pcommon.Map {
	return []pcommon.Map{
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host1.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.1") },
			func(attrs Attrs) { attrs.PutBool("up", true) },
			func(attrs Attrs) { attrs.PutInt("status", 200) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.0) },
		),
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host2.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.2") },
			func(attrs Attrs) { attrs.PutBool("up", true) },
			func(attrs Attrs) { attrs.PutInt("status", 200) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.0) },
		),
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host3.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.3") },
			func(attrs Attrs) { attrs.PutBool("up", false) },
			func(attrs Attrs) { attrs.PutInt("status", 500) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.5) },
		),
	}
}

func (te TestEntropy) NewStandardInstrumentationScopes() []pcommon.InstrumentationScope {
	s1 := pcommon.NewInstrumentationScope()
	s1.SetName("fake_generator")
	s1.SetVersion("1.0.0")

	s2 := pcommon.NewInstrumentationScope()
	s2.SetName("fake_generator")
	s2.SetVersion("1.0.1")

	return []pcommon.InstrumentationScope{s1, s2}
}

func (te TestEntropy) NewStandardSpanEventAttributes() pcommon.Map {
	return te.shuffleAttrs(
		func(attrs Attrs) { attrs.PutStr("hostname", pick(te, HOSTNAMES)) },
		func(attrs Attrs) { attrs.PutStr("version", pick(te, VERSIONS)) },
		func(attrs Attrs) { attrs.PutBool("up", pick(te, UPS)) },
		func(attrs Attrs) { attrs.PutInt("status", pick(te, STATUS)) },
	)
}

func (te TestEntropy) NewStandardSpanLinkAttributes() pcommon.Map {
	return te.shuffleAttrs(
		func(attrs Attrs) { attrs.PutStr("hostname", pick(te, HOSTNAMES)) },
		func(attrs Attrs) { attrs.PutBool("up", pick(te, UPS)) },
		func(attrs Attrs) { attrs.PutInt("status", pick(te, STATUS)) },
	)
}
