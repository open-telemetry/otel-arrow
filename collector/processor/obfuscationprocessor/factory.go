package obfuscationprocessor

import (
	"context"
	"github.com/cyrildever/feistel"
	"github.com/cyrildever/feistel/common/utils/hash"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/processor"
	"go.opentelemetry.io/collector/processor/processorhelper"
)

const (
	// The value of "type" key in configuration.
	typeStr = "obfuscation"
	// The stability level of the exporter.
	stability = component.StabilityLevelAlpha

	defaultRound = 10
)

// NewFactory creates a factory for the obfuscation processor.
func NewFactory() processor.Factory {
	return processor.NewFactory(
		typeStr,
		createDefaultConfig,
		processor.WithTraces(createTracesProcessor, stability),
		processor.WithMetrics(createMetricsProcessor, stability),
	)
}

func createDefaultConfig() component.Config { 
	return &Config{
		EncryptRound: defaultRound,
		// encrypt all string attributes by default
		EncryptAll:   true,
	}
}

func createMetricsProcessor(
	ctx context.Context,
	set processor.CreateSettings,
	cfg component.Config,
	next consumer.Metrics,
) (processor.Metrics, error) {
	oCfg := cfg.(*Config)
	processor := &obfuscation{
		logger:            set.Logger,
		nextMetrics:       next,
		encrypt:           feistel.NewFPECipher(hash.SHA_256, oCfg.EncryptKey, oCfg.EncryptRound),
		encryptAttributes: makeEncryptList(oCfg),
		encryptAll:        oCfg.EncryptAll,
	}
	return processorhelper.NewMetricsProcessor(
		ctx,
		set,
		cfg,
		next,
		processor.processMetrics,
		processorhelper.WithCapabilities(processor.Capabilities()),
		processorhelper.WithStart(processor.Start),
		processorhelper.WithShutdown(processor.Shutdown))
}

// createTracesProcessor creates an instance of obfuscation for processing traces
func createTracesProcessor(
	ctx context.Context,
	set processor.CreateSettings,
	cfg component.Config,
	next consumer.Traces,
) (processor.Traces, error) {
	oCfg := cfg.(*Config)
	processor := &obfuscation{
		logger:            set.Logger,
		nextTraces:        next,
		encrypt:           feistel.NewFPECipher(hash.SHA_256, oCfg.EncryptKey, oCfg.EncryptRound),
		encryptAttributes: makeEncryptList(oCfg),
		encryptAll:        oCfg.EncryptAll,
	}
	return processorhelper.NewTracesProcessor(
		ctx,
		set,
		cfg,
		next,
		processor.processTraces,
		processorhelper.WithCapabilities(processor.Capabilities()),
		processorhelper.WithStart(processor.Start),
		processorhelper.WithShutdown(processor.Shutdown))
}

// makeEncryptList sets up a lookup table of span attribute keys which need to be encrypted.
func makeEncryptList(c *Config) map[string]struct{} {
	allowList := make(map[string]struct{}, len(c.EncryptAttributes))
	for _, key := range c.EncryptAttributes {
		allowList[key] = struct{}{}
	}
	if len(allowList) > 0 {
		c.EncryptAll = false
	}
	return allowList
}