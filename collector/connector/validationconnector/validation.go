// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package validationconnector

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"
	"sync"

	"github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/connector"

	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumererror"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"go.uber.org/zap"
)

const (
	typeStr = "validation"
)

type inputToOutputContext struct{}   // value is int64
type inputToValidateContext struct{} // value is int64

type Config struct {
	// Follower is the name of a pipeline where the second validation
	// connector is a receiver.  This should be set only for the first
	// validation connector, as it is how they distinguish themselves.
	Follower component.ID `mapstructure:"follower"`
}

// validation is used for any signal.
type validation struct {
	// lock protects store and sendSequence.
	lock sync.Mutex

	cfg    *Config
	logger *zap.Logger

	// store is holds all the expected data that has yet to arrive.
	store map[int64]any

	// sendSequence is used to generate new sequence numbers.
	sendSequence int64

	// only one of the following fields will be set in any given pipeline

	nextTraces  consumer.Traces
	nextMetrics consumer.Metrics
	nextLogs    consumer.Logs
}

var (
	errUnexpectedConsumer = fmt.Errorf("expected a connector router as consumer")
	errMissingFollower    = fmt.Errorf("validation input should have validation output as follower")

	asserter = assert.NewStandaloneTest()
)

func NewFactory() connector.Factory {
	return connector.NewFactory(
		typeStr,
		createDefaultConfig,
		connector.WithTracesToTraces(createTracesToTraces, component.StabilityLevelBeta),
		connector.WithMetricsToMetrics(createMetricsToMetrics, component.StabilityLevelBeta),
		connector.WithLogsToLogs(createLogsToLogs, component.StabilityLevelBeta),
	)
}

func createDefaultConfig() component.Config {
	return &Config{}
}

func (c *Config) hasFollower() bool {
	return c.Follower.Type() != ""
}

// reorder _was an attempt_ to solve the problem described in
// https://github.com/open-telemetry/opentelemetry-collector/issues/8104,
// however it does not work.  This places the follower first in the list
// of components because we want it to receive the data first.
func (v *validation) reorder(ids []component.ID) ([]component.ID, error) {
	var ordered []component.ID

	found := false
	if v.cfg.hasFollower() {
		ordered = append(ordered, v.cfg.Follower)
	}
	for _, pid := range ids {
		if v.cfg.hasFollower() && v.cfg.Follower == pid {
			found = true
			continue
		}
		ordered = append(ordered, pid)
	}
	if v.cfg.hasFollower() && !found {
		return nil, errMissingFollower
	}
	return ordered, nil
}

// expecting places an expectation by sequence number.
func (v *validation) expecting(seq int64, data any) error {
	v.lock.Lock()
	defer v.lock.Unlock()

	if v.store[seq] != nil {
		return fmt.Errorf("duplicate sequence number received: %d", seq)
	}
	v.store[seq] = data
	return nil
}

// received calls assert.Equiv for the received data item, assuming we
// have the expectation.
func (v *validation) received(seq int64, data any) error {
	v.lock.Lock()
	defer v.lock.Unlock()

	expect := v.store[seq]
	if expect == nil {
		return fmt.Errorf("missing expectation for sequence %d", seq)
	}

	if td, ok := data.(ptrace.Traces); ok {
		assert.Equiv(asserter, []json.Marshaler{
			ptraceotlp.NewExportRequestFromTraces(expect.(ptrace.Traces)),
		}, []json.Marshaler{
			ptraceotlp.NewExportRequestFromTraces(td),
		})
	} else if md, ok := data.(pmetric.Metrics); ok {
		assert.Equiv(asserter, []json.Marshaler{
			pmetricotlp.NewExportRequestFromMetrics(expect.(pmetric.Metrics)),
		}, []json.Marshaler{
			pmetricotlp.NewExportRequestFromMetrics(md),
		})
	} else if ld, ok := data.(plog.Logs); ok {
		assert.Equiv(asserter, []json.Marshaler{
			plogotlp.NewExportRequestFromLogs(expect.(plog.Logs)),
		}, []json.Marshaler{
			plogotlp.NewExportRequestFromLogs(ld),
		})
	} else {
		return fmt.Errorf("unrecognized data type")
	}

	return nil
}

func newValidation(cfg *Config, logger *zap.Logger) *validation {
	return &validation{
		cfg:    cfg,
		logger: logger,
		store:  map[int64]any{},
	}
}

func createTracesToTraces(
	ctx context.Context,
	set connector.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Traces,
) (connector.Traces, error) {
	v := newValidation(cfg.(*Config), set.Logger)

	tr, ok := nextConsumer.(connector.TracesRouter)
	if !ok {
		return nil, errUnexpectedConsumer
	}
	ordered, err := v.reorder(tr.PipelineIDs())
	if err != nil {
		return nil, err
	}
	next, err := tr.Consumer(ordered...)
	if err != nil {
		return nil, err
	}
	v.nextTraces = next
	return v, nil
}

