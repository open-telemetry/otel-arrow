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
	"errors"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWError(t *testing.T) {
	t.Parallel()

	err := Level1a()
	require.Equal(t, "github.com/f5/otel-arrow-adapter/pkg/werror.Level1a:40->github.com/f5/otel-arrow-adapter/pkg/werror.Level2:48{id=1}->test error", err.Error())

	err = Level1b()
	require.Equal(t, "github.com/f5/otel-arrow-adapter/pkg/werror.Level1b:44->github.com/f5/otel-arrow-adapter/pkg/werror.Level2:48{id=2}->test error", err.Error())
}

var ErrTest = errors.New("test error")

func Level1a() error {
	return Wrap(Level2(1))
}

func Level1b() error {
	return Wrap(Level2(2))
}

func Level2(id int) error {
	return WrapWithContext(ErrTest, map[string]interface{}{"id": id})
}
