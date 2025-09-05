/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

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
