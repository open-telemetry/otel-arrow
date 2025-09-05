/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow

import (
	"math"
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	acommon "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema"
	builder "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/config"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/constants"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/internal"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/stats"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16, 0.0)
var ProducerStats = stats.NewProducerStats()

func TestAttributesBuilder(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.Attributes, Type: AttributesDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig, ProducerStats, nil)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		ab := NewAttributesBuilder(rBuilder.MapBuilder(constants.Attributes))

		err := ab.Append(internal.Attrs1())
		require.NoError(t, err)

		err = ab.Append(internal.Attrs2())
		require.NoError(t, err)

		err = ab.Append(internal.Attrs3())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}]}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}]}
,{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}]}
]`

	require.JSONEq(t, expected, string(json))
}
