/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package schema

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/stretchr/testify/require"

	transform "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/transform"
)

// histogramBucketCountsField mirrors the histogram bucket_counts prototype
// field: an optional, nullable list of uint64 values (see
// pkg/otel/metrics/arrow/histogram_dp.go). The element field of an
// arrow.ListOf list is nullable, so its transform node starts with a
// NoField transform and is only un-dropped by the first non-zero element
// append.
func histogramBucketCountsField() arrow.Field {
	return arrow.Field{
		Name:     "bucket_counts",
		Type:     arrow.ListOf(arrow.PrimitiveTypes.Uint64),
		Nullable: true,
	}
}

// Scenario: A list prototype field is visible (e.g. un-dropped by
// ListBuilder.Reserve, as happens for a non-empty histogram bucket_counts
// list) while its element transform node is still dropped, which is the
// case when every element append is zero and therefore not sufficient to
// un-drop the element node (see #4397).
// Guarantees: NewFieldFrom returns nil, i.e. drops the list field instead
// of dereferencing the nil element field, matching the existing behavior
// of the map branch for a dropped key or item field.
func TestNewFieldFromDroppedListElementDropsListField(t *testing.T) {
	elemNode := &TransformNode{
		name:       "item",
		path:       "bucket_counts.item",
		transforms: []FieldTransform{&transform.NoField{}},
	}
	listNode := &TransformNode{
		name:     "bucket_counts",
		path:     "bucket_counts",
		Children: []*TransformNode{elemNode},
	}

	protoField := histogramBucketCountsField()
	field := NewFieldFrom(&protoField, listNode)
	require.Nil(t, field)
}

// Scenario: A list prototype field and its element transform node are both
// visible (e.g. after ListBuilder.Reserve un-dropped the element node
// alongside the list node).
// Guarantees: NewFieldFrom returns the list field with its uint64 element
// type preserved, so a list carrying only zero values stays in the schema
// instead of being dropped.
func TestNewFieldFromVisibleListElementKeepsListField(t *testing.T) {
	elemNode := &TransformNode{
		name: "item",
		path: "bucket_counts.item",
	}
	listNode := &TransformNode{
		name:     "bucket_counts",
		path:     "bucket_counts",
		Children: []*TransformNode{elemNode},
	}

	protoField := histogramBucketCountsField()
	field := NewFieldFrom(&protoField, listNode)
	require.NotNil(t, field)
	listType, ok := field.Type.(*arrow.ListType)
	require.True(t, ok)
	require.Equal(t, arrow.PrimitiveTypes.Uint64, listType.Elem())
}
