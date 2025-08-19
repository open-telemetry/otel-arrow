/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package datagen

import (
	"fmt"

	"github.com/brianvoe/gofakeit/v6"
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

// RandomAttributes returns a random set of attributes. The number of attributes
// is random [0,10). The type and value of each attribute is also random.
func (te TestEntropy) RandomAttributes() pcommon.Map {
	count := te.rng.Intn(10)

	attrs := pcommon.NewMap()

	for i := 0; i < count; i++ {
		switch te.rng.Intn(3) {
		case 0:
			attrs.PutStr(fmt.Sprintf("attr_%d", i), gofakeit.LoremIpsumWord())
		case 1:
			attrs.PutInt(fmt.Sprintf("attr_%d", i), te.rng.Int63())
		case 2:
			attrs.PutDouble(fmt.Sprintf("attr_%d", i), te.rng.Float64())
		case 3:
			attrs.PutBool(fmt.Sprintf("attr_%d", i), te.rng.Intn(2) == 0)
		case 4:
			attrs.PutEmpty(fmt.Sprintf("attr_%d", i))
		case 5:
			attrs.PutEmptyBytes(fmt.Sprintf("attr_%d", i)).FromRaw([]byte(gofakeit.LoremIpsumWord()))
		case 6:
			vMap := attrs.PutEmptyMap(fmt.Sprintf("attr_%d", i))
			vMap.PutInt("int", te.rng.Int63())
			vMap.PutStr("str", gofakeit.LoremIpsumWord())
		case 7:
			vSlice := attrs.PutEmptySlice(fmt.Sprintf("attr_%d", i))
			vSlice.AppendEmpty().SetBool(te.rng.Intn(2) == 0)
			vSlice.AppendEmpty().SetDouble(te.rng.Float64())
			vSlice.AppendEmpty().SetInt(te.rng.Int63())
		}
	}

	return attrs
}

func (te TestEntropy) NewStandardAttributes() pcommon.Map {
	return te.shuffleAttrs(
		func(attrs Attrs) { attrs.PutStr("hostname", pick(te, HOSTNAMES)) },
		func(attrs Attrs) { attrs.PutBool("up", pick(te, UPS)) },
		func(attrs Attrs) { attrs.PutInt("status", pick(te, STATUS)) },
		func(attrs Attrs) { attrs.PutStr("version", pick(te, VERSIONS)) },
		func(attrs Attrs) {
			attrs.PutEmpty("group_id").
				SetEmptyBytes().
				FromRaw([]byte(pick(te, GroupIds)))
		},
	)
}

func (te TestEntropy) NewSingleResourceAttributes() []pcommon.Map {
	return []pcommon.Map{
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host1.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.1") },
			func(attrs Attrs) { attrs.PutBool("up", true) },
			func(attrs Attrs) { attrs.PutInt("status", 200) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.0) },
		),
	}
}

func (te TestEntropy) NewStandardResourceAttributes() []pcommon.Map {
	// 3 resources with attributes and 1 resource with no attributes
	return []pcommon.Map{
		pcommon.NewMap(), // No attributes
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host1.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.1") },
			func(attrs Attrs) { attrs.PutBool("up", true) },
			func(attrs Attrs) { attrs.PutInt("status", 200) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.0) },
			func(attrs Attrs) { attrs.PutStr("unique1", "uv1") },
		),
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host2.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.2") },
			func(attrs Attrs) { attrs.PutBool("up", true) },
			func(attrs Attrs) { attrs.PutInt("status", 200) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.0) },
			func(attrs Attrs) { attrs.PutStr("unique2", "uv2") },
		),
		te.shuffleAttrs(
			func(attrs Attrs) { attrs.PutStr("hostname", "host3.mydomain.com") },
			func(attrs Attrs) { attrs.PutStr("ip", "192.168.0.3") },
			func(attrs Attrs) { attrs.PutBool("up", false) },
			func(attrs Attrs) { attrs.PutInt("status", 500) },
			func(attrs Attrs) { attrs.PutDouble("version", 1.5) },
			func(attrs Attrs) { attrs.PutStr("unique3", "uv3") },
		),
	}
}

func (te TestEntropy) NewRandomResourceAttributes(count int) []pcommon.Map {
	attrs := make([]pcommon.Map, count)

	// empty attributes
	attrs[0] = pcommon.NewMap()

	// attributes with random values
	for i := 1; i < count; i++ {
		attrs[i] = te.RandomAttributes()
	}
	return attrs
}

func (te TestEntropy) NewSingleInstrumentationScopes() []pcommon.InstrumentationScope {
	s1 := pcommon.NewInstrumentationScope()
	s1.SetName("fake_generator")
	s1.SetVersion("1.0.0")

	return []pcommon.InstrumentationScope{s1}
}

func (te TestEntropy) NewStandardInstrumentationScopes() []pcommon.InstrumentationScope {
	s1 := pcommon.NewInstrumentationScope()
	s1.SetName("fake_generator")
	s1.SetVersion("1.0.0")

	s2 := pcommon.NewInstrumentationScope()
	s2.SetName("fake_generator")
	s2.SetVersion("1.0.1")

	empty := pcommon.NewInstrumentationScope()

	// 2 instrumentation scopes and 1 empty
	return []pcommon.InstrumentationScope{s1, s2, empty}
}

func (te TestEntropy) NewRandomInstrumentationScopes(count int) []pcommon.InstrumentationScope {
	scopes := make([]pcommon.InstrumentationScope, count)

	// empty scope
	scopes[0] = pcommon.NewInstrumentationScope()

	// scopes with random attributes
	for i := 1; i < count; i++ {
		scope := pcommon.NewInstrumentationScope()
		scope.SetName("fake_generator")
		scope.SetVersion(fmt.Sprintf("1.0.%d", i))
		if i%2 == 0 {
			te.RandomAttributes().CopyTo(scope.Attributes())
		}
		scopes[i] = scope
	}

	return scopes
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
