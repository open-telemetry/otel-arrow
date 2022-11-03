package arrow

import "github.com/apache/arrow/go/v10/arrow"

var (
	DictU16String = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}

	DictU16Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.Binary,
		Ordered:   false,
	}

	DictU16Fixed16Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 16},
		Ordered:   false,
	}

	DictU16Fixed8Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 8},
		Ordered:   false,
	}
)
