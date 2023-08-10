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
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/processor"
	"go.uber.org/multierr"
	"go.uber.org/zap"
)

var _ processor.Logs = (*logProcessor)(nil)

type logProcessor struct {
	logger *zap.Logger
	config *Config

	router router[exporter.Logs]
}

func newLogProcessor(settings component.TelemetrySettings, config component.Config) *logProcessor {
	cfg := config.(*Config)

	return &logProcessor{
		logger: settings.Logger,
		config: cfg,
		router: newRouter[exporter.Logs](
			cfg.Table,
			settings,
		),
	}
}

func (p *logProcessor) Start(_ context.Context, host component.Host) error {
	return p.router.registerExporters(host.GetExporters()[component.DataTypeLogs])
}

func (p *logProcessor) ConsumeLogs(ctx context.Context, l plog.Logs) error {
	exporters := p.router.getExporters()

	var errs error
	for _, e := range exporters {
		errs = multierr.Append(errs, e.ConsumeLogs(ctx, l))
	}
	return errs
}

func (p *logProcessor) Shutdown(context.Context) error {
	return nil
}

func (p *logProcessor) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}
