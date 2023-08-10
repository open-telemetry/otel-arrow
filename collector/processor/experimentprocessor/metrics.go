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

package experimentprocessor // import "github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor"

import (
	"context"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/processor"
	"go.uber.org/multierr"
	"go.uber.org/zap"
)

var _ processor.Metrics = (*metricsProcessor)(nil)

type metricsProcessor struct {
	logger *zap.Logger
	config *Config

	router router[exporter.Metrics]
}

func newMetricProcessor(settings component.TelemetrySettings, config component.Config) *metricsProcessor {
	cfg := config.(*Config)

	return &metricsProcessor{
		logger: settings.Logger,
		config: cfg,
		router: newRouter[exporter.Metrics](
			cfg.Table,
			settings,
		),
	}
}

func (p *metricsProcessor) Start(_ context.Context, host component.Host) error {
	return p.router.registerExporters(host.GetExporters()[component.DataTypeMetrics])
}

func (p *metricsProcessor) ConsumeMetrics(ctx context.Context, m pmetric.Metrics) error {
	exporters := p.router.getExporters()

	var errs error
	for _, e := range exporters {
		errs = multierr.Append(errs, e.ConsumeMetrics(ctx, m))
	}
	return errs
}

func (p *metricsProcessor) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

func (p *metricsProcessor) Shutdown(context.Context) error {
	return nil
}
