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

package stats

import (
	"fmt"
	"runtime"
	"time"

	"github.com/dustin/go-humanize"
)

// SystemProbe is a probe that measures CPU and memory usage.
type SystemProbe struct {
	LastTime       time.Time // last time the probe was reset
	LastMalloc     uint64    // last observed number of malloc
	LastTotalAlloc uint64    // last observed number of total bytes allocated
	LastNumGC      uint32    // last observed number of garbage collections
}

// CpuMemUsage contains the CPU and memory usage of a specific step.
type CpuMemUsage struct {
	Heap      uint64  // heap memory usage in bytes
	Malloc    uint64  // number of malloc
	Bandwidth float64 // memory bandwidth in B/s
	GcCount   uint32  // number of garbage collections
}

func NewSystemProbe() *SystemProbe {
	return &SystemProbe{}
}

func (sp *SystemProbe) Reset() {
	runtime.GC()

	sp.LastTime = time.Now()

	// Read memory stats and get the number of GCs
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	sp.LastMalloc = m.Mallocs
	sp.LastTotalAlloc = m.TotalAlloc
	sp.LastNumGC = m.NumGC
}

func (sp *SystemProbe) MeasureUsage() *CpuMemUsage {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	durationSec := time.Now().Sub(sp.LastTime).Seconds()

	return &CpuMemUsage{
		Heap:      m.Alloc,
		Malloc:    m.Mallocs - sp.LastMalloc,
		Bandwidth: float64(m.TotalAlloc-sp.LastTotalAlloc) / durationSec,
		GcCount:   m.NumGC - sp.LastNumGC,
	}
}

func (u *CpuMemUsage) ToString() string {
	malloc, unitPrefix := humanize.ComputeSI(float64(u.Malloc))
	return fmt.Sprintf("Heap{%s, %6.2f%s mallocs, bw=%s/s, #gc=%2d}",
		humanize.Bytes(u.Heap), malloc, unitPrefix, humanize.Bytes(uint64(u.Bandwidth)), u.GcCount)
}
