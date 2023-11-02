package zstd

import (
	"fmt"
	"io"
	"runtime"
	"sync"

	"github.com/klauspost/compress/zstd"
	zstdlib "github.com/klauspost/compress/zstd"
	"google.golang.org/grpc/encoding"
)

// Name is the customized for Arrow so that the Arrow exporter and
// receiver can be configured without clobbering others use of "zstd".
const Name = "zstdarrow"

type CompressionLevel zstd.EncoderLevel

type EncoderConfig struct {
	Level         CompressionLevel `mapstructure:"level"`
	WindowSizeMiB uint64           `mapstructure:"window_size_mib"`
	Concurrency   int              `mapstructure:"concurrency"`
}

type DecoderConfig struct {
	Concurrency      int    `mapstructure:"concurrency"`
	MaxMemoryMiB     uint64 `mapstructure:"max_memory_mib"`
	MaxWindowSizeMiB uint64 `mapstructure:"max_window_size_mib"`
}

type instance struct {
	lock             sync.Mutex
	encodeOpts       []zstdlib.EOption
	decodeOpts       []zstdlib.DOption
	poolCompressor   mru[*writer]
	poolDecompressor mru[*reader]
}

type reader struct {
	*zstdlib.Decoder
	pool *mru[*reader]
}

type writer struct {
	*zstdlib.Encoder
	pool *mru[*writer]
}

func DefaultEncoderConfig() EncoderConfig {
	return EncoderConfig{
		Concurrency: 1,                                      // Avoids extra CPU/memory
		Level:       CompressionLevel(zstdlib.SpeedDefault), // Determines other defaults

	}
}

func DefaultDecoderConfig() DecoderConfig {
	return DecoderConfig{
		Concurrency:      1,   // Avoids extra CPU/memory
		MaxMemoryMiB:     512, // More conservative than library default
		MaxWindowSizeMiB: 32,  // Corresponds w/ "best" level default
	}
}

var staticInstance = &instance{}

var _ encoding.Compressor = &instance{}

func init() {
	SetEncoderConfig(DefaultEncoderConfig())
	SetDecoderConfig(DefaultDecoderConfig())
	encoding.RegisterCompressor(staticInstance)
}

func SetEncoderConfig(enc EncoderConfig) {
	staticInstance.setEncoderConfig(enc)
}

func SetDecoderConfig(dec DecoderConfig) {
	staticInstance.setDecoderConfig(dec)
}

func (c *instance) setEncoderConfig(enc EncoderConfig) {
	var opts []zstdlib.EOption{}

	opts = append(opts, zstdlib.WithEncoderLevel(enc.Level))
	if enc.Concurrency != 0 {
		opts = append(opts, zstdlib.WithEncoderConcurrency(enc.Concurrency))
	}
	if enc.WindowSizeMiB != 0 {
		opts = append(opts, zstdlib.WithWindowSize(enc.WindowSizeMiB<<20))
	}
		
	c.lock.Lock()
	defer c.lock.Unlock()

	c.encodeOpts = opts
}

func (c *instance) setDecoderConfig(dec DecoderConfig) {
	var opts []zstdlib.DOption{}

	// @@@
		
	c.lock.Lock()
	defer c.lock.Unlock()

	c.decodeOpts = opts
}

func (c *instance) decoderOptions() []zstdlib.DOption {
	c.lock.Lock()
	defer c.lock.Unlock()
	return c.decodeOpts
}

func (c *instance) encoderOptions() []zstdlib.EOption {
	c.lock.Lock()
	defer c.lock.Unlock()
	return c.encodeOpts
}

func (c *instance) Compress(w io.Writer) (io.WriteCloser, error) {
	z := c.poolCompressor.Get()
	if z == nil {
		encoder, err := zstdlib.NewWriter(w, c.encoderOptions()...)
		if err != nil {
			return nil, err
		}
		z = &writer{Encoder: encoder, pool: &c.poolCompressor}
	} else {
		z.Encoder.Reset(w)
	}
	return z, nil
}

func (w *writer) Close() error {
	defer w.pool.Put(w)
	return w.Encoder.Close()
}

func (c *instance) Decompress(r io.Reader) (io.Reader, error) {
	z := c.poolDecompressor.Get()
	if z == nil {
		decoder, err := zstdlib.NewReader(r, c.decoderOptions()...)
		if err != nil {
			return nil, err
		}
		z = &reader{Decoder: decoder, pool: &c.poolDecompressor}

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

func (c *instance) Name() string {
	return Name
}

func (cl *CompressionLevel) UnmarshalText(in []byte) error {
	lstr := string(in)
	if len(lstr) == 0 {
		*cl = CompressionLevel(zstdlib.SpeedDefault)
		return nil
	}

	ok, level := zstdlib.EncoderLevelFromString(lstr)
	if !ok {
		return fmt.Errorf("unsupported compression level %q", lstr)
	}
	*cl = CompressionLevel(level)
	return nil
}
