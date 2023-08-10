This package supports validation of data that has passed through Arrow
encoding and decoding, recognizing that there may be structural
differences for semantically equivalent data.

Examples of transformations that are allowed by `assert.Equiv()`:

- Appearance of duplicate Resource, Scope, and Metric entities
- Order of Resource instances in a Resource list
- Order of Scope items in a Resource
- Order of Span, Metric, and LogRecord items in a Scope
- Order of Links/Events in a Span
- and so on.

The `assert.Equiv()` method in this package should be used for
unittesting and validation of data in an OTel Arrow pipeline.  See the
[code](equiv.go) for details.
