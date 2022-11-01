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

package assert

import (
	"encoding/json"
	"testing"

	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

func TestEquiv(t *testing.T) {
	t.Parallel()

	traces := ptrace.NewTraces()
	rs := traces.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("foo1", "bar")
	rs.Resource().Attributes().PutInt("foo2", 123)
	rs.Resource().Attributes().PutDouble("foo3", 123.0)
	rs.Resource().Attributes().PutBool("foo4", true)
	rs.SetSchemaUrl("http://foo.bar")

	expectedTraces := []json.Marshaler{
		ptraceotlp.NewRequestFromTraces(traces),
	}

	actualTraces := []json.Marshaler{
		ptraceotlp.NewRequestFromTraces(traces),
		ptraceotlp.NewRequestFromTraces(traces),
	}
	Equiv(t, expectedTraces, actualTraces)

	traces = ptrace.NewTraces()
	rs = traces.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("foo", "bar")
	rs.Resource().Attributes().PutStr("baz", "qux")
	rs.SetSchemaUrl("http://foo.bar")
	actualTraces = []json.Marshaler{
		ptraceotlp.NewRequestFromTraces(traces),
	}
	NotEquiv(t, expectedTraces, actualTraces)
}
