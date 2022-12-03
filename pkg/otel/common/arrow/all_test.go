package arrow

import (
	"testing"

	"github.com/apache/arrow/go/v10/arrow/memory"
	"github.com/stretchr/testify/require"

	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestAttributesBuilder(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	ab := NewAttributesBuilder(pool)

	if err := ab.Append(internal.Attrs1()); err != nil {
		t.Fatal(err)
	}
	if err := ab.Append(internal.Attrs2()); err != nil {
		t.Fatal(err)
	}
	if err := ab.Append(internal.Attrs3()); err != nil {
		t.Fatal(err)
	}
	arr, err := ab.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}]
,[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}]
,[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}]
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeBuilder(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewScopeBuilder(pool)

	if err := sb.Append(internal.Scope1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(internal.Scope2()); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceBuilder(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rb := NewResourceBuilder(pool)

	if err := rb.Append(internal.Resource1()); err != nil {
		t.Fatal(err)
	}
	if err := rb.Append(internal.Resource2()); err != nil {
		t.Fatal(err)
	}
	arr, err := rb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1}
]`

	require.JSONEq(t, expected, string(json))
}
