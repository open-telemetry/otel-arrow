// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parquet study: compare the read/write cost and serialized size of OTAP logs
//! encoded as compressed Arrow IPC versus several flattened-Parquet layouts.
//!
//! Every contender implements [`Codec`], which turns an [`OtapArrowRecords`]
//! logs batch into "wire" bytes ([`Codec::write`]) and back ([`Codec::read`]).
//! Contenders are enumerated by [`Scheme`] and parameterized by a [`Compressor`].
//! Compressors are explicit codecs (`zstd`, `lz4`, `snappy`, `none`) so that
//! zstd can be compared head-to-head with lz4 -- important when a consumer's
//! Arrow/Parquet stack (for example some .NET implementations) may not support
//! zstd. Arrow IPC only supports zstd and lz4 (frame), so snappy is offered for
//! the Parquet schemes only.

use otap_df_pdata::otap::OtapArrowRecords;

pub mod attrs;
pub mod datagen;
pub mod ipc;
pub mod map;
pub mod nested;
pub mod parquet_io;
pub mod server;
#[cfg(feature = "vortex")]
pub mod vortex;
pub mod wide;

/// Error type used throughout the study (benchmark/test code, so a boxed error
/// is sufficient).
pub type StudyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Compression codec applied by a [`Codec`].
///
/// | variant  | Arrow IPC     | Parquet        |
/// |----------|---------------|----------------|
/// | `Zstd`   | `ZSTD`        | `ZSTD`         |
/// | `Lz4`    | `LZ4_FRAME`   | `LZ4_RAW`      |
/// | `Snappy` | *unsupported* | `SNAPPY`       |
/// | `None`   | uncompressed  | uncompressed   |
///
/// `LZ4_RAW` (not the deprecated Hadoop-framed `LZ4`) is used for Parquet
/// because it is the cross-language interoperable variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compressor {
    /// zstd.
    Zstd,
    /// lz4 (frame for IPC, raw for Parquet).
    Lz4,
    /// snappy (Parquet only).
    Snappy,
    /// No compression.
    None,
}

impl Compressor {
    /// All compressors, in reporting order (valid for the Parquet schemes).
    pub const ALL: [Compressor; 4] = [
        Compressor::Zstd,
        Compressor::Lz4,
        Compressor::Snappy,
        Compressor::None,
    ];

    /// Compressors Arrow IPC supports (snappy is not an Arrow IPC codec).
    pub const IPC: [Compressor; 3] = [Compressor::Zstd, Compressor::Lz4, Compressor::None];

    /// Short label used in benchmark ids and size tables.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Compressor::Zstd => "zstd",
            Compressor::Lz4 => "lz4",
            Compressor::Snappy => "snappy",
            Compressor::None => "none",
        }
    }

    /// IPC compression for this setting. Panics for [`Compressor::Snappy`], which
    /// Arrow IPC does not support; callers use [`Scheme::compressors`] to avoid
    /// that combination.
    #[must_use]
    pub fn ipc(self) -> Option<arrow_ipc::CompressionType> {
        match self {
            Compressor::Zstd => Some(arrow_ipc::CompressionType::ZSTD),
            Compressor::Lz4 => Some(arrow_ipc::CompressionType::LZ4_FRAME),
            Compressor::None => None,
            Compressor::Snappy => unreachable!("Arrow IPC does not support snappy"),
        }
    }

    /// Parquet compression for this setting.
    #[must_use]
    pub fn parquet(self) -> parquet::basic::Compression {
        use parquet::basic::{Compression, ZstdLevel};
        match self {
            Compressor::Zstd => Compression::ZSTD(ZstdLevel::try_new(3).expect("valid zstd level")),
            Compressor::Lz4 => Compression::LZ4_RAW,
            Compressor::Snappy => Compression::SNAPPY,
            Compressor::None => Compression::UNCOMPRESSED,
        }
    }
}

/// A round-trippable encoding of an OTAP logs batch.
pub trait Codec {
    /// Stable name of this contender (e.g. `"ipc"`, `"parquet-nested"`).
    fn name(&self) -> &'static str;

