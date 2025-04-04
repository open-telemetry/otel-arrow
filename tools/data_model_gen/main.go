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
	"os"
	"strings"
	"text/template"

	"github.com/apache/arrow-go/v18/arrow"

	carrow "github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema"
	logsarrow "github.com/open-telemetry/otel-arrow/pkg/otel/logs/arrow"
	metrics "github.com/open-telemetry/otel-arrow/pkg/otel/metrics/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
	traces "github.com/open-telemetry/otel-arrow/pkg/otel/traces/arrow"
)

const (
	Metrics = "Metrics"
	Logs    = "Logs"
	Traces  = "Traces"
)

type (
	Domains struct {
		domains []*Domain
	}

	Domain struct {
		name      string
		records   map[string]*Record
		relations []*Relation
	}

	Record struct {
		domain *Domain
		name   string
		fields []*Field
	}

	Relation struct {
		label           string
		from            string
		to              string
		fromCardinality string
		toCardinality   string
	}

	Field struct {
		name     string
		dataType string
		pk       bool
		fk       bool
		comments []string
	}

	DataModel struct {
		Metrics string
		Logs    string
		Traces  string
	}
)

func main() {
	domains := NewDomains()

	VisitMetricsDataModel(domains.Domain(Metrics))
	VisitLogsDataModel(domains.Domain(Logs))
	VisitTracesDataModel(domains.Domain(Traces))

	tmpl := template.Must(template.ParseFiles("tools/data_model_gen/data_model.tmpl"))

	// Write the content of generated Markdown to the `docs/data_model.md` file
	f, err := os.Create("docs/data_model.md")
	check(err)
	defer func() { check(f.Close()) }()

	err = tmpl.Execute(f, domains.ToDataModel())
	check(err)
}

func check(err error) {
	if err != nil {
		panic(err)
	}
}

func VisitMetricsDataModel(domain *Domain) {
	mainSchema := metrics.MetricsSchema
	relatedData, err := metrics.NewRelatedData(metrics.DefaultConfig(), stats.NewProducerStats(), nil)
	if err != nil {
		panic(err)
	}

	mainRecord := domain.RecordByPayloadType(carrow.PayloadTypes.Metrics)
	VisitSchema(mainRecord, mainSchema)
	for _, relatedSchema := range relatedData.Schemas() {
		relatedRecord := domain.RecordByPayloadType(relatedSchema.PayloadType)
		domain.OneToManyRelation(relatedSchema.ParentPayloadType, relatedSchema.PayloadType)
		VisitSchema(relatedRecord, relatedSchema.Schema)
	}
}

func VisitLogsDataModel(domain *Domain) {
	mainSchema := logsarrow.LogsSchema
	relatedData, err := logsarrow.NewRelatedData(logsarrow.DefaultConfig(), stats.NewProducerStats(), nil)
	if err != nil {
		panic(err)
	}

	mainRecord := domain.RecordByPayloadType(carrow.PayloadTypes.Logs)
	VisitSchema(mainRecord, mainSchema)
	for _, relatedSchema := range relatedData.Schemas() {
		relatedRecord := domain.RecordByPayloadType(relatedSchema.PayloadType)
		domain.OneToManyRelation(relatedSchema.ParentPayloadType, relatedSchema.PayloadType)
		VisitSchema(relatedRecord, relatedSchema.Schema)
	}
}

func VisitTracesDataModel(domain *Domain) {
	mainSchema := traces.TracesSchema
	relatedData, err := traces.NewRelatedData(traces.DefaultConfig(), stats.NewProducerStats(), nil)
	if err != nil {
		panic(err)
	}

	mainRecord := domain.RecordByPayloadType(carrow.PayloadTypes.Spans)
	VisitSchema(mainRecord, mainSchema)
	for _, relatedSchema := range relatedData.Schemas() {
		relatedRecord := domain.RecordByPayloadType(relatedSchema.PayloadType)
		domain.OneToManyRelation(relatedSchema.ParentPayloadType, relatedSchema.PayloadType)
		VisitSchema(relatedRecord, relatedSchema.Schema)
	}
}

func VisitSchema(record *Record, schema *arrow.Schema) {
	for _, field := range schema.Fields() {
		visitField(record, "", &field)
	}
}

func visitStructType(record *Record, path string, dt *arrow.StructType) {
	for _, child := range dt.Fields() {
		visitField(record, path, &child)
	}
}

func visitField(record *Record, path string, arrowField *arrow.Field) {
	var fieldName string
	if path != "" {
		fieldName = path + "_" + arrowField.Name
	} else {
		fieldName = arrowField.Name
	}

	switch dt := arrowField.Type.(type) {
	case *arrow.DictionaryType:
		field := record.Field(fieldName)
		field.Comment(fmt.Sprintf("dict<%s>", dt.IndexType.Name()))
		field.DataType(fmt.Sprintf("%s", dt.ValueType.Name()))
	case *arrow.ListType:
		structType, ok := dt.Elem().(*arrow.StructType)
		if ok {
			child := record.OneToMany(arrowField.Name, arrowField.Name)
			visitStructType(child, path, structType)
			return
		} else {
			record.Field(fieldName).DataType(DataTypeToString(dt.Elem()))
		}
	case *arrow.StructType:
		path += arrowField.Name
		visitStructType(record, path, dt)
		return
	case *arrow.SparseUnionType:
		field := record.Field(fieldName)
		unionType := ""
		for _, child := range dt.Fields() {
			if len(unionType) > 0 {
				unionType += "|"
			}
			unionType += child.Name
		}
		if len(unionType) > 0 {
			field.Comment(unionType)
		}
		field.DataType("union")
	case *arrow.MapType:
		field := record.Field(fieldName)
		field.Comment(fmt.Sprintf("map<%s, %s>", dt.KeyType().Name(), dt.ItemType().Name()))
		field.DataType("map")
	default:
		record.Field(fieldName).DataType(DataTypeToString(arrowField.Type))
	}

	if arrowField.Metadata.FindKey(schema.OptionalKey) != -1 {
		record.Field(fieldName).Comment("optional")
	}
}

