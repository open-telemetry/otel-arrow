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

package common

import (
	"math"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.opentelemetry.io/collector/pdata/pcommon"
)

func TestIntValues(t *testing.T) {
	t.Parallel()

	// Test small negative value
	expectedValue := pcommon.NewValueInt(-0)

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test big negative value
	expectedValue = pcommon.NewValueInt(math.MinInt64)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test small value
	expectedValue = pcommon.NewValueInt(0)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test big value
	expectedValue = pcommon.NewValueInt(math.MaxInt64)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)
}

func TestDoubleValues(t *testing.T) {
	t.Parallel()

	// Test small negative value
	expectedValue := pcommon.NewValueDouble(-0.0)

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test big negative value
	expectedValue = pcommon.NewValueDouble(-112334324234)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test small value
	expectedValue = pcommon.NewValueDouble(0.0)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	// Test big value
	expectedValue = pcommon.NewValueDouble(math.MaxFloat64)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)
}

func TestBoolValues(t *testing.T) {
	t.Parallel()

	expectedValue := pcommon.NewValueBool(true)

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	expectedValue = pcommon.NewValueBool(false)

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)
}

func TestStringValues(t *testing.T) {
	t.Parallel()

	expectedValue := pcommon.NewValueStr("")

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)

	expectedValue = pcommon.NewValueStr("string")

	cborData, err = Serialize(&expectedValue)
	assert.NoError(t, err)

	value = pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)
}

func TestBinaryValue(t *testing.T) {
	t.Parallel()

	expectedValue := pcommon.NewValueEmpty()
	expectedValue.SetEmptyBytes().Append([]byte("binary")...)

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue, value)
}

func TestCbor(t *testing.T) {
	t.Parallel()

	expectedValue := pcommon.NewValueMap()
	mapV := expectedValue.SetEmptyMap()
	mapV.PutDouble("double", 1.1)
	mapV.PutInt("int", 2)
	mapV.PutBool("bool", true)
	mapV.PutStr("string", "string1")
	mapV.PutEmpty("empty")
	mapV.PutEmptyBytes("binary").Append([]byte("binary")...)
	array := mapV.PutEmptySlice("array")
	array.AppendEmpty().SetInt(1)
	array.AppendEmpty().SetDouble(2.2)
	array.AppendEmpty().SetStr("3")
	array.AppendEmpty().SetBool(false)
	subMapV := array.AppendEmpty().SetEmptyMap()
	subMapV.PutDouble("double", 2.1)
	subMapV.PutInt("int", 3)
	subMapV.PutBool("bool", false)
	subMapV.PutStr("string", "string2")
	subMapV.PutEmpty("empty")
	subMapV.PutEmptyBytes("binary").Append([]byte("binary")...)
	subArray := subMapV.PutEmptySlice("array")
	subArray.AppendEmpty().SetInt(2)
	subArray.AppendEmpty().SetDouble(3.3)
	subArray.AppendEmpty().SetStr("3")
	subArray.AppendEmpty().SetBool(false)

	cborData, err := Serialize(&expectedValue)
	assert.NoError(t, err)

	value := pcommon.NewValueEmpty()
	err = Deserialize(cborData, value)
	assert.NoError(t, err)

	assert.Equal(t, expectedValue.AsRaw(), value.AsRaw())
}
