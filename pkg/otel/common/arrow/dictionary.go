package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow/array"
)

// AdaptiveDictionaryBuilder is a wrapper around arrow builders that
// can be used to append values to a dictionary builder or a regular
// builder (string or binary) depending on the type of the builder.
type AdaptiveDictionaryBuilder struct {
	builder array.Builder
}

// AdaptiveDictionaryBuilderFrom creates a new AdaptiveDictionaryBuilder from an existing builder.
func AdaptiveDictionaryBuilderFrom(builder array.Builder) *AdaptiveDictionaryBuilder {
	return &AdaptiveDictionaryBuilder{builder: builder}
}

// AppendString appends a string value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendString(s string) error {
	switch b := b.builder.(type) {
	case *array.BinaryDictionaryBuilder:
		return b.AppendString(s)
	case *array.StringBuilder:
		b.Append(s)
		return nil
	default:
		return fmt.Errorf("AdaptiveDictionaryBuilder: unsupported builder type: %T", b)
	}
}

// AppendBinary appends a binary value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendBinary(v []byte) error {
	switch b := b.builder.(type) {
	case *array.BinaryDictionaryBuilder:
		return b.Append(v)
	case *array.FixedSizeBinaryDictionaryBuilder:
		return b.Append(v)
	case *array.BinaryBuilder:
		b.Append(v)
		return nil
	default:
		return fmt.Errorf("AdaptiveDictionaryBuilder: unsupported builder type: %T", b)
	}
}

// AppendNull appends a null value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendNull() {
	b.builder.AppendNull()
}

// Release releases the underlying builder.
func (b *AdaptiveDictionaryBuilder) Release() {
	b.builder.Release()
}
