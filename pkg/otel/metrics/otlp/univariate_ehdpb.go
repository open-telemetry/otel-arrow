package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type EHistogramDataPointBucketsIds struct {
	Id           int
	Offset       int
	BucketCounts int
}

func NewEHistogramDataPointBucketsIds(parentId int, parentDT *arrow.StructType) (*EHistogramDataPointBucketsIds, error) {
	offset, found := parentDT.FieldIdx(constants.EXP_HISTOGRAM_OFFSET)
	if !found {
		return nil, fmt.Errorf("missing field %q", constants.EXP_HISTOGRAM_OFFSET)
	}

	bucketCounts, found := parentDT.FieldIdx(constants.EXP_HISTOGRAM_BUCKET_COUNTS)
	if !found {
		return nil, fmt.Errorf("missing field %q", constants.EXP_HISTOGRAM_BUCKET_COUNTS)
	}

	return &EHistogramDataPointBucketsIds{
		Id:           parentId,
		Offset:       offset,
		BucketCounts: bucketCounts,
	}, nil
}

func AppendUnivariateEHistogramDataPointBucketsInto(dpBuckets pmetric.ExponentialHistogramDataPointBuckets, ehdp *array.Struct, ids *EHistogramDataPointBucketsIds, row int) error {
	if ehdp == nil {
		return nil
	}

	offsetArr := ehdp.Field(ids.Offset)
	if offsetArr != nil {
		if i32OffsetArr, ok := offsetArr.(*array.Int32); ok {
			dpBuckets.SetOffset(i32OffsetArr.Value(row))
		} else {
			return fmt.Errorf("field %q is not an int32", constants.EXP_HISTOGRAM_OFFSET)
		}
	}

	bucketCountsArr := ehdp.Field(ids.BucketCounts)
	if bucketCountsArr != nil {
		if i64BucketCountsArr, ok := bucketCountsArr.(*array.List); ok {
			start := int(i64BucketCountsArr.Offsets()[row])
			end := int(i64BucketCountsArr.Offsets()[row+1])
			values := i64BucketCountsArr.ListValues()

			if v, ok := values.(*array.Uint64); ok {
				dpbs := dpBuckets.BucketCounts()
				dpbs.EnsureCapacity(end - start)
				for i := start; i < end; i++ {
					dpbs.Append(v.Value(i))
				}
			}
		} else {
			return fmt.Errorf("field %q is not an int64", constants.EXP_HISTOGRAM_BUCKET_COUNTS)
		}
	}

	return nil
}
