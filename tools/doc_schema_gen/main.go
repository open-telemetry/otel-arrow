/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package main

import (
	"fmt"
	"strings"

	"github.com/apache/arrow/go/v12/arrow"

	logsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	metricsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
)

func main() {
	sdg := NewSchemaDocGenerator().
		reusableField("attributes", "shared_attributes").
		reusableField("exemplars").
		comment("flags", "used as a bit mask").
		comment("shared_attributes", "inherited by data points").
		comment("shared_start_time_unix_nano", "inherited by data points").
		comment("shared_time_unix_nano", "inherited by data points").
		comment("severity_number", "OTLP enum with 25 variants").
		comment("aggregation_temporality", "OTLP enum with 3 variants").
		comment("kind", "OTLP enum with 6 variants").
		comment("code", "OTLP enum with 4 variants")

	sdg.genSchemaDoc(metricsarrow.MetricsSchema, 2)
	sdg.genSchemaDoc(logsarrow.Schema, 2)
	sdg.genSchemaDoc(tracesarrow.Schema, 2)

	sdg.printDoc(2)
}

type ReusableDoc struct {
	Name string
	doc  *[]Line
}
type SchemaDocGenerator struct {
	reusableFields  map[string]*ReusableDoc
	defaultComments map[string]string
	doc             []Line
	currentDoc      *[]Line
}

func NewSchemaDocGenerator() *SchemaDocGenerator {
	sdg := SchemaDocGenerator{
		reusableFields:  make(map[string]*ReusableDoc),
		defaultComments: make(map[string]string),
		doc:             make([]Line, 0, 100),
	}
	sdg.currentDoc = &sdg.doc
	return &sdg
}

type Line struct {
	indent  string
	field   string
	comment string
}

func (sdg *SchemaDocGenerator) printDoc(minSpaceBeforeComment int) {
	printDocElement := func() {
		var fmtLines []*strings.Builder
		maxLen := 0

		for _, line := range *sdg.currentDoc {
			var fmtLine strings.Builder
			fmtLine.WriteString(fmt.Sprintf("%s%s", line.indent, line.field))
			if fmtLine.Len() > maxLen {
				maxLen = fmtLine.Len()
			}
			fmtLines = append(fmtLines, &fmtLine)
		}

		charsBeforeComment := maxLen + minSpaceBeforeComment
		for i, line := range *sdg.currentDoc {
			fmtLine := fmtLines[i]
			spaces := charsBeforeComment - fmtLine.Len()
			if len(line.comment) > 0 {
				fmtLine.WriteString(strings.Repeat(" ", spaces))
				fmtLine.WriteString(fmt.Sprintf("# %s", line.comment))
			}
			println(fmtLine.String())
		}
	}

	for k, rdoc := range sdg.reusableFields {
		if k == rdoc.Name {
			sdg.currentDoc = rdoc.doc
			printDocElement()
			println("---\n")
		}
	}

	sdg.currentDoc = &sdg.doc
	printDocElement()
}

func (sdg *SchemaDocGenerator) reusableField(fieldNames ...string) *SchemaDocGenerator {
	doc := make([]Line, 0, 100)
	rdoc := ReusableDoc{
		Name: fieldNames[0],
		doc:  &doc,
	}
	for _, fieldName := range fieldNames {
		sdg.reusableFields[fieldName] = &rdoc
	}
	return sdg
}

func (sdg *SchemaDocGenerator) comment(fieldName string, comment string) *SchemaDocGenerator {
	sdg.defaultComments[fieldName] = comment
	return sdg
}

func (sdg *SchemaDocGenerator) genSchemaDoc(schema *arrow.Schema, spacesPerIndent int) {
	for _, field := range schema.Fields() {
		sdg.genFieldDoc("", spacesPerIndent, false, "", &field)
	}
	*sdg.currentDoc = append(*sdg.currentDoc, Line{indent: "", field: "---\n", comment: ""})
}

