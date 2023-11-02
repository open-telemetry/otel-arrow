// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package zstd

import (
	"bytes"
	"fmt"
	"io"
	"runtime"
	"sync"

	zstdlib "github.com/klauspost/compress/zstd"
	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding"
)

// NamePrefix is prefix, with N for compression level.
const NamePrefix = "zstdarrow"

// Level is an integer value mapping to compression level.
// [0] implies disablement; not registered in grpc
// [1,2] fastest i.e., "zstdarrow1", "zstdarrow2"
// [3-5] default
// [6-9] better
// [10] best.
type Level uint

const (
	DefaultLevel Level = 5
	MinLevel     Level = 1
	MaxLevel     Level = 10
)

type EncoderConfig struct {
	// Level is meaningful in the range [0, 10].  No invalid
	// values, they all map into 4 default configurations.
	Level         Level  `mapstructure:"level"`
	WindowSizeMiB uint32 `mapstructure:"window_size_mib"`
	Concurrency   uint   `mapstructure:"concurrency"`
}

type DecoderConfig struct {
	// Level is symmetric with encoder config.  Although the
	// decoder object does not use this configuration, it is the
	// key used to lookup configuration corresponding with the
	// same setting on the encoder, so provides a means of
	// multi-configuration.
	Level            Level  `mapstructure:"level"`
	Concurrency      uint   `mapstructure:"concurrency"`
	MemoryLimitMiB   uint32 `mapstructure:"memory_limit_mib"`
	MaxWindowSizeMiB uint32 `mapstructure:"max_window_size_mib"`
}

type encoder struct {
	cfg  EncoderConfig
	pool mru[*writer]
}

type decoder struct {
	cfg  DecoderConfig
	pool mru[*reader]
}

type reader struct {
	*zstdlib.Decoder
	pool *mru[*reader]
}

type writer struct {
	*zstdlib.Encoder
	pool *mru[*writer]
}

type combined struct {
	enc encoder
	dec decoder
}

type instance struct {
	lock    sync.Mutex
	byLevel map[Level]*combined
}

var _ encoding.Compressor = &combined{}

var staticInstances = &instance{
	byLevel: map[Level]*combined{},
}

func DefaultEncoderConfig() EncoderConfig {
	return EncoderConfig{
		Level:       DefaultLevel, // Determines other defaults
		Concurrency: 1,            // Avoids extra CPU/memory
	}
}

func DefaultDecoderConfig() DecoderConfig {
	return DecoderConfig{
		Level:            DefaultLevel, // Default speed
		Concurrency:      1,            // Avoids extra CPU/memory
		MemoryLimitMiB:   512,          // More conservative than library default
		MaxWindowSizeMiB: 32,           // Corresponds w/ "best" level default
	}
}

func validate(level Level, f func() error) error {
	if level > MaxLevel {
		return fmt.Errorf("level out of range [0,10]: %d", level)
	}
	if level == 0 {
		return nil
	}
	return f()
}

func (cfg *EncoderConfig) Validate() error {
	return validate(cfg.Level, func() error {
		var buf bytes.Buffer
		test, err := zstdlib.NewWriter(&buf, cfg.options()...)
		if test != nil {
			test.Close()
		}
		return err
	})
}

func (cfg *DecoderConfig) Validate() error {
	return validate(cfg.Level, func() error {
		var buf bytes.Buffer
		test, err := zstdlib.NewReader(&buf, cfg.options()...)
		if test != nil {
			test.Close()
		}
		return err
	})
}

func init() {
	staticInstances.lock.Lock()
	defer staticInstances.lock.Unlock()
	for level := Level(MinLevel); level <= MaxLevel; level++ {
		var combi combined
		combi.enc.cfg = DefaultEncoderConfig()
		combi.dec.cfg = DefaultDecoderConfig()
		combi.enc.cfg.Level = level
		combi.dec.cfg.Level = level
		encoding.RegisterCompressor(&combi)
		staticInstances.byLevel[level] = &combi
	}
}

func SetEncoderConfig(cfg EncoderConfig) error {
	if err := cfg.Validate(); err == nil || cfg.Level == 0 {
		return err
	}
	staticInstances.lock.Lock()
	defer staticInstances.lock.Unlock()

	staticInstances.byLevel[cfg.Level].enc.cfg = cfg
	return nil
}

func SetDecoderConfig(cfg DecoderConfig) error {
	if err := cfg.Validate(); err == nil || cfg.Level == 0 {
		return err
	}
	staticInstances.lock.Lock()
	defer staticInstances.lock.Unlock()

	staticInstances.byLevel[cfg.Level].dec.cfg = cfg
	return nil
}

func (cfg *EncoderConfig) options() (opts []zstdlib.EOption) {
	opts = append(opts, zstdlib.WithEncoderLevel(zstdlib.EncoderLevelFromZstd(int(cfg.Level))))

	if cfg.Concurrency != 0 {
		opts = append(opts, zstdlib.WithEncoderConcurrency(int(cfg.Concurrency)))
	}
	if cfg.WindowSizeMiB != 0 {
		opts = append(opts, zstdlib.WithWindowSize(int(cfg.WindowSizeMiB<<20)))
	}

	return opts
}

func (cfg *EncoderConfig) Name() string {
	return fmt.Sprint(NamePrefix, cfg.Level)
}

func (cfg *EncoderConfig) CallOption() grpc.CallOption {
	return grpc.UseCompressor(cfg.Name())
}

func (cfg *DecoderConfig) options() (opts []zstdlib.DOption) {
	if cfg.Concurrency != 0 {
		opts = append(opts, zstdlib.WithDecoderConcurrency(int(cfg.Concurrency)))
	}
	if cfg.MaxWindowSizeMiB != 0 {
		opts = append(opts, zstdlib.WithDecoderMaxWindow(uint64(cfg.MaxWindowSizeMiB)<<20))
	}
	if cfg.MemoryLimitMiB != 0 {
		opts = append(opts, zstdlib.WithDecoderMaxMemory(uint64(cfg.MemoryLimitMiB)<<20))
	}

	return opts
}

func (c *combined) Compress(w io.Writer) (io.WriteCloser, error) {
	z := c.enc.pool.Get()
	if z == nil {
		encoder, err := zstdlib.NewWriter(w, c.enc.cfg.options()...)
		if err != nil {
			return nil, err
		}
		z = &writer{Encoder: encoder, pool: &c.enc.pool}
	} else {
		z.Encoder.Reset(w)
	}
	return z, nil
}

func (w *writer) Close() error {
	defer w.pool.Put(w)
	return w.Encoder.Close()
}

func (c *combined) Decompress(r io.Reader) (io.Reader, error) {
	z := c.dec.pool.Get()
	if z == nil {
		decoder, err := zstdlib.NewReader(r, c.dec.cfg.options()...)
		if err != nil {
			return nil, err
		}
		z = &reader{Decoder: decoder, pool: &c.dec.pool}

		// zstd decoders need to be closed when they are evicted from
		// the freelist. Note that the finalizer is attached to the
		// reader object, not to the decoder, because zstd maintains
		// background references to the decoder that prevent it from
		// being GC'ed.
		runtime.SetFinalizer(z, (*reader).Close)
	} else if err := z.Decoder.Reset(r); err != nil {
		return nil, err
	}
	return z, nil
}

func (r *reader) Read(p []byte) (n int, err error) {
	n, err = r.Decoder.Read(p)
	if err == io.EOF {
		r.pool.Put(r)
	}
	return n, err
}

func (c *combined) Name() string {
	return c.enc.cfg.Name()
}
