/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
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
	require.Equal(t, "github.com/open-telemetry/otel-arrow/go/pkg/werror.Level1a:40->github.com/open-telemetry/otel-arrow/go/pkg/werror.Level2:48{id=1}->test error", err.Error())

	err = Level1b()
	require.Equal(t, "github.com/open-telemetry/otel-arrow/go/pkg/werror.Level1b:44->github.com/open-telemetry/otel-arrow/go/pkg/werror.Level2:48{id=2}->test error", err.Error())
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
