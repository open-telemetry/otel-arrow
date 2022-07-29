/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package events

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/ipc"

	coleventspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
)

type consumer struct {
	streamConsumers map[string]*streamConsumer
}

type streamConsumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

func NewConsumer() *consumer {
	return &consumer{
		streamConsumers: make(map[string]*streamConsumer),
	}
}

func (c *consumer) ConsumeEvent(event *coleventspb.BatchEvent) ([]arrow.Record, error) {
	// ToDo return XYZ_ServiceRequest and BatchId

	sc := c.streamConsumers[event.SubStreamId]
	if sc == nil {
		bufReader := bytes.NewReader([]byte{})
		sc = &streamConsumer{
			bufReader: bufReader,
		}
	}

	sc.bufReader.Reset(event.OtlpArrowPayloads[0].Schema)
	if sc.ipcReader == nil {
		ipcReader, err := ipc.NewReader(sc.bufReader)
		if err != nil {
			return nil, err
		}
		sc.ipcReader = ipcReader
	}

	var records []arrow.Record

	for sc.ipcReader.Next() {
		rec := sc.ipcReader.Record()
		records = append(records, rec)
	}
	return records, nil
}
