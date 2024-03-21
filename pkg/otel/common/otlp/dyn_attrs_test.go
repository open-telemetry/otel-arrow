/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package otlp

import (
	"testing"

	"github.com/apache/arrow/go/v14/arrow/memory"
	"github.com/stretchr/testify/assert"
	"go.opentelemetry.io/collector/pdata/pcommon"

	carrow "github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/internal"
)

type ExpectedAttributes struct {
	ParentId uint32
	Attrs    pcommon.Map
}

func TestDynAttrsFromRecord(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	dynAttrs := carrow.NewDynAttrsBuilder(carrow.PayloadTypes.SpanAttrs, pool)
	defer dynAttrs.Release()

	attrCount := 1000
	attrNames := []string{
		"service", "operation", "span.kind", "http.status_code", "http.url",
		"http.method", "http.host", "http.target", "http.flavor", "http.user_agent", "http.route",
		"http.server_name", "http.scheme", "net.host.name", "net.host.port", "net.peer.name",
		"net.peer.port", "net.transport", "db.system", "db.name", "db.statement", "db.operation",
	}
	expected := make(map[uint32]ExpectedAttributes)

	for i := 0; i < attrCount; i++ {
		parentID := uint32(i)
		attrs := internal.RandAttrs(attrNames)
		expected[parentID] = ExpectedAttributes{parentID, attrs}
		err := dynAttrs.Append(parentID, attrs)
		assert.NoError(t, err)
	}

	record, err := dynAttrs.Build(nil)
	assert.NoError(t, err)

	store, err := DynAttrsStoreFromRecord(record)
	assert.NoError(t, err)

	for _, attrs := range expected {
		actual, found := store.Attributes(attrs.ParentId)
		assert.True(t, found)
		AttributesEqual(t, attrs.Attrs, actual)
	}
}

func AttributesEqual(t *testing.T, expected pcommon.Map, actual pcommon.Map) {
	assert.NotNil(t, actual, "actual attributes is nil")
	expectedLen := expected.Len()
	actualLen := actual.Len()
	assert.Equal(t, expectedLen, actualLen, "expected and actual attributes have different lengths")
	expected.Range(func(k string, v pcommon.Value) bool {
		actualV, found := actual.Get(k)
		assert.True(t, found, "expected attribute %s not found in actual", k)
		assert.Equal(t, v, actualV, "expected attribute %s has different value in actual", k)
		return true
	})
}
