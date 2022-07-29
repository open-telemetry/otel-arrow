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

package trace

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow/ipc"
	"github.com/davecgh/go-spew/spew"

	coleventspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
)

type consumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

func NewConsumer() *consumer {
	bufReader := bytes.NewReader([]byte{})
	return &consumer{
		bufReader: bufReader,
	}
}

func (c *consumer) ConsumeEvent(event *coleventspb.BatchEvent) error {
	c.bufReader.Reset(event.OtlpArrowPayloads[0].Schema)
	if c.ipcReader == nil {
		ipcReader, err := ipc.NewReader(c.bufReader)
		if err != nil {
			return err
		}
		c.ipcReader = ipcReader
	}

	for c.ipcReader.Next() {
		out := c.ipcReader.Record()
		spew.Dump(out)
	}
	return nil
}
