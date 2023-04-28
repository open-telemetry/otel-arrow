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
	"errors"
	"testing"

	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
)

func TestLimitedAllocator(t *testing.T) {
	const boundary = 1000000
	check := memory.NewCheckedAllocator(memory.NewGoAllocator())
	limit := NewLimitedAllocator(check, boundary)

	var b []byte
	check.AssertSize(t, 0)

	b = limit.Allocate(boundary)
	require.NotNil(t, b)

	var capture interface{}

	func() {
		defer func() {
			if ret := recover(); ret != nil {
				capture = ret
			}
		}()

		_ = limit.Allocate(1)
		panic("not reached")
	}()
	require.NotNil(t, capture)
	require.True(t, errors.Is(capture.(error), LimitError{}))
	require.Equal(t, "allocation size 1 exceeds limit 1000000 (in-use=1000000)", capture.(error).Error())

	limit.Free(b)

	check.AssertSize(t, 0)
}
