/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package dataset

import (
	"bufio"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/klauspost/compress/zstd"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"golang.org/x/exp/rand"

	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark"
	carrow "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/otlp"
)

// ===== Real traces dataset =====

type RealTraceDataset struct {
	spans       []ptrace.Span
	s2r         map[ptrace.Span]pcommon.Resource
	s2s         map[ptrace.Span]pcommon.InstrumentationScope
	sizeInBytes int
}

type spanSorter struct {
	*RealTraceDataset
	field string
}

var _ sort.Interface = spanSorter{}

type traceReader struct {
	stringReader *bufio.Reader
	unmarshaler  *ptrace.JSONUnmarshaler
	bytesRead    int
}

func (tr *traceReader) readAllTraces() (ptrace.Traces, error) {
	traces := ptrace.NewTraces()

	for {
		if line, err := tr.stringReader.ReadString('\n'); err == nil {
			tl, err := tr.unmarshaler.UnmarshalTraces([]byte(line))
			if err != nil {
				return traces, err
			}
			for i := 0; i < tl.ResourceSpans().Len(); i++ {
				rs := traces.ResourceSpans().AppendEmpty()
				tl.ResourceSpans().At(i).CopyTo(rs)
			}
			tr.bytesRead += len(line)
		} else { // failed to read line
			if err != nil {
				if errors.Is(err, io.EOF) {
					return traces, nil
				}
				return traces, err
			}
		}
	}
}

func tracesFromJSON(path string, compression string) (ptrace.Traces, int) {
	file, err := os.Open(filepath.Clean(path))
	if err != nil {
		log.Fatal("open file:", err)
	}

	tr := &traceReader{
		unmarshaler: &ptrace.JSONUnmarshaler{},
		bytesRead:   0,
	}

	if compression == benchmark.CompressionTypeZstd {
		cr, err := zstd.NewReader(file)
		if err != nil {
			log.Fatal("Failed to create compressed reader for `", path, "`: ", err)
		}
		tr.stringReader = bufio.NewReader(cr)
	} else { // no compression
		tr.stringReader = bufio.NewReader(file)
	}

	traces, err := tr.readAllTraces()
	if err != nil {
		if tr.bytesRead == 0 {
			log.Fatal("Read zero bytes from file `", path, "`: ", err)
		}
		log.Print("Found error when reading file `", path, "`: ", err)
		log.Print("Bytes read: ", tr.bytesRead)
	}

	return traces, tr.bytesRead
}

func tracesFromProto(path string, compression string) (ptrace.Traces, int) {
	data, err := os.ReadFile(filepath.Clean(path))
	if err != nil {
		log.Fatal("read file:", err)
	}

	otlp := ptraceotlp.NewExportRequest()
	if err := otlp.UnmarshalProto(data); err != nil {
		log.Fatalf("in %q unmarshal: %v", path, err)
	}

	traces := otlp.Traces()
	return traces, len(data)
}

// NewRealTraceDataset creates a new RealTraceDataset from a binary file
// which is either formatted as otlp protobuf or compressed otlp json.
func NewRealTraceDataset(path string, compression string, format string, sortOrder []string) *RealTraceDataset {
	var traces ptrace.Traces
	var size int
	if format == "json" {
		traces, size = tracesFromJSON(path, compression)
	} else {
		traces, size = tracesFromProto(path, compression)
	}

	ds := &RealTraceDataset{
		s2r:         map[ptrace.Span]pcommon.Resource{},
		s2s:         map[ptrace.Span]pcommon.InstrumentationScope{},
		sizeInBytes: size,
	}

	for i := 0; i < traces.ResourceSpans().Len(); i++ {
		rs := traces.ResourceSpans().At(i)
		for j := 0; j < rs.ScopeSpans().Len(); j++ {
			ss := rs.ScopeSpans().At(j)

			for k := 0; k < ss.Spans().Len(); k++ {
				s := ss.Spans().At(k)

				ds.spans = append(ds.spans, s)
				ds.s2r[s] = rs.Resource()
				ds.s2s[s] = ss.Scope()
			}
		}
	}

	rand.Shuffle(len(ds.spans), spanSorter{RealTraceDataset: ds}.Swap)

	for i := len(sortOrder) - 1; i >= 0; i-- {
		sort.Stable(spanSorter{
			RealTraceDataset: ds,
			field:            sortOrder[i],
		})
	}

	return ds
}

