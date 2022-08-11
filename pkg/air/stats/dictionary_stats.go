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

package stats

import (
	"fmt"
	"os"
)

type DictionaryStatsSlice []*DictionaryStats

// Sort interface
func (d DictionaryStatsSlice) Less(i, j int) bool {
	a := d[i]
	b := d[j]
	a_ratio := float64(a.Cardinality) / float64(a.TotalEntry)
	b_ratio := float64(b.Cardinality) / float64(b.TotalEntry)
	if a_ratio == b_ratio {
		return a.AvgEntryLength > b.AvgEntryLength
	} else {
		return a_ratio < b_ratio
	}
}
func (d DictionaryStatsSlice) Len() int      { return len(d) }
func (d DictionaryStatsSlice) Swap(i, j int) { d[i], d[j] = d[j], d[i] }

type DictionaryType int64

const (
	StringDic DictionaryType = iota
	BinaryDic
)

func (c DictionaryType) String() string {
	switch c {
	case StringDic:
		return "StringDic"
	case BinaryDic:
		return "BinaryDic"
	default:
		return "Unknown"
	}
}

type DictionaryStats struct {
	Type           DictionaryType
	NumPath        []int
	StringPath     string
	AvgEntryLength float64
	Cardinality    int
	TotalEntry     int
}

func (s DictionaryStats) Dump(f *os.File) {
	_, err := f.WriteString(fmt.Sprintf("Dictionary type=%s, path=%v, avg-entry-length=%f, card=%d, total-entry=%d\n", s.Type,
		s.StringPath, s.AvgEntryLength, s.Cardinality, s.TotalEntry))
	if err != nil {
		panic(err)
	}
}
