// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ADX-private resource admission. Prost remains responsible for complete
//! validation, string validity, message merging, and oneof selection.

use prost::DecodeError;
use prost::encoding::{WireType, check_wire_type, decode_key, decode_varint};

mod schema;

// Leave room for Prost's full recursion allowance and a packed-field frame.
const MAX_ADMISSION_FRAMES: usize = 128;

/// A malformed source or a resource admission limit.
#[derive(Debug, thiserror::Error)]
pub(super) enum AdmissionError {
    /// Invalid protobuf framing or scalar encoding.
    #[error(transparent)]
    Protobuf(#[from] DecodeError),
    /// Invalid nested framing.
    #[error("{0}")]
    Invalid(&'static str),
    /// Estimated decoded storage exceeds the caller's limit.
    #[error("OTLP decoded allocation estimate exceeds {0} bytes")]
    Allocation(usize),
    /// Field and packed-element processing exceeds the caller's limit.
    #[error("OTLP decoding work exceeds {0} fields or elements")]
    Work(usize),
}

/// Allocation layouts kept private to the ADX exporter.
pub(super) trait DecodeSchema {
    /// Returns the allocation shape of a known field.
    fn field(tag: u32) -> Option<Field>;

    #[cfg(test)]
    fn check_fields();
}

/// Allocation-relevant field information used by admission.
#[derive(Clone, Copy)]
pub(super) enum Field {
    /// A nested message, with its inline size and generated field lookup.
    Message {
        /// The child message's field lookup.
        schema: fn(u32) -> Option<Field>,
        /// Size of one decoded child.
        size: usize,
    },
    /// A string or byte buffer.
    Bytes {
        /// Whether a vector of buffers is allocated.
        repeated: bool,
        /// Inline size of one buffer.
        size: usize,
    },
    /// A scalar, including packed or unpacked repeated values.
    Scalar {
        /// Scalar wire representation.
        wire: WireType,
        /// Whether a vector of scalars is allocated.
        repeated: bool,
        /// Inline size of one scalar.
        size: usize,
    },
}

struct Frame {
    end: usize,
    schema: fn(u32) -> Option<Field>,
    group: Option<u32>,
    packed: Option<(WireType, usize)>,
}

/// Incremental resource preflight of a borrowed protobuf source.
///
/// Scratch storage grows only with depth. Allocation estimates count optional
/// messages conservatively and allow four element sizes per repeated entry to
/// cover vector growth and small-vector minimum capacities. Byte buffers allow
/// twice their length plus minimum capacity. Overwritten fields also count:
/// they still allocate and consume decoding work before Prost discards them.
pub(super) struct DecodeAdmission<'a> {
    bytes: &'a [u8],
    position: usize,
    stack: Vec<Frame>,
    allocation: usize,
    work: usize,
    max_allocation: usize,
    max_work: usize,
}

impl<'a> DecodeAdmission<'a> {
    /// Starts admission before any source message is materialized.
    pub(super) fn new<T: DecodeSchema>(
        bytes: &'a [u8],
        max_allocation: usize,
        max_work: usize,
    ) -> Result<Self, AdmissionError> {
        let mut admission = Self {
            bytes,
            position: 0,
            stack: vec![Frame {
                end: bytes.len(),
                schema: T::field,
                group: None,
                packed: None,
            }],
            allocation: 0,
            work: 0,
            max_allocation,
            max_work,
        };
        admission.charge_allocation(size_of::<T>())?;
        Ok(admission)
    }

    fn charge_allocation(&mut self, amount: usize) -> Result<(), AdmissionError> {
        self.allocation = self.allocation.saturating_add(amount);
        if self.allocation > self.max_allocation {
            return Err(AdmissionError::Allocation(self.max_allocation));
        }
        Ok(())
    }

    fn advance(&mut self, amount: usize, end: usize) -> Result<(), AdmissionError> {
        self.position = self
            .position
            .checked_add(amount)
            .filter(|next| *next <= end)
            .ok_or(AdmissionError::Invalid("truncated protobuf field"))?;
        Ok(())
    }

