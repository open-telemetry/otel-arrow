/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCsvDataset(t *testing.T) {
	t.Parallel()

	ds := CsvToLogsDataset("./test-logs.csv")

	assert.Equal(t, 3, ds.Len())

	logs := ds.Logs(0, ds.Len())
	assert.Equal(t, 1, len(logs))

	logResource := logs[0]
	lrs := logResource.ResourceLogs()
	assert.Equal(t, 1, lrs.Len())
	lr := lrs.At(0)
	res := lr.Resource()
	resAttrs := res.Attributes()
	assert.Equal(t, 1, resAttrs.Len())
	v, ok := resAttrs.Get("host.name")
	assert.True(t, ok)
	assert.Equal(t, "host", v.Str())

	sls := lr.ScopeLogs()
	assert.Equal(t, 1, sls.Len())
	sl := sls.At(0)
	lrecs := sl.LogRecords()
	assert.Equal(t, 3, lrecs.Len())

	lrec := lrecs.At(0)
	assert.Equal(t, "OK", lrec.Body().AsString())
	attrs := lrec.Attributes()
	assert.Equal(t, 6, attrs.Len())
	method, ok := attrs.Get("method")
	assert.True(t, ok)
	assert.Equal(t, "GET", method.Str())
	url, ok := attrs.Get("url")
	assert.True(t, ok)
	assert.Equal(t, "http://www.example.com/", url.Str())
	status, ok := attrs.Get("status")
	assert.True(t, ok)
	assert.Equal(t, int64(200), status.Int())
	duration, ok := attrs.Get("duration")
	assert.True(t, ok)
	assert.Equal(t, float64(10.5), duration.Double())
	contentSize, ok := attrs.Get("content-size")
	assert.True(t, ok)
	assert.Equal(t, int64(2000), contentSize.Int())
	valid, ok := attrs.Get("valid")
	assert.True(t, ok)
	assert.Equal(t, true, valid.Bool())

	lrec = lrecs.At(1)
	assert.Equal(t, "Not Found", lrec.Body().AsString())
	attrs = lrec.Attributes()
	assert.Equal(t, 6, attrs.Len())
	method, ok = attrs.Get("method")
	assert.True(t, ok)
	assert.Equal(t, "GET", method.Str())
	url, ok = attrs.Get("url")
	assert.True(t, ok)
	assert.Equal(t, "http://www.example.com/abc", url.Str())
	status, ok = attrs.Get("status")
	assert.True(t, ok)
	assert.Equal(t, int64(404), status.Int())
	duration, ok = attrs.Get("duration")
	assert.True(t, ok)
	assert.Equal(t, float64(34.5), duration.Double())
	contentSize, ok = attrs.Get("content-size")
	assert.True(t, ok)
	assert.Equal(t, int64(3000), contentSize.Int())
	valid, ok = attrs.Get("valid")
	assert.True(t, ok)
	assert.Equal(t, true, valid.Bool())

	lrec = lrecs.At(2)
	assert.Equal(t, "Service Unavailable", lrec.Body().AsString())
	attrs = lrec.Attributes()
	assert.Equal(t, 6, attrs.Len())
	method, ok = attrs.Get("method")
	assert.True(t, ok)
	assert.Equal(t, "POST", method.Str())
	url, ok = attrs.Get("url")
	assert.True(t, ok)
	assert.Equal(t, "http://www.example.com/abc", url.Str())
	status, ok = attrs.Get("status")
	assert.True(t, ok)
	assert.Equal(t, int64(503), status.Int())
	duration, ok = attrs.Get("duration")
	assert.True(t, ok)
	assert.Equal(t, float64(7.6), duration.Double())
	contentSize, ok = attrs.Get("content-size")
	assert.True(t, ok)
	assert.Equal(t, int64(4000), contentSize.Int())
	valid, ok = attrs.Get("valid")
	assert.True(t, ok)
	assert.Equal(t, false, valid.Bool())
}
