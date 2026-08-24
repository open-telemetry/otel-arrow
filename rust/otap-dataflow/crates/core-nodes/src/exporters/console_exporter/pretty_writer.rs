// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unbounded styled writer for data-plane console output.

use otap_df_pdata_views::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_telemetry::self_tracing::{AnsiCode, ColorMode};
use std::io::{self, Write};

/// Styled writer that delegates capacity and failure behavior to its output.
pub(super) struct PrettyWriter<'a, W: Write + ?Sized> {
    output: &'a mut W,
    color_mode: ColorMode,
}

impl<'a, W: Write + ?Sized> PrettyWriter<'a, W> {
    /// Create a writer over the supplied output.
    pub(super) const fn new(output: &'a mut W, color_mode: ColorMode) -> Self {
        Self { output, color_mode }
    }

    /// Write content with ANSI styling and reset the style afterward.
    pub(super) fn write_styled(
        &mut self,
        code: AnsiCode,
        format: impl FnOnce(&mut Self) -> io::Result<()>,
    ) -> io::Result<()> {
        self.write_ansi(code)?;
        let format_result = format(self);
        let reset_result = self.write_ansi(AnsiCode::Reset);
        format_result.and(reset_result)
    }

    /// Write attributes with a leading separator.
    pub(super) fn write_attrs<A: AttributeView>(
        &mut self,
        attrs: impl Iterator<Item = A>,
    ) -> io::Result<()> {
        let mut attrs = attrs.peekable();
        if attrs.peek().is_none() {
            return Ok(());
        }

        self.write_all(b" [")?;
        for (index, attr) in attrs.enumerate() {
            if index > 0 {
                self.write_all(b", ")?;
            }
            self.write_all(attr.key())?;
            self.write_all(b"=")?;
            match attr.value() {
                Some(value) => self.write_any_value(&value)?,
                None => self.write_all(b"<?>")?,
            }
        }
        self.write_all(b"]")
    }

    /// Terminate the current output line.
    pub(super) fn finish_line(&mut self) -> io::Result<()> {
        self.write_all(b"\n")
    }

    fn write_ansi(&mut self, code: AnsiCode) -> io::Result<()> {
        if let ColorMode::Color = self.color_mode {
            write!(self, "\x1b[{}m", code as u8)?;
        }
        Ok(())
    }

    fn write_any_value<'b>(&mut self, value: &impl AnyValueView<'b>) -> io::Result<()> {
        match value.value_type() {
            ValueType::String => {
                if let Some(value) = value.as_string() {
                    self.write_all(value)?;
                }
            }
            ValueType::Int64 => {
                if let Some(value) = value.as_int64() {
                    write!(self, "{value}")?;
                }
            }
            ValueType::Bool => {
                if let Some(value) = value.as_bool() {
                    self.write_all(if value { b"true" } else { b"false" })?;
                }
            }
            ValueType::Double => {
                if let Some(value) = value.as_double() {
                    write!(self, "{value:.6}")?;
                }
            }
            ValueType::Bytes => {
                if let Some(bytes) = value.as_bytes() {
                    self.write_all(b"[")?;
                    for (index, byte) in bytes.iter().enumerate() {
                        if index > 0 {
                            self.write_all(b", ")?;
                        }
                        write!(self, "{byte}")?;
                    }
                    self.write_all(b"]")?;
                }
            }
            ValueType::Array => {
                self.write_all(b"[")?;
                if let Some(values) = value.as_array() {
                    for (index, item) in values.enumerate() {
                        if index > 0 {
                            self.write_all(b", ")?;
                        }
                        self.write_any_value(&item)?;
                    }
                }
                self.write_all(b"]")?;
            }
            ValueType::KeyValueList => {
                self.write_all(b"{")?;
                if let Some(values) = value.as_kvlist() {
                    for (index, item) in values.enumerate() {
                        if index > 0 {
                            self.write_all(b", ")?;
                        }
                        self.write_all(item.key())?;
                        if let Some(value) = item.value() {
                            self.write_all(b"=")?;
                            self.write_any_value(&value)?;
                        }
                    }
                }
                self.write_all(b"}")?;
            }
            ValueType::Empty => {}
        }
        Ok(())
    }
}

impl<W: Write + ?Sized> Write for PrettyWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