func DataTypeToString(dataType arrow.DataType) string {
	switch dt := dataType.(type) {
	case *arrow.BooleanType:
		return "bool"
	case *arrow.Int8Type:
		return "i8"
	case *arrow.Int16Type:
		return "i16"
	case *arrow.Int32Type:
		return "i32"
	case *arrow.Int64Type:
		return "i64"
	case *arrow.Uint8Type:
		return "u8"
	case *arrow.Uint16Type:
		return "u16"
	case *arrow.Uint32Type:
		return "u32"
	case *arrow.Uint64Type:
		return "u64"
	case *arrow.Float16Type:
		return "f16"
	case *arrow.Float32Type:
		return "f32"
	case *arrow.Float64Type:
		return "f64"
	case *arrow.StringType:
		return "string"
	case *arrow.BinaryType:
		return "bytes"
	case *arrow.Date32Type:
		return "date32"
	case *arrow.Date64Type:
		return "date64"
	case *arrow.TimestampType:
		return "timestamp"
	case *arrow.Time32Type:
		return "time32"
	case *arrow.Time64Type:
		return "time64"
	case *arrow.DurationType:
		return "duration"
	case *arrow.FixedSizeBinaryType:
		return fmt.Sprintf("bytes[%d]", dt.ByteWidth)
	default:
		panic(fmt.Sprintf("Unknown type: %v", dt))
	}
}

func NewDomains() *Domains {
	return &Domains{
		domains: make([]*Domain, 0),
	}
}

func (d *Domains) Domain(name string) *Domain {
	for _, domain := range d.domains {
		if domain.name == name {
			return domain
		}
	}

	d.domains = append(d.domains, &Domain{
		name:    name,
		records: make(map[string]*Record),
	})

	return d.domains[len(d.domains)-1]
}

func (d *Domains) ToDataModel() DataModel {
	return DataModel{
		Metrics: d.Domain(Metrics).ToMarkdown(),
		Logs:    d.Domain(Logs).ToMarkdown(),
		Traces:  d.Domain(Traces).ToMarkdown(),
	}
}

func (d *Domain) RecordByPayloadType(payloadType *carrow.PayloadType) *Record {
	name := payloadType.PayloadType().String()
	return d.RecordByName(name)
}

func (d *Domain) RecordByName(name string) *Record {
	record, ok := d.records[name]
	if !ok {
		record = &Record{
			domain: d,
			name:   name,
		}
		d.records[name] = record
	}
	return record
}

func (d *Domain) Relation(from, to, fromCardinality, toCardinality string, label string) {
	d.relations = append(d.relations, &Relation{
		from:            from,
		to:              to,
		fromCardinality: fromCardinality,
		toCardinality:   toCardinality,
		label:           label,
	})
}

func (d *Domain) OneToManyRelation(from *carrow.PayloadType, to *carrow.PayloadType) {
	d.relations = append(d.relations, &Relation{
		from:            from.PayloadType().String(),
		to:              to.PayloadType().String(),
		fromCardinality: "||",
		toCardinality:   "o{",
		label:           to.SchemaPrefix(),
	})
}

func (d *Domain) ToMarkdown() string {
	md := "```mermaid\n"
	md += "erDiagram\n"

	for _, relation := range d.relations {
		md += relation.ToMermaid("    ")
	}

	for _, record := range d.records {
		md += record.ToMermaid("    ")
	}

	md += "```"

	return md
}

func (r *Relation) ToMermaid(indent string) string {
	return fmt.Sprintf("%s%s %s--%s %s : %s\n", indent, r.from, r.fromCardinality, r.toCardinality, r.to, r.label)
}

func (r *Record) OneToMany(name, label string) *Record {
	r.domain.Relation(r.name, name, "||", "o{", label)
	return r.domain.RecordByName(name)
}

func (r *Record) Field(name string) *Field {
	// returns existing field if it exists
	for _, f := range r.fields {
		if f.name == name {
			return f
		}
	}

	field := &Field{
		name: name,
	}
	r.fields = append(r.fields, field)
	return field
}

func (r *Record) ToMermaid(indent string) string {
	mermaid := "    " + r.name + "{\n"
	for _, field := range r.fields {
		mermaid += field.ToMermaid(indent + "    ")
	}
	mermaid += "    }\n"

	return mermaid
}

func (f *Field) DataType(dataType string) *Field {
	f.dataType = dataType
	return f
}

func (f *Field) Comment(comment string) *Field {
	f.comments = append(f.comments, comment)
	return f
}

func (f *Field) ToMermaid(indent string) string {
	comments := ""
	if len(f.comments) > 0 {
		comments = "\"" + strings.Join(f.comments, ", ") + "\""
	}
	return fmt.Sprintf("%s%s %s %s\n", indent, f.name, f.dataType, comments)
}