func (d *RealTraceDataset) Resize(size int) {
	d.spans = d.spans[:size]
}

func (d *RealTraceDataset) SizeInBytes() int {
	return d.sizeInBytes
}

func (d *RealTraceDataset) Len() int {
	return len(d.spans)
}

func (d *RealTraceDataset) ShowStats() {
}

func (d *RealTraceDataset) Traces(offset, size int) []ptrace.Traces {
	otlp := ptrace.NewTraces()
	ssm := map[string]ptrace.ScopeSpans{}
	rsm := map[string]ptrace.ResourceSpans{}

	for _, span := range d.spans[offset : offset+size] {
		inres := d.s2r[span]
		inscope := d.s2s[span]

		inscopeID := ResourceAndScopeId(inres, inscope)
		outscope, ok := ssm[inscopeID]

		if !ok {
			inres := d.s2r[span]
			inresID := carrow.ResourceID(inres, "")
			outres, ok := rsm[inresID]

			if !ok {
				outres = otlp.ResourceSpans().AppendEmpty()
				inres.CopyTo(outres.Resource())
				rsm[inresID] = outres
			}

			outscope = outres.ScopeSpans().AppendEmpty()
			inscope.CopyTo(outscope.Scope())
			ssm[inscopeID] = outscope
		}

		span.CopyTo(outscope.Spans().AppendEmpty())
	}

	return []ptrace.Traces{otlp}
}

func v2s(v pcommon.Value) string {
	switch v.Type() {
	case pcommon.ValueTypeStr:
		return v.Str()
	case pcommon.ValueTypeBool:
		return fmt.Sprint(v.Bool())
	case pcommon.ValueTypeInt:
		return fmt.Sprint(v.Int())
	case pcommon.ValueTypeDouble:
		return fmt.Sprint(v.Double())
	case pcommon.ValueTypeBytes, pcommon.ValueTypeEmpty, pcommon.ValueTypeMap, pcommon.ValueTypeSlice:
		panic(fmt.Sprint("unsupported sorting value:", v.Type()))
	default:
		panic(fmt.Sprint("unsupported sorting value:", v.Type()))
	}
}

func (d *RealTraceDataset) getSortkey(field string, span ptrace.Span) (result string) {
	switch field {
	case "trace_id":
		tid := span.TraceID()
		return hex.EncodeToString(tid[:])
	case "span_id":
		sid := span.SpanID()
		return hex.EncodeToString(sid[:])
	default:
		// scan attributes next
	}

	span.Attributes().Range(func(key string, value pcommon.Value) bool {
		if key == field {
			result = v2s(value)
			return false
		}
		return true
	})
	if result != "" {
		return result
	}
	d.s2r[span].Attributes().Range(func(key string, value pcommon.Value) bool {
		if key == field {
			result = v2s(value)
			return false
		}
		return true
	})
	panic(fmt.Sprintf("missing getSortkey lookup: %v %v", field, span))
}

func (ss spanSorter) Len() int {
	return len(ss.spans)
}

func (ss spanSorter) Swap(i, j int) {
	ss.spans[i], ss.spans[j] = ss.spans[j], ss.spans[i]
}

func (ss spanSorter) Less(i, j int) bool {
	return strings.Compare(ss.getSortkey(ss.field, ss.spans[i]), ss.getSortkey(ss.field, ss.spans[j])) < 0
}