func createMetricsToMetrics(
	ctx context.Context,
	set connector.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Metrics,
) (connector.Metrics, error) {
	v := newValidation(cfg.(*Config), set.Logger)

	tr, ok := nextConsumer.(connector.MetricsRouter)
	if !ok {
		return nil, errUnexpectedConsumer
	}
	ordered, err := v.reorder(tr.PipelineIDs())
	if err != nil {
		return nil, err
	}
	next, err := tr.Consumer(ordered...)
	if err != nil {
		return nil, err
	}
	v.nextMetrics = next
	return v, nil
}

func createLogsToLogs(
	ctx context.Context,
	set connector.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Logs,
) (connector.Logs, error) {
	v := newValidation(cfg.(*Config), set.Logger)

	tr, ok := nextConsumer.(connector.LogsRouter)
	if !ok {
		return nil, errUnexpectedConsumer
	}
	ordered, err := v.reorder(tr.PipelineIDs())
	if err != nil {
		return nil, err
	}
	next, err := tr.Consumer(ordered...)
	if err != nil {
		return nil, err
	}
	v.nextLogs = next

	return v, nil
}

func (v *validation) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

func (v *validation) Start(ctx context.Context, host component.Host) error {
	return nil
}

func (v *validation) Shutdown(ctx context.Context) error {
	return nil
}

func (v *validation) nextSequence() int64 {
	v.lock.Lock()
	defer v.lock.Unlock()
	v.sendSequence++
	return v.sendSequence
}

func (v *validation) consumeNext(ctx context.Context, data any) error {
	var err error
	switch {
	case v.nextTraces != nil:
		err = v.nextTraces.ConsumeTraces(ctx, data.(ptrace.Traces))
	case v.nextMetrics != nil:
		err = v.nextMetrics.ConsumeMetrics(ctx, data.(pmetric.Metrics))
	case v.nextLogs != nil:
		err = v.nextLogs.ConsumeLogs(ctx, data.(plog.Logs))
	default:
		err = fmt.Errorf("unhandled data type")
	}
	if err != nil {
		return consumererror.NewPermanent(err)
	}
	return nil
}

// consume is the central logic of the validation connector, both
// instances.
func (v *validation) consume(ctx context.Context, data any) error {
	if v.cfg.hasFollower() {
		// Here, the first validation connector.
		sequence := v.nextSequence()

		// Here, insert a propagating marker that we expect to
		// propagate by context headers.
		info := client.FromContext(ctx)

		// TODO: the OTel client Metadata doesn't allow itself
		// to be copied or modified without an explicit list
		// of keys, which is completely bogus and makes the
		// intended correct action here impossible. So, clobber
		// the metadata and file an issue about the problem.
		info.Metadata = client.NewMetadata(map[string][]string{
			"X-Validation-Sequence": []string{
				fmt.Sprint(sequence),
			},
		})
		ctx = client.NewContext(ctx, info)

		// Insert a non-propagating context marker for the
		// second validation connector.
		ctx = context.WithValue(ctx, inputToOutputContext{}, sequence)

		return v.consumeNext(ctx, data)
	}

	// TODO: Because of the problem documented in
	// https://github.com/open-telemetry/opentelemetry-collector/issues/8104
	// this code does not reliably receive the expectation input before
	// the actual value input.  We await a solution.
	if directExpect := ctx.Value(inputToOutputContext{}); directExpect != nil {
		// In this branch, the second validation connector is
		// receiving data directly from the first validation
		// connector.
		sequence := directExpect.(int64)

		v.logger.Debug("Received expectation", zap.Int64("sequence", sequence))

		// Expected test input.  Do not consume.
		return v.expecting(sequence, data)
	}

	// In this branch, the second validation connector is
	// receiving the data from the pipeline under test.
	info := client.FromContext(ctx)
	seqHdrs := info.Metadata.Get("validation-sequence")
	var sequence int64
	if len(seqHdrs) == 1 {
		sequence, _ = strconv.ParseInt(seqHdrs[0], 10, 64)
	}
	if sequence == 0 {
		return consumererror.NewPermanent(
			fmt.Errorf("missing first validation connector sequence expectation"))
	}

	if err := v.received(sequence, data); err != nil {
		v.logger.Info("Validation failure", zap.Error(err))

		// Output validating actual input failed.
		return consumererror.NewPermanent(err)
	}

	// Success, pass the data.
	v.logger.Info("Validation success", zap.Int64("sequence", sequence))
	return v.consumeNext(ctx, data)
}

func (v *validation) ConsumeMetrics(ctx context.Context, md pmetric.Metrics) error {
	return v.consume(ctx, md)
}

func (v *validation) ConsumeLogs(ctx context.Context, ld plog.Logs) error {
	return v.consume(ctx, ld)
}

func (v *validation) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	return v.consume(ctx, td)
}
