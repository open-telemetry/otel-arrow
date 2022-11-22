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

package datagen

import (
	"math/rand"
	"time"

	"go.opentelemetry.io/collector/pdata/pcommon"
)

type Config struct {
	// Probability of generating a metric description
	ProbMetricDescription float64
	// Probability of generating a metric unit
	ProbMetricUnit float64
	// Probability of generating a metric histogram with a sum
	ProbHistogramHasSum float64
	// Probability of generating a metric histogram with a min
	ProbHistogramHasMin float64
	// Probability of generating a metric histogram with a max
	ProbHistogramHasMax float64
}

func NewDefaultConfig() Config {
	return Config{
		ProbMetricDescription: 1.0,
		ProbMetricUnit:        1.0,
		ProbHistogramHasSum:   1.0,
		ProbHistogramHasMin:   1.0,
		ProbHistogramHasMax:   1.0,
	}
}

type TestEntropy struct {
	rng   *rand.Rand
	start int64
}

func NewTestEntropy(seed int64) TestEntropy {
	//#nosec G404 -- This is a false positive, this random number generator is not used for test purposes
	rng := rand.New(rand.NewSource(seed))

	// let the start time happen in 2022 as the first rng draw.
	start := time.Date(2022, time.January, 1, 0, 0, 0, 0, time.UTC).
		Add(time.Hour * 24 * time.Duration(rng.Intn(365))).
		UnixNano()

	return TestEntropy{
		rng:   rng,
		start: start,
	}
}

func (te TestEntropy) Start() int64 {
	return te.start
}

type DataGenerator struct {
	TestEntropy

	prevTime    pcommon.Timestamp
	currentTime pcommon.Timestamp
	id8Bytes    pcommon.SpanID
	id16Bytes   pcommon.TraceID

	config Config

	resourceAttributes    []pcommon.Map
	instrumentationScopes []pcommon.InstrumentationScope
}

func NewDataGenerator(entropy TestEntropy, resourceAttributes []pcommon.Map, instrumentationScopes []pcommon.InstrumentationScope) *DataGenerator {
	dg := &DataGenerator{
		TestEntropy:           entropy,
		prevTime:              pcommon.Timestamp(entropy.Start()),
		currentTime:           pcommon.Timestamp(entropy.Start()),
		config:                NewDefaultConfig(),
		resourceAttributes:    resourceAttributes,
		instrumentationScopes: instrumentationScopes,
	}
	dg.NextId8Bytes()
	dg.NextId16Bytes()
	return dg
}

func (dg *DataGenerator) WithConfig(config Config) *DataGenerator {
	dg.config = config
	return dg
}

func (dg *DataGenerator) PrevTime() pcommon.Timestamp {
	return dg.prevTime
}

func (dg *DataGenerator) CurrentTime() pcommon.Timestamp {
	return dg.currentTime
}

func (dg *DataGenerator) AdvanceTime(timeDelta time.Duration) {
	dg.prevTime = dg.currentTime
	dg.currentTime += pcommon.Timestamp(timeDelta)
}

func (dg *DataGenerator) NextId8Bytes() {
	copy(dg.id8Bytes[:], dg.GenId(8))
}

func (dg *DataGenerator) NextId16Bytes() {
	copy(dg.id16Bytes[:], dg.GenId(16))
}

func (dg *DataGenerator) Id8Bytes() pcommon.SpanID {
	return dg.id8Bytes
}

func (dg *DataGenerator) Id16Bytes() pcommon.TraceID {
	return dg.id16Bytes
}

func (dg *DataGenerator) GenF64Range(min float64, max float64) float64 {
	return min + dg.rng.Float64()*(max-min)
}

func (dg *DataGenerator) GenI64Range(min int64, max int64) int64 {
	return min + int64(dg.rng.Float64()*float64(max-min))
}

func (dg *DataGenerator) HasMetricDescription() bool {
	if dg.config.ProbMetricDescription == 1.0 {
		return true
	} else if dg.config.ProbMetricDescription == 0.0 {
		return false
	} else {
		return dg.rng.Float64() < dg.config.ProbMetricDescription
	}
}

func (dg *DataGenerator) HasMetricUnit() bool {
	if dg.config.ProbMetricUnit == 1.0 {
		return true
	} else if dg.config.ProbMetricUnit == 0.0 {
		return false
	} else {
		return dg.rng.Float64() < dg.config.ProbMetricUnit
	}
}

func (dg *DataGenerator) HasHistogramSum() bool {
	return dg.rng.Float64() < dg.config.ProbHistogramHasSum
}

func (dg *DataGenerator) HasHistogramMin() bool {
	return dg.rng.Float64() < dg.config.ProbHistogramHasMin
}

func (dg *DataGenerator) HasHistogramMax() bool {
	return dg.rng.Float64() < dg.config.ProbHistogramHasMax
}

func (dg *DataGenerator) GenBool() bool {
	return dg.rng.Intn(2) == 0
}

func (e TestEntropy) GenId(len uint) []byte {
	d := make([]byte, len)
	_, _ = e.rng.Read(d)
	return d
}
