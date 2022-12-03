package arrow

import "github.com/apache/arrow/go/v10/arrow"

var (
	DefaultDictString = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}

	DefaultDictBinary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.Binary,
		Ordered:   false,
	}

	DefaultDictFixed16Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 16},
		Ordered:   false,
	}

	DefaultDictFixed8Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 8},
		Ordered:   false,
	}
)
