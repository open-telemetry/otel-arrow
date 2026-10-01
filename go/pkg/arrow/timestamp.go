/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow

import (
	"fmt"

	"github.com/apache/arrow-go/v18/arrow"

	"github.com/open-telemetry/otel-arrow/go/pkg/werror"
)

const (
	// TimestampTimeZone is the time zone that OTAP producers must attach to
	// every timestamp column. See section 5.5.2 of the OTAP specification.
	TimestampTimeZone = "UTC"

	// TimestampTimeZoneOffset is the equivalent fixed-offset spelling of
	// TimestampTimeZone. Producers emit TimestampTimeZone, but consumers also
	// accept this form.
	TimestampTimeZoneOffset = "+00:00"
)

// ValidTimestampTimeZone reports whether timeZone is an OTAP-conformant time
// zone for a timestamp column.
//
// Per section 5.5.2 of the OTAP specification, producers must emit
// TimestampTimeZone. Consumers accept "UTC" and "+00:00".
//
// An empty time zone is accepted as a transitional allowance so that a
// producer which has not yet upgraded to tagging its timestamp columns stays
// interoperable during a rolling upgrade. That allowance is temporary and will
// be removed, after which a missing time zone is invalid.
// TODO: Remove the empty time zone allowance once producers have upgraded.
// See https://github.com/open-telemetry/otel-arrow/issues/2369.
func ValidTimestampTimeZone(timeZone string) bool {
	switch timeZone {
	case TimestampTimeZone, TimestampTimeZoneOffset, "":
		return true
	default:
		return false
	}
}

// ValidateTimestampType returns an error if dt is not a timestamp type that
// conforms to the OTAP specification, i.e. nanosecond precision tagged with the
// UTC time zone.
func ValidateTimestampType(dt *arrow.TimestampType) error {
	if dt.Unit != arrow.Nanosecond {
		return werror.WrapWithMsg(ErrInvalidTimestampType,
			fmt.Sprintf("timestamp must use nanosecond precision, got %s", dt.Unit))
	}

	if !ValidTimestampTimeZone(dt.TimeZone) {
		return werror.WrapWithMsg(ErrInvalidTimestampType,
			fmt.Sprintf("timestamp time zone must be %q or %q, got %q",
				TimestampTimeZone, TimestampTimeZoneOffset, dt.TimeZone))
	}

	return nil
}
