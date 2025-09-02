/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package main

import (
	"encoding/json"
	"log"
	"os"
	"testing"
	"time"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
)

type Evt struct {
	Timestamp time.Time
	Tags      map[string]string
	Fields    map[string]int64
}

type Entry struct {
	Ts       time.Time
	SourceId string `json:"source_id"`
	Evt      Evt
}

func TestFile(t *testing.T) {
	t.Skipf("Not really a test, just a way to run the code without creating a main")

	content, err := os.ReadFile("../../data/multivariate-metrics.json")
	if err != nil {
		log.Fatal("Error when opening file: ", err)
	}

	var payload []*Entry
	err = json.Unmarshal(content, &payload)
	if err != nil {
		log.Fatal("Error during Unmarshal(): ", err)
	}

	metrics := pmetric.NewMetrics()
	rms := metrics.ResourceMetrics()
	rms.EnsureCapacity(1)
	rm := rms.AppendEmpty()
	sms := rm.ScopeMetrics()

	sources := make(map[string][]*Entry)

	for _, entry := range payload {
		sources[entry.SourceId] = append(sources[entry.SourceId], entry)
	}

	sms.EnsureCapacity(len(sources))
	for sourceId, entries := range sources {
		sm := sms.AppendEmpty()
		sm.Scope().Attributes().PutStr("source_id", sourceId)
		ms := sm.Metrics()
		ms.EnsureCapacity(len(entries))
		for _, entry := range entries {
			m := ms.AppendEmpty()
			m.SetName("http.metrics")
			m.SetDescription("Set of HTTP metrics")
			sum := m.SetEmptySum()
			dps := sum.DataPoints()
			dps.EnsureCapacity(len(entry.Evt.Fields))
			for k, v := range entry.Evt.Fields {
				dp := dps.AppendEmpty()
				dp.SetTimestamp(pcommon.Timestamp(entry.Ts.UnixNano()))
				attrs := dp.Attributes()
				attrs.EnsureCapacity(len(entry.Evt.Tags) + 1)
				for ak, av := range entry.Evt.Tags {
					attrs.PutStr(ak, av)
				}
				attrs.PutStr("metric", k)
				dp.SetIntValue(v)
			}
		}
	}

	bytes, err := pmetricotlp.NewExportRequestFromMetrics(metrics).MarshalProto()
	if err != nil {
		log.Fatal("Error during MarshalProto(): ", err)
	}
	err = os.WriteFile("/Users/L.Querel/GolandProjects/otel-arrow-adapter/data/multivariate-metrics.pb", bytes, 0600)
	if err != nil {
		log.Fatal("Error during WriteFile(): ", err)
	}
}
