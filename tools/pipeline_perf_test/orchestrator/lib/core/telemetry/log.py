"""
Custom logging handler that integrates Python logging with OpenTelemetry tracing.

This module defines a logging handler that attaches log records as events
to the current active OpenTelemetry span, allowing for improved traceability
across distributed systems.
"""

import logging
from opentelemetry.trace import get_current_span, Span


class SpanAwareLogHandler(logging.Handler):
    """
    Logging handler that enriches the current OpenTelemetry span with log events.

    If a span is active and recording, log records are added to it as events.
    Certain custom keys (those starting with 'test.') are also included as attributes.
    """

    def emit(self, record: logging.LogRecord):
        """
        Emit a log record as an OpenTelemetry span event if a span is active.

        Args:
            record (logging.LogRecord): The log record to process.
        """
        span: Span = get_current_span()
        if span and span.is_recording():

            record_dict = record.__dict__.copy()

            attributes = {
                "log.severity": record.levelname,
                "log.message": record.getMessage(),
                "logger.name": record.name,
            }
            for key in list(record_dict.keys()):
                if "test." in key:
                    attributes[key] = record_dict[key]

            span.add_event(name="log", attributes=attributes)
