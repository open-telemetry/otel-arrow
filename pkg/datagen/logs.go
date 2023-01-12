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
	"time"

	"github.com/brianvoe/gofakeit/v6"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
)

type LogsGenerator struct {
	*DataGenerator

	generation int
}

func NewLogsGenerator(entropy TestEntropy, resourceAttributes []pcommon.Map, instrumentationScopes []pcommon.InstrumentationScope) *LogsGenerator {
	return &LogsGenerator{
		DataGenerator: NewDataGenerator(entropy, resourceAttributes, instrumentationScopes),
		generation:    0,
	}
}

func (lg *LogsGenerator) Generate(batchSize int, collectInterval time.Duration) plog.Logs {
	result := plog.NewLogs()

	for i := 0; i < batchSize; i++ {
		resourceLogs := result.ResourceLogs().AppendEmpty()
		lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)].
			CopyTo(resourceLogs.Resource().Attributes())

		scopeLogs := resourceLogs.ScopeLogs().AppendEmpty()
		lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)].
			CopyTo(scopeLogs.Scope())

		logRecords := scopeLogs.LogRecords()

		lg.AdvanceTime(collectInterval)
		lg.NextId8Bytes()
		lg.NextId16Bytes()

		lg.LogDebugRecord(logRecords.AppendEmpty())
		lg.LogInfoRecord(logRecords.AppendEmpty())
		lg.LogWarnRecord(logRecords.AppendEmpty())
		lg.LogErrorRecord(logRecords.AppendEmpty())
		lg.LogInfoComplexRecord(logRecords.AppendEmpty())
	}

	return result
}

func (dg *DataGenerator) LogDebugRecord(log plog.LogRecord) {
	dg.logRecord(log, plog.SeverityNumberDebug, "DEBUG")
}

func (dg *DataGenerator) LogInfoRecord(log plog.LogRecord) {
	dg.logRecord(log, plog.SeverityNumberInfo, "INFO")
}

func (dg *DataGenerator) LogWarnRecord(log plog.LogRecord) {
	dg.logRecord(log, plog.SeverityNumberWarn, "INFO")
}

func (dg *DataGenerator) LogErrorRecord(log plog.LogRecord) {
	dg.logRecord(log, plog.SeverityNumberError, "INFO")
}

func (dg *DataGenerator) LogInfoComplexRecord(log plog.LogRecord) {
	dg.complexLogRecord(log, plog.SeverityNumberError, "INFO")
}

func (dg *DataGenerator) logRecord(log plog.LogRecord, sev plog.SeverityNumber, txt string) {
	log.SetTimestamp(dg.CurrentTime())
	log.SetObservedTimestamp(dg.CurrentTime())
	log.SetSeverityNumber(sev)
	log.SetSeverityText(txt)
	log.Body().SetStr(gofakeit.LoremIpsumSentence(10))
	dg.NewStandardAttributes().CopyTo(log.Attributes())
	log.SetTraceID(dg.Id16Bytes())
	log.SetSpanID(dg.Id8Bytes())
}

func (dg *DataGenerator) complexLogRecord(log plog.LogRecord, sev plog.SeverityNumber, txt string) {
	log.SetTimestamp(dg.CurrentTime())
	log.SetObservedTimestamp(dg.CurrentTime())
	log.SetSeverityNumber(sev)
	log.SetSeverityText(txt)
	obj := log.Body().SetEmptyMap()
	obj.PutStr("attr1", gofakeit.LoremIpsumSentence(10))
	obj.PutInt("attr2", 1)
	obj.PutDouble("attr3", 2.0)
	obj.PutBool("attr4", true)
	dg.NewStandardAttributes().CopyTo(log.Attributes())
	log.SetTraceID(dg.Id16Bytes())
	log.SetSpanID(dg.Id8Bytes())
}
