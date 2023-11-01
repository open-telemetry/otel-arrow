package observer

import (
	"github.com/apache/arrow/go/v12/arrow"

	"github.com/open-telemetry/otel-arrow/pkg/record_message"
)

// ProducerObserver is an interface for observing the OTel Arrow producer.
type ProducerObserver interface {
	// OnDictionaryUpgrade is called when a dictionary index is upgraded.
	OnDictionaryUpgrade(recordName string, fieldPath string, prevIndexType, newIndexType arrow.DataType, card, total uint64)

	// OnDictionaryOverflow is called when a dictionary index overflows, i.e.
	// the cardinality of the dictionary exceeds the maximum cardinality of the
	// index type.
	// The column type is no longer a dictionary and is downgraded to its value
	// type.
	OnDictionaryOverflow(recordName string, fieldPath string, card, total uint64)

	// OnSchemaUpdate is called when the schema is updated.
	OnSchemaUpdate(recordName string, old, new *arrow.Schema)

	// OnRecord is called when a record is produced.
	OnRecord(arrow.Record, record_message.PayloadType)
}
