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

package benchmark

import (
	"fmt"
	"io"
	"strings"
)

type ProfileableSystem interface {
	Name() string
	Tags() []string
	DatasetSize() int
	CompressionAlgorithm() CompressionAlgorithm

	StartProfiling(writer io.Writer)
	EndProfiling(writer io.Writer)

	InitBatchSize(writer io.Writer, batchSize int)
	PrepareBatch(writer io.Writer, startAt, size int)
	ConvertOtlpToOtlpArrow(writer io.Writer, startAt, size int)

	Process(writer io.Writer) string

	Serialize(writer io.Writer) ([][]byte, error)
	Deserialize(writer io.Writer, buffers [][]byte)
	ConvertOtlpArrowToOtlp(writer io.Writer)

	Clear()
	ShowStats()
}

func ProfileableSystemID(ps ProfileableSystem) string {
	return fmt.Sprintf("%s:%s", ps.Name(), strings.Join(ps.Tags()[:], "+"))
}