    fn varint(&mut self, end: usize) -> Result<u64, AdmissionError> {
        let mut remaining = &self.bytes[self.position..end];
        let value = decode_varint(&mut remaining)?;
        self.position = end - remaining.len();
        Ok(value)
    }

    fn scalar(&mut self, wire: WireType, end: usize) -> Result<(), AdmissionError> {
        match wire {
            WireType::Varint => {
                let _ = self.varint(end)?;
            }
            WireType::ThirtyTwoBit => self.advance(4, end)?,
            WireType::SixtyFourBit => self.advance(8, end)?,
            _ => return Err(AdmissionError::Invalid("invalid scalar wire type")),
        }
        Ok(())
    }

    fn push(&mut self, frame: Frame) -> Result<(), AdmissionError> {
        if self.stack.len() >= MAX_ADMISSION_FRAMES {
            return Err(AdmissionError::Invalid(
                "OTLP admission recursion limit exceeded",
            ));
        }
        self.stack.push(frame);
        Ok(())
    }

    /// Scans up to the field/element budget or wire-byte target per call.
    /// A length-delimited leaf is skipped in constant time and may cross the byte target.
    /// `true` means resource admission succeeded, not that Prost validation can
    /// be skipped. `false` asks the caller to yield before the next step.
    pub(super) fn step(
        &mut self,
        max_fields: usize,
        max_bytes: usize,
    ) -> Result<bool, AdmissionError> {
        if max_fields == 0 || max_bytes == 0 {
            return Err(AdmissionError::Invalid(
                "OTLP admission step budgets must be positive",
            ));
        }
        let start = self.position;
        let mut steps = 0;
        while let Some(frame) = self.stack.last() {
            let end = frame.end;
            if self.position == end {
                if frame.group.is_some() {
                    return Err(AdmissionError::Invalid("unterminated protobuf group"));
                }

                let _ = self.stack.pop();
                continue;
            }
            if steps >= max_fields || self.position - start >= max_bytes {
                return Ok(false);
            }
            steps += 1;
            self.work = self.work.saturating_add(1);
            if self.work > self.max_work {
                return Err(AdmissionError::Work(self.max_work));
            }
            if let Some((wire, size)) = frame.packed {
                self.charge_allocation(size.saturating_mul(4))?;
                self.scalar(wire, end)?;
                continue;
            }
            let schema = frame.schema;
            let group = frame.group;
            let mut remaining = &self.bytes[self.position..end];
            let (tag, wire) = decode_key(&mut remaining)?;
            self.position = end - remaining.len();
            if wire == WireType::EndGroup {
                if group != Some(tag) {
                    return Err(AdmissionError::Invalid("unexpected protobuf end group"));
                }
                let _ = self.stack.pop();
                continue;
            }
            let field = schema(tag);
            let expected = match field {
                Some(Field::Message { .. } | Field::Bytes { .. }) => {
                    Some(WireType::LengthDelimited)
                }
                Some(Field::Scalar {
                    wire: expected,
                    repeated,
                    ..
                }) if !(repeated && wire == WireType::LengthDelimited) => Some(expected),
                _ => None,
            };
            if let Some(expected) = expected {
                check_wire_type(expected, wire)?;
            }
            match wire {
                WireType::LengthDelimited => {
                    let length = usize::try_from(self.varint(end)?)
                        .map_err(|_| AdmissionError::Invalid("protobuf length overflow"))?;
                    let child_end = self
                        .position
                        .checked_add(length)
                        .filter(|next| *next <= end)
                        .ok_or(AdmissionError::Invalid("truncated protobuf message"))?;
                    match field {
                        Some(Field::Message { schema, size }) => {
                            self.charge_allocation(size.saturating_mul(4))?;
                            self.push(Frame {
                                end: child_end,
                                schema,
                                group: None,
                                packed: None,
                            })?;
                        }
                        Some(Field::Bytes { repeated, size }) => {
                            self.charge_allocation(
                                length.saturating_mul(2).saturating_add(8).saturating_add(
                                    if repeated { size.saturating_mul(4) } else { 0 },
                                ),
                            )?;
                            self.position = child_end;
                        }
                        Some(Field::Scalar { wire, size, .. }) => {
                            self.push(Frame {
                                end: child_end,
                                schema,
                                group: None,
                                packed: Some((wire, size)),
                            })?;
                        }
                        None => self.position = child_end,
                    }
                }
                WireType::StartGroup => {
                    self.push(Frame {
                        end,
                        schema: |_| None,
                        group: Some(tag),
                        packed: None,
                    })?;
                }
                _ => {
                    if let Some(Field::Scalar {
                        repeated: true,
                        size,
                        ..
                    }) = field
                    {
                        self.charge_allocation(size.saturating_mul(4))?;
                    }
                    self.scalar(wire, end)?;
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        HistogramDataPoint, MetricsData,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
    use prost::Message;

    /// Scenario: every ADX layout is compared with Prost encodings of each field and oneof case.
    /// Guarantees: field numbers and wire representations match Prost, and every layout is exercised.
    #[test]
    fn local_layouts_match_prost_fields() {
        schema::check_all_fields();
    }

    fn nested(path: &[u32], contents: &[u8]) -> Vec<u8> {
        let mut bytes = contents.to_vec();
        for &tag in path.iter().rev() {
            let mut parent = Vec::new();
            prost::encoding::encode_key(tag, WireType::LengthDelimited, &mut parent);
            prost::encoding::encode_varint(bytes.len() as u64, &mut parent);
            parent.extend(bytes);
            bytes = parent;
        }
        bytes
    }

    fn scan<T: DecodeSchema>(
        bytes: &[u8],
        allocation: usize,
        work: usize,
    ) -> Result<(), AdmissionError> {
        let mut admission = DecodeAdmission::new::<T>(bytes, allocation, work)?;
        while !admission.step(7, 128)? {}
        Ok(())
    }

    /// Scenario: tiny repeated messages amplify storage in each telemetry signal and metric kind.
    /// Guarantees: nested layouts reject the source before any decoded message is built.
    #[test]
    fn repeated_message_storage_is_checked_for_every_signal_and_metric_kind() {
        let entries = [0x12, 0].repeat(100_000);
        assert!(matches!(
            scan::<LogsData>(&nested(&[1, 2], &entries), 4096, usize::MAX),
            Err(AdmissionError::Allocation(4096))
        ));
        assert!(matches!(
            scan::<TracesData>(&nested(&[1, 2], &entries), 4096, usize::MAX),
            Err(AdmissionError::Allocation(4096))
        ));
        let points = [0x0a, 0].repeat(100_000);
        for kind in [5, 7, 9, 10, 11] {
            assert!(
                matches!(
                    scan::<MetricsData>(&nested(&[1, 2, 2, kind], &points), 4096, usize::MAX),
                    Err(AdmissionError::Allocation(4096))
                ),
                "metric tag {kind}"
            );
        }
    }

    /// Scenario: repeated attributes, nested arrays, strings, and scalar buckets consume storage.
    /// Guarantees: admission follows common-message and packed-scalar layouts, not only record counts.
    #[test]
    fn composite_and_scalar_allocations_are_accounted() {
        for path in [&[1, 1][..], &[1, 2, 2, 6][..], &[1, 2, 2, 5, 5, 1][..]] {
            let bytes = nested(path, &[0x0a, 0].repeat(4096));
            assert!(
                matches!(
                    scan::<LogsData>(&bytes, 4096, usize::MAX),
                    Err(AdmissionError::Allocation(4096))
                ),
                "{path:?}"
            );
        }
        let large_string = nested(&[1, 2, 2, 5, 1], &vec![b'x'; 4096]);
        assert!(matches!(
            scan::<LogsData>(&large_string, 4096, usize::MAX),
            Err(AdmissionError::Allocation(4096))
        ));
        for (path, scalar) in [
            (&[1, 2, 2, 9, 1, 6][..], vec![0; 8192]),
            (&[1, 2, 2, 10, 1, 8, 2][..], vec![0; 1024]),
        ] {
            assert!(
                matches!(
                    scan::<MetricsData>(&nested(path, &scalar), 4096, usize::MAX),
                    Err(AdmissionError::Allocation(4096))
                ),
                "{path:?}"
            );
        }
    }

    /// Scenario: decoding reaches its exact work and allocation budgets.
    /// Guarantees: equality is accepted, the next unit is refused, and packed elements count as work.
    #[test]
    fn exact_admission_limits_are_enforced() {
        let point = HistogramDataPoint {
            bucket_counts: vec![1, 2, 3],
            ..Default::default()
        };
        let bytes = point.encode_to_vec();
        let mut admission =
            DecodeAdmission::new::<HistogramDataPoint>(&bytes, usize::MAX, usize::MAX)
                .expect("admission");
        assert!(admission.step(4, usize::MAX).expect("four operations"));
        assert_eq!(admission.work, 4);
        let allocation = admission.allocation;
        scan::<HistogramDataPoint>(&bytes, allocation, 4).expect("exact budgets");
        assert!(matches!(
            scan::<HistogramDataPoint>(&bytes, allocation - 1, 4),
            Err(AdmissionError::Allocation(_))
        ));
        assert!(matches!(
            scan::<HistogramDataPoint>(&bytes, allocation, 3),
            Err(AdmissionError::Work(3))
        ));
    }

    /// Scenario: a packed repeated field contains more elements than one cooperative step allows.
    /// Guarantees: stepping stops at the element budget rather than scanning the entire packed field.
    #[test]
    fn packed_fields_resume_cooperatively() {
        let point = HistogramDataPoint {
            bucket_counts: vec![0; 1024],
            ..Default::default()
        };
        let bytes = point.encode_to_vec();
        let mut admission = DecodeAdmission::new::<HistogramDataPoint>(&bytes, usize::MAX, 2048)
            .expect("admission");
        assert!(!admission.step(8, usize::MAX).expect("first step"));
        assert_eq!(admission.work, 8);
        assert_eq!(admission.stack.len(), 2);
        while !admission.step(8, 64).expect("next step") {}
        assert_eq!(admission.work, 1025);
    }

    /// Scenario: unknown fields include opaque non-protobuf bytes and correctly matched nested groups.
    /// Guarantees: admission preserves Prost's unknown-field semantics without parsing opaque contents.
    #[test]
    fn unknown_fields_and_groups_remain_compatible_with_prost() {
        let mut bytes = b"\xa3\x06\xab\x06\xac\x06\xa4\x06".to_vec();
        bytes.extend(nested(&[101], b"\xff\x80"));
        scan::<LogsData>(&bytes, 1024, 8).expect("admitted");
        assert_eq!(
            LogsData::decode(bytes.as_slice()).expect("valid protobuf"),
            LogsData::default()
        );
        for invalid in [b"\xa3\x06".as_slice(), b"\xa3\x06\xac\x06", b"\xa4\x06"] {
            assert!(scan::<LogsData>(invalid, 1024, 8).is_err());
            assert!(LogsData::decode(invalid).is_err());
        }
        let mut deepest_groups = b"\xa3\x06".repeat(100);
        deepest_groups.extend(b"\xa4\x06".repeat(100));
        scan::<LogsData>(&deepest_groups, 1024, 256)
            .expect("Prost's full group recursion allowance");
        assert_eq!(
            LogsData::decode(deepest_groups.as_slice()).expect("valid deepest groups"),
            LogsData::default(),
        );
    }

    /// Scenario: many unknown scalar fields consume CPU without allocating decoded objects.
    /// Guarantees: the work limit independently refuses them with no dependence on allocation growth.
    #[test]
    fn unknown_field_work_is_limited() {
        let bytes = [0xa0, 0x06, 0].repeat(100_000);
        assert!(matches!(
            scan::<LogsData>(&bytes, 1024, 256),
            Err(AdmissionError::Work(256))
        ));
    }
}
