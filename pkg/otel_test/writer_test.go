package otel_test

import (
	"github.com/davecgh/go-spew/spew"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/logs/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
	"testing"
)

func TestProto(t *testing.T) {
	t.Parallel()

	request := collogspb.ExportLogsServiceRequest{
		ResourceLogs: []*logspb.ResourceLogs{
			{
				Resource: &resourcepb.Resource{
					Attributes: []*commonpb.KeyValue{
						{
							Key: "key",
							Value: &commonpb.AnyValue{
								Value: &commonpb.AnyValue_StringValue{StringValue: "value"},
							},
						},
					},
					DroppedAttributesCount: 1,
				},
			},
		},
	}
	spew.Dump(request)
}
