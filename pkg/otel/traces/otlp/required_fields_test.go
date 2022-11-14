package otlp

import (
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
)

func TestRequiredField(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	ToDo implement this test
}
