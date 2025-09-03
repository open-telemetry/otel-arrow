/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

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

	resourceLogs := result.ResourceLogs().AppendEmpty()
	lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)].
		CopyTo(resourceLogs.Resource().Attributes())

	scopeLogs := resourceLogs.ScopeLogs().AppendEmpty()
	lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)].
		CopyTo(scopeLogs.Scope())

	for i := 0; i < batchSize; i++ {
		logRecords := scopeLogs.LogRecords()

		lg.AdvanceTime(collectInterval)
		lg.NextId8Bytes()
		lg.NextId16Bytes()

		lg.LogDebugRecord(logRecords.AppendEmpty())
		lg.LogInfoRecord(logRecords.AppendEmpty())
		lg.LogWarnRecord(logRecords.AppendEmpty())
		lg.LogErrorRecord(logRecords.AppendEmpty())
		//lg.LogInfoComplexRecord(logRecords.AppendEmpty())
		lg.RandomLogRecord(logRecords.AppendEmpty())
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

// RandomLogRecord generates a random log record. The list of fields set is random.
// The value of these fields is also random.
func (dg *DataGenerator) RandomLogRecord(log plog.LogRecord) {
	if dg.GenBool() {
		log.SetTimestamp(dg.CurrentTime())
	}
	if dg.GenBool() {
		log.SetObservedTimestamp(dg.CurrentTime())
	}
	if dg.GenBool() {
		log.SetSeverityNumber(plog.SeverityNumber(gofakeit.Number(0, 4)))
	}
	if dg.GenBool() {
		log.SetSeverityText(gofakeit.LetterN(4))
	}
	if dg.GenBool() {
		dg.RandomBody(log.Body())
	}
	if dg.GenBool() {
		dg.RandomAttributes().CopyTo(log.Attributes())
	}
	if dg.GenBool() {
		log.SetTraceID(dg.Id16Bytes())
	}
	if dg.GenBool() {
		log.SetSpanID(dg.Id8Bytes())
	}
	if dg.GenBool() {
		log.SetDroppedAttributesCount(uint32(gofakeit.Number(0, 1000)))
	}
	if dg.GenBool() {
		log.SetFlags(plog.LogRecordFlags(gofakeit.Number(0, 1000)))
	}
}

func (dg *DataGenerator) RandomBody(body pcommon.Value) {
	switch dg.GenI64Range(0, 11) {
	case 0:
		// Body with a random string
		body.SetStr(gofakeit.LoremIpsumSentence(20))
	case 1:
		// Body an empty string
		body.SetStr("")
	case 2:
		// Empty body
	case 3:
		// Body with a random int
		body.SetInt(gofakeit.Int64())
	case 4:
		// Body with a random double
		body.SetDouble(gofakeit.Float64())
	case 5:
		// Body with a random bool
		body.SetBool(gofakeit.Bool())
	case 6:
		// Body with a slice of random bytes
		body.SetEmptyBytes().FromRaw(dg.GenId(10))
	case 7:
		// Body with an empty slice of bytes
		body.SetEmptyBytes()
	case 8:
		// Body with a random map
		bodyMap := body.SetEmptyMap()
		bodyMap.PutStr("attr1", gofakeit.LoremIpsumSentence(10))
		bodyMap.PutInt("attr2", 1)
	case 9:
		// Body with an empty map
		body.SetEmptyMap()
	case 10:
		// Body with a random slice
		bodySlice := body.SetEmptySlice()
		bodySlice.AppendEmpty().SetStr(gofakeit.LoremIpsumSentence(10))
		bodySlice.AppendEmpty().SetInt(gofakeit.Int64())
	case 11:
		// Body with an empty slice
		body.SetEmptySlice()
	}
}
