/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package config

// Main configuration object in the package.

import (
	"math"

	"github.com/apache/arrow/go/v12/arrow/memory"
)

type Config struct {
	Pool memory.Allocator

	// InitIndexSize sets the initial size of a dictionary index.
	InitIndexSize uint64
	// LimitIndexSize sets the maximum size of a dictionary index
	// before it is no longer encoded as a dictionary.
	LimitIndexSize uint64
	// Zstd enables the use of ZSTD compression for IPC messages.
	Zstd bool // Use IPC ZSTD compression
	// Stats enables the collection of statistics about the data being encoded.
	Stats bool
}

type Option func(*Config)

// DefaultConfig returns a Config with the following default values:
//  - Pool: memory.NewGoAllocator()
//  - InitIndexSize: math.MaxUint16
//  - LimitIndexSize: math.MaxUint32
//  - Stats: false
//  - Zstd: true
func DefaultConfig() *Config {
	return &Config{
		Pool:           memory.NewGoAllocator(),
		InitIndexSize:  math.MaxUint16,
		LimitIndexSize: math.MaxUint32,
		Stats:          false,
		Zstd:           true,
	}
}

// WithAllocator sets the allocator to use for the Producer.
func WithAllocator(allocator memory.Allocator) Option {
	return func(cfg *Config) {
		cfg.Pool = allocator
	}
}

// WithNoDictionary sets the Producer to not use dictionary encoding.
func WithNoDictionary() Option {
	return func(cfg *Config) {
		cfg.InitIndexSize = 0
		cfg.LimitIndexSize = 0
	}
}

// WithUint8InitDictIndex sets the Producer to use an uint8 index for all dictionaries.
func WithUint8InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.InitIndexSize = math.MaxUint8
	}
}

// WithUint16InitDictIndex sets the Producer to use an uint16 index for all dictionaries.
func WithUint16InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.InitIndexSize = math.MaxUint16
	}
}

// WithUint32LinitDictIndex sets the Producer to use an uint32 index for all dictionaries.
func WithUint32LinitDictIndex() Option {
	return func(cfg *Config) {
		cfg.InitIndexSize = math.MaxUint32
	}
}

// WithUint64InitDictIndex sets the Producer to use an uint64 index for all dictionaries.
func WithUint64InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.InitIndexSize = math.MaxUint64
	}
}

// WithUint8LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint8 index.
func WithUint8LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.LimitIndexSize = math.MaxUint8
	}
}

// WithUint16LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint16 index.
func WithUint16LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.LimitIndexSize = math.MaxUint16
	}
}

// WithUint32LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint32 index.
func WithUint32LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.LimitIndexSize = math.MaxUint32
	}
}

// WithUint64LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint64 index.
func WithUint64LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.LimitIndexSize = math.MaxUint64
	}
}

// WithZstd sets the Producer to use Zstd compression at the Arrow IPC level.
func WithZstd() Option {
	return func(cfg *Config) {
		cfg.Zstd = true
	}
}

// WithNoZstd sets the Producer to not use Zstd compression at the Arrow IPC level.
func WithNoZstd() Option {
	return func(cfg *Config) {
		cfg.Zstd = false
	}
}

// WithStats enables the collection of statistics about the data being encoded.
func WithStats() Option {
	return func(cfg *Config) {
		cfg.Stats = true
	}
}
