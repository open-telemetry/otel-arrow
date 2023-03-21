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

package werror

import (
	"fmt"
	"runtime"
	"strconv"
	"strings"
)

// Wrapper wraps an error with the file, line, and function where the error
// was wrapped and an optional context map.
type Wrapper struct {
	// The wrapped error.
	err error

	// The file, line, and function where the error was wrapped.
	file     string
	line     int
	function string

	// An optional context map.
	context map[string]interface{}
}

// Error returns the wrapped error's message.
func (w Wrapper) Error() string {
	var msg strings.Builder

	msg.WriteString(w.function)
	msg.WriteString(":")
	msg.WriteString(strconv.Itoa(w.line))

	if w.context != nil {
		msg.WriteString("{")
		for k, v := range w.context {
			msg.WriteString(k)
			msg.WriteString("=")
			msg.WriteString(fmt.Sprintf("%v", v))
		}
		msg.WriteString("}")
	}

	if w.err != nil {
		msg.WriteString("->")
		msg.WriteString(w.err.Error())
	}

	return msg.String()
}

// Unwrap returns the wrapped error.
func (w Wrapper) Unwrap() error {
	return w.err
}

// File returns the file where the error was wrapped.
func (w Wrapper) File() string {
	return w.file
}

// Line returns the line where the error was wrapped.
func (w Wrapper) Line() int {
	return w.line
}

// Function returns the function where the error was wrapped.
func (w Wrapper) Function() string {
	return w.function
}

// Wrap wraps the given error with the current file, line, and function.
func Wrap(err error) error {
	return WrapWithContext(err, nil)
}

// WrapWithContext wraps the given error with the current file, line, function,
// and the given context.
func WrapWithContext(err error, context map[string]interface{}) error {
	if err == nil {
		return nil
	}

	pc, file, line, _ := runtime.Caller(1)
	fn := runtime.FuncForPC(pc)

	return Wrapper{
		err:      err,
		file:     file,
		line:     line,
		function: fn.Name(),
		context:  context,
	}
}

func WrapWithMsg(err error, msg string) error {
	if err == nil {
		return nil
	}

	return WrapWithContext(err, map[string]interface{}{"msg": msg})
}
