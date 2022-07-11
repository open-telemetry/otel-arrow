package logs_test

import (
	"otel-arrow-adapter/pkg/otel/fake"
	"otel-arrow-adapter/pkg/otel/logs"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/config"
	"testing"
)

func TestOtlpLogsToArrowLogs(t *testing.T) {
	t.Parallel()

	cfg := config.NewDefaultConfig()
	rbr := rbb.NewRecordBatchRepository(cfg)
	lg := fake.NewLogsGenerator(fake.DefaultResourceAttributes(), fake.DefaultInstrumentationScope())

	request := lg.Generate(10, 100)
	records, err := logs.OtlpLogsToArrowLogs(rbr, request)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(records) != 1 {
		t.Errorf("Expected 1 record, got %d", len(records))
	}
}