    /// Encode an OTAP logs batch into serialized "wire" bytes.
    ///
    /// Takes the batch by value: the IPC producer mutates it in place
    /// (transport-optimized encoding), and the Criterion harness clones the
    /// input in its (untimed) setup closure.
    fn write(&self, logs: OtapArrowRecords) -> StudyResult<Vec<u8>>;

    /// Decode serialized bytes back into an OTAP logs batch.
    fn read(&self, bytes: &[u8]) -> StudyResult<OtapArrowRecords>;
}

/// The contenders compared by the study.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    /// OTAP interleaved Arrow IPC streams (the representation we have today).
    Ipc,
    /// Flattened Parquet, attributes as `List<Struct>`.
    Nested,
    /// Flattened Parquet, attributes as `Map`.
    Map,
    /// Flattened Parquet, attributes exploded to one typed column per key.
    Wide,
    /// Flattened Vortex file (nested layout). Requires the `vortex` feature.
    #[cfg(feature = "vortex")]
    Vortex,
}

impl Scheme {
    /// All schemes, in reporting order (includes Vortex when the `vortex`
    /// feature is enabled).
    #[must_use]
    pub fn all() -> Vec<Scheme> {
        #[allow(unused_mut)]
        let mut v = vec![Scheme::Ipc, Scheme::Nested, Scheme::Map, Scheme::Wide];
        #[cfg(feature = "vortex")]
        v.push(Scheme::Vortex);
        v
    }

    /// Only the flattened-file schemes (used by the server-cost model, where IPC
    /// is the input rather than an output format). Includes Vortex when enabled.
    #[must_use]
    pub fn flattened() -> Vec<Scheme> {
        #[allow(unused_mut)]
        let mut v = vec![Scheme::Nested, Scheme::Map, Scheme::Wide];
        #[cfg(feature = "vortex")]
        v.push(Scheme::Vortex);
        v
    }

    /// Stable contender name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Ipc => "ipc",
            Scheme::Nested => "parquet-nested",
            Scheme::Map => "parquet-map",
            Scheme::Wide => "parquet-wide",
            #[cfg(feature = "vortex")]
            Scheme::Vortex => "vortex",
        }
    }

    /// The compressors valid for this scheme. IPC excludes snappy; Vortex applies
    /// its own cascading compression, so it exposes only a single `none` setting.
    #[must_use]
    pub fn compressors(self) -> &'static [Compressor] {
        match self {
            Scheme::Ipc => &Compressor::IPC,
            #[cfg(feature = "vortex")]
            Scheme::Vortex => &[Compressor::None],
            _ => &Compressor::ALL,
        }
    }

    /// Construct the [`Codec`] for this scheme with the given compressor.
    #[must_use]
    pub fn codec(self, compressor: Compressor) -> Box<dyn Codec> {
        match self {
            Scheme::Ipc => Box::new(ipc::IpcCodec { compressor }),
            Scheme::Nested => Box::new(nested::NestedParquetCodec { compressor }),
            Scheme::Map => Box::new(map::MapParquetCodec { compressor }),
            Scheme::Wide => Box::new(wide::WideParquetCodec { compressor }),
            #[cfg(feature = "vortex")]
            Scheme::Vortex => Box::new(vortex::VortexCodec),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};

    /// Round-trip every scheme x its valid compressors and assert the result is
    /// logically equivalent to the input.
    #[test]
    fn all_contenders_round_trip() {
        let params = LogsGenParams {
            num_resources: 2,
            num_scopes: 3,
            num_logs: 4,
        };
        let (otap, _) = gen_logs_otap(&params);

        for scheme in Scheme::all() {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let bytes = codec
                    .write(otap.clone())
                    .unwrap_or_else(|e| panic!("{} write failed: {e}", codec.name()));
                assert!(!bytes.is_empty(), "{} produced no bytes", codec.name());

                let decoded = codec
                    .read(&bytes)
                    .unwrap_or_else(|e| panic!("{} read failed: {e}", codec.name()));

                attrs::assert_logs_equivalent(&otap, &decoded, codec.name(), compressor.label());
            }
        }
    }
}