func (sdg *SchemaDocGenerator) genFieldDoc(indent string, spacesPerIndent int, inList bool, prefix string, field *arrow.Field) {
	reusableFieldDoc, reusableFieldFound := sdg.reusableFields[field.Name]
	localCurrentDoc := sdg.currentDoc
	if reusableFieldFound {
		sdg.currentDoc = reusableFieldDoc.doc
		defer func() { sdg.currentDoc = localCurrentDoc }()
	}

	// Mute intermediary level in Arrow list
	if !inList {
		yamlType, comment := sdg.arrowTypeToYamlType(field.Type)

		if reusableFieldFound {
			*localCurrentDoc = append(*localCurrentDoc, Line{indent: indent, field: fmt.Sprintf("%s%s: *%s", prefix, field.Name, reusableFieldDoc.Name), comment: sdg.genComment(field, comment)})
			if len(*reusableFieldDoc.doc) == 0 {
				*reusableFieldDoc.doc = append(*reusableFieldDoc.doc, Line{indent: "", field: fmt.Sprintf("%s%s: &%s", prefix, field.Name, reusableFieldDoc.Name), comment: sdg.genComment(field, comment)})
				indent = ""
			} else {
				return
			}
		} else {
			*sdg.currentDoc = append(*sdg.currentDoc, Line{indent: indent, field: fmt.Sprintf("%s%s: %s", prefix, field.Name, yamlType), comment: sdg.genComment(field, comment)})
		}
	}

	// List indentation must not be propagated to children
	indent = strings.ReplaceAll(indent, "-", " ")

	switch t := field.Type.(type) {
	case *arrow.StructType:
		indent += strings.Repeat(" ", spacesPerIndent)
		for i, child := range t.Fields() {
			newIndent := indent
			if inList {
				if i == 0 {
					newIndent += "- "
				} else {
					newIndent += "  "
				}
			}
			sdg.genFieldDoc(newIndent, spacesPerIndent, false, prefix, &child)
		}
	case *arrow.ListType:
		sdg.genFieldDoc(indent, spacesPerIndent, true, "", &arrow.Field{Name: "", Type: t.Elem()})
	case *arrow.SparseUnionType:
		indent += strings.Repeat(" ", spacesPerIndent)
		for _, child := range t.Fields() {
			sdg.genFieldDoc(indent, spacesPerIndent, false, prefix, &child)
		}
	case *arrow.DenseUnionType:
		indent += strings.Repeat(" ", spacesPerIndent)
		for _, child := range t.Fields() {
			sdg.genFieldDoc(indent, spacesPerIndent, false, prefix, &child)
		}
	case *arrow.MapType:
		indent += strings.Repeat(" ", spacesPerIndent)
		sdg.genFieldDoc(indent, spacesPerIndent, false, prefix, &arrow.Field{Name: "key", Type: t.KeyType()})
		sdg.genFieldDoc(indent, spacesPerIndent, false, prefix, &arrow.Field{Name: "value", Type: t.ItemType()})
	}
}

func (sdg *SchemaDocGenerator) genComment(field *arrow.Field, otherComment string) string {
	addComment := func(comment string, otherComment string) string {
		if len(comment) > 0 {
			comment += ", "
		}
		comment += otherComment
		return comment
	}

	comment := ""
	switch t := field.Type.(type) {
	//case *arrow.StructType:
	//	comment = "arrow struct"
	case *arrow.ListType:
		dt, _ := sdg.arrowTypeToYamlType(t.Elem())
		comment = fmt.Sprintf("arrow list of %s", dt)
	case *arrow.SparseUnionType:
		comment = "arrow sparse union"
	case *arrow.DenseUnionType:
		comment = "arrow dense union"
	case *arrow.MapType:
		comment = "arrow map"
	}
	if len(otherComment) > 0 {
		comment = addComment(comment, otherComment)
	}
	if defaultComment, found := sdg.defaultComments[field.Name]; found {
		comment = addComment(comment, defaultComment)
	}
	return comment
}

func (sdg *SchemaDocGenerator) arrowTypeToYamlType(dt arrow.DataType) (string, string) {
	switch t := dt.(type) {
	case *arrow.Uint8Type:
		return "uint8", ""
	case *arrow.Uint16Type:
		return "uint16", ""
	case *arrow.Uint32Type:
		return "uint32", ""
	case *arrow.Uint64Type:
		return "uint64", ""
	case *arrow.Int8Type:
		return "int8", ""
	case *arrow.Int16Type:
		return "int16", ""
	case *arrow.Int32Type:
		return "int32", ""
	case *arrow.Int64Type:
		return "int64", ""
	case *arrow.Float32Type:
		return "float32", ""
	case *arrow.Float64Type:
		return "float64", ""
	case *arrow.StringType:
		return "string", ""
	case *arrow.BinaryType:
		return "binary", ""
	case *arrow.BooleanType:
		return "bool", ""
	case *arrow.TimestampType:
		return "timestamp", "time unit nanoseconds"
	case *arrow.DictionaryType:
		baseType, comment := sdg.arrowTypeToYamlType(t.ValueType)
		if len(comment) > 0 {
			comment += ", "
		}
		comment += fmt.Sprintf("%s_dictionary by default, fallback to %s when cardinality too high", baseType, baseType)
		return fmt.Sprintf("%s_dictionary | %s", baseType, baseType), comment
	case *arrow.FixedSizeBinaryType:
		return fmt.Sprintf("%d_bytes_binary", t.ByteWidth), "arrow fixed size binary array"
	case *arrow.StructType:
		return "", "struct"
	}
	return "", ""
}
