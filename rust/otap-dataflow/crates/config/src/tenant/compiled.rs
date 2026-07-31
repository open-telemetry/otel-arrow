// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compiled, hot-path representation of tenant tokens and conditions.
//!
//! The design follows a database hash join. There are three phases:
//!
//! 1. **Build** (once per pipeline group). Config strings are interned to
//!    numeric ids, an extractor index is built over transport header names,
//!    declared conditions are grouped by *signature* (their set of keys), and
//!    the cross product of applicable (token, signature) pairs is assigned
//!    dense slots. The literal values of declared conditions are hashed here:
//!    that is the build side of the join.
//! 2. **Context creation** (per request, in a receiver). One pass over the
//!    request's headers resolves tokens, then one fingerprint per allocated
//!    pair slot is computed by projecting the token onto the signature's keys.
//!    That is the probe side of the join. The result is packed into a single
//!    allocation.
//! 3. **Consumer probe** (per request, in a routing or limiting node). One
//!    hash table lookup per signature using the precomputed fingerprint, with
//!    no string work and no allocation.
//!
//! A token resolves all-or-nothing: every one of its extractors must succeed.
//! That is what makes wildcard entries free at runtime -- if a signature is
//! applicable to a token and the token resolved, every key the condition
//! requires is necessarily present.
//!
//! The packed per-request value replaces the previous transport header map, so
//! this machinery is paid for by deleting the old representation rather than
//! added on top of it. See `docs/multitenancy-tenant.md`.

use crate::error::Error;
use crate::tenant::{Condition, Extractor, TenantBoundaryPolicy, TenantTokenSpec, TenantTokens};
use ahash::AHashMap;
use smallvec::SmallVec;
use std::net::SocketAddr;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

/// Interned token key identifier.
pub type KeyId = u16;
/// Index of a tenant token definition within the registry.
pub type TokenIdx = u16;
/// Index of a condition signature within the registry.
pub type SignatureId = u16;
/// Dense slot identifying one applicable (token, signature) pair.
pub type PairSlot = u16;
/// Index of a declared condition within one consumer's condition set.
pub type ConditionIdx = u16;

/// Maximum number of extractor keys in a single token, bounded by the width of
/// the per-token unsatisfied-key bitmask.
pub const MAX_TOKEN_KEYS: usize = 64;
/// Maximum number of tokens, bounded by the width of the resolved bitmask.
pub const MAX_TOKENS: usize = 64;
/// Maximum total size of the retained value bytes in one request context.
pub const MAX_VALUE_BYTES: usize = u16::MAX as usize;

fn config_error(error: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: error.into(),
    }
}

/// Serialize one `key: value` term into the fingerprint buffer. Build side and
/// probe side both go through here so the two layouts cannot drift.
fn fingerprint_term(buf: &mut Vec<u8>, key: KeyId, value: &[u8]) {
    buf.extend_from_slice(&key.to_le_bytes());
    let len = u32::try_from(value.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(value);
}

// -- Build-side structures ---------------------------------------------------

/// One extractor slot reached through the header index.
#[derive(Debug, Clone, Copy)]
struct Slot {
    token: TokenIdx,
    bit: u8,
    key: KeyId,
}

/// Entry of the header index: the extractors satisfied by one header name.
#[derive(Debug, Default)]
struct HeaderSlot {
    any_value: SmallVec<[Slot; 2]>,
}

/// An extractor that does not read a transport header.
#[derive(Debug)]
struct StaticExtractor {
    slot: Slot,
    /// Constant value for a `generic_key` extractor. `None` selects the
    /// network peer address, rendered as text.
    value: Option<Box<[u8]>>,
}

/// A compiled tenant token definition.
#[derive(Debug)]
struct CompiledToken {
    name: Box<str>,
    /// Bit per extractor key; a token resolves when all bits are cleared.
    key_mask: u64,
    /// Keys of this token, sorted, used for applicability tests.
    keys: Box<[KeyId]>,
}

/// Layout shared by every condition that tests the same set of keys.
///
/// The key order here defines the fingerprint layout, so the build side and
/// the probe side agree byte for byte.
#[derive(Debug)]
struct Signature {
    /// Keys carrying a literal value, sorted. These form the fingerprint.
    fixed_keys: Box<[KeyId]>,
    /// Keys required to be present with any value, sorted.
    wildcard_keys: Box<[KeyId]>,
    /// Union of the two, sorted, used for applicability tests.
    required_keys: Box<[KeyId]>,
}

/// Compiled tenant token registry, immutable after build and shared by every
/// node in a pipeline group.
#[derive(Debug, Default)]
pub struct TenantTokenRegistry {
    /// Deployment generation this registry was built for. Every packed context
    /// carries it, so a context built against a superseded registry -- whose
    /// value slots and pair slots may mean something else entirely -- can be
    /// recognized rather than misread.
    epoch: u16,
    /// Key id to key name, for diagnostics and condition lookup.
    key_names: Vec<Box<str>>,
    tokens: Vec<CompiledToken>,
    /// Lowercased transport header name to the extractors it satisfies.
    header_index: AHashMap<Box<str>, HeaderSlot>,
    static_extractors: Vec<StaticExtractor>,
    signatures: Vec<Signature>,
    /// Dense (token, signature) pairs; the index is the [`PairSlot`].
    pairs: Vec<(TokenIdx, SignatureId)>,
    /// Reverse lookup used at node construction, never on the hot path.
    pair_index: AHashMap<(TokenIdx, SignatureId), PairSlot>,
    /// Extractors resolved from an inbound cross-boundary context, indexed by
    /// the key id they read. Both sides of a boundary share this registry, so
    /// an import is a slot lookup rather than a name match.
    import_by_key: Vec<SmallVec<[Slot; 2]>>,
    /// Keys whose values are carried in the request context. The index into
    /// this vector is the value slot each such key occupies in every packed
    /// context built by this registry.
    retained: Vec<KeyId>,
    /// Per value slot: whether the key name travels with the value.
    bagged: Vec<bool>,
    /// Reverse of `retained`, indexed by key id; [`NO_VALUE_SLOT`] for keys
    /// that are match-only.
    value_slot: Vec<u16>,
    /// Indexed by value slot: the tokens that declared this key with
    /// `retain: true`. A slot is populated only when one of them resolves, so
    /// a value never travels without the evidence that justified carrying it.
    retain_mask: Vec<u64>,
}

/// How a key acquires its value.
///
/// Staging is per key, so within one resolve path a key must have exactly one
/// binding: two tokens binding it to different sources would race, and both
/// the surviving value and every fingerprint computed from it would depend on
/// request ordering.
///
/// There are two disjoint resolve paths. `resolve` runs transport-header and
/// static extractors; `resolve_imported` runs imported-key and static
/// extractors. A key bound to a header on the ingress side and imported on
/// the far side of a boundary is therefore consistent, and is exactly how a
/// portable key crosses a boundary keeping its name.
#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyBinding {
    /// Lowercased transport header name.
    Header(Box<str>),
    /// Literal value from a `generic_key` extractor.
    Static(Box<[u8]>),
    RemoteAddress,
    /// Upstream key id read across a boundary.
    Imported(KeyId),
}

impl KeyBinding {
    fn describe(&self) -> String {
        match self {
            Self::Header(name) => format!("transport_header '{name}'"),
            Self::Static(value) => {
                format!("generic_key '{}'", String::from_utf8_lossy(value))
            }
            Self::RemoteAddress => "remote_address".to_owned(),
            Self::Imported(key) => format!("imported_key #{key}"),
        }
    }
}

/// Builder collecting token definitions and declared conditions before the
/// registry is frozen.
#[derive(Debug, Default)]
pub struct TenantTokenRegistryBuilder {
    key_ids: AHashMap<Box<str>, KeyId>,
    key_names: Vec<Box<str>>,
    tokens: Vec<CompiledToken>,
    token_ids: AHashMap<Box<str>, TokenIdx>,
    header_index: AHashMap<Box<str>, HeaderSlot>,
    static_extractors: Vec<StaticExtractor>,
    signatures: Vec<Signature>,
    signature_ids: AHashMap<(Vec<KeyId>, Vec<KeyId>), SignatureId>,
    pairs: Vec<(TokenIdx, SignatureId)>,
    pair_index: AHashMap<(TokenIdx, SignatureId), PairSlot>,
    import_by_key: AHashMap<KeyId, SmallVec<[Slot; 2]>>,
    /// The single binding each key may have on the request path, indexed by
    /// key id. Transport-header and static extractors register here.
    request_binding: Vec<Option<KeyBinding>>,
    /// The single binding each key may have on the import path, indexed by
    /// key id. Imported-key and static extractors register here.
    import_binding: Vec<Option<KeyBinding>>,
    /// Tokens declaring each key with `retain: true`, indexed by key id.
    key_retain_mask: Vec<u64>,
    retained: Vec<KeyId>,
    bagged: Vec<bool>,
}

impl TenantTokenRegistryBuilder {
    /// Create an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn intern_key(&mut self, key: &str) -> KeyId {
        if let Some(id) = self.key_ids.get(key) {
            return *id;
        }
        let id = KeyId::try_from(self.key_names.len()).expect("too many tenant token keys");
        let boxed: Box<str> = key.into();
        self.key_names.push(boxed.clone());
        self.request_binding.push(None);
        self.import_binding.push(None);
        self.key_retain_mask.push(0);
        let _ = self.key_ids.insert(boxed, id);
        id
    }

    /// Record a key's binding on the request path, the import path, or both,
    /// rejecting a second, different binding on the same path.
    fn bind_key(
        &mut self,
        name: &str,
        key: KeyId,
        binding: &KeyBinding,
        request: bool,
        import: bool,
    ) -> Result<(), Error> {
        if request {
            Self::bind_on(
                &mut self.request_binding,
                &self.key_names,
                name,
                key,
                binding,
                "request",
            )?;
        }
        if import {
            Self::bind_on(
                &mut self.import_binding,
                &self.key_names,
                name,
                key,
                binding,
                "import",
            )?;
        }
        Ok(())
    }

    fn bind_on(
        table: &mut [Option<KeyBinding>],
        key_names: &[Box<str>],
        name: &str,
        key: KeyId,
        binding: &KeyBinding,
        path: &str,
    ) -> Result<(), Error> {
        match &table[usize::from(key)] {
            Some(existing) if existing != binding => Err(config_error(format!(
                "tenant token '{}' binds key '{}' to {} on the {path} path, but it is \
                 already bound there to {}; a key resolves from one source per path, \
                 so give these different key names",
                name,
                key_names[usize::from(key)],
                binding.describe(),
                existing.describe(),
            ))),
            Some(_) => Ok(()),
            None => {
                table[usize::from(key)] = Some(binding.clone());
                Ok(())
            }
        }
    }

    /// Add every token definition from the engine configuration.
    pub fn add_tokens(&mut self, tokens: &TenantTokens) -> Result<(), Error> {
        // Deterministic order so that ids are stable across runs.
        let mut names: Vec<&String> = tokens.keys().collect();
        names.sort();
        for name in names {
            self.add_token(name, &tokens[name])?;
        }
        Ok(())
    }

    fn add_token(&mut self, name: &str, spec: &TenantTokenSpec) -> Result<(), Error> {
        if spec.extractors.is_empty() {
            return Err(config_error(format!(
                "tenant token '{name}' declares no extractors"
            )));
        }
        if spec.extractors.len() > MAX_TOKEN_KEYS {
            return Err(config_error(format!(
                "tenant token '{name}' declares more than {MAX_TOKEN_KEYS} extractors"
            )));
        }
        if self.tokens.len() >= MAX_TOKENS {
            return Err(config_error(format!(
                "more than {MAX_TOKENS} tenant tokens declared"
            )));
        }
        let token = TokenIdx::try_from(self.tokens.len()).expect("token count checked above");

        let mut keys: Vec<KeyId> = Vec::with_capacity(spec.extractors.len());
        for (bit, extractor) in spec.extractors.iter().enumerate() {
            let key = self.intern_key(extractor.key());
            keys.push(key);

            if extractor.retain() {
                let _ = self.add_retained(key, extractor.bag());
                self.key_retain_mask[usize::from(key)] |= 1u64 << token;
            }

            let slot = Slot {
                token,
                bit: u8::try_from(bit).expect("extractor count checked above"),
                key,
            };

            match extractor {
                Extractor::TransportHeader {
                    transport_header, ..
                } => {
                    let lower: Box<str> = transport_header.to_ascii_lowercase().into_boxed_str();
                    self.bind_key(name, key, &KeyBinding::Header(lower.clone()), true, false)?;
                    self.header_index
                        .entry(lower)
                        .or_default()
                        .any_value
                        .push(slot);
                }
                Extractor::GenericKey { generic_key, .. } => {
                    let binding = KeyBinding::Static(generic_key.as_bytes().into());
                    self.bind_key(name, key, &binding, true, true)?;
                    self.static_extractors.push(StaticExtractor {
                        slot,
                        value: Some(generic_key.as_bytes().into()),
                    });
                }
                Extractor::RemoteAddress { .. } => {
                    self.bind_key(name, key, &KeyBinding::RemoteAddress, true, true)?;
                    self.static_extractors
                        .push(StaticExtractor { slot, value: None });
                }
                Extractor::ImportedKey { imported_key, .. } => {
                    let upstream = self.intern_key(imported_key);
                    self.bind_key(name, key, &KeyBinding::Imported(upstream), false, true)?;
                    self.import_by_key.entry(upstream).or_default().push(slot);
                }
            }
        }

        let key_mask = if spec.extractors.len() == MAX_TOKEN_KEYS {
            u64::MAX
        } else {
            (1u64 << spec.extractors.len()) - 1
        };
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != keys.len() {
            return Err(config_error(format!(
                "tenant token '{name}' declares duplicate keys"
            )));
        }

        let boxed: Box<str> = name.into();
        let _ = self.token_ids.insert(boxed.clone(), token);
        self.tokens.push(CompiledToken {
            name: boxed,
            key_mask,
            keys: sorted.into_boxed_slice(),
        });
        Ok(())
    }

    fn add_retained(&mut self, key: KeyId, bag: bool) -> u16 {
        if let Some(idx) = self.retained.iter().position(|k| *k == key) {
            self.bagged[idx] |= bag;
            return u16::try_from(idx).expect("retain slot fits");
        }
        let idx = u16::try_from(self.retained.len()).expect("retain slot fits");
        self.retained.push(key);
        self.bagged.push(bag);
        idx
    }

    /// Declare the conditions of one consumer node so that the corresponding
    /// signatures and pair slots exist in the frozen registry.
    ///
    /// `tokens` names the tenant tokens the consumer binds; `None` binds every
    /// declared token.
    pub fn declare_conditions(
        &mut self,
        tokens: Option<&[String]>,
        conditions: &[Condition],
    ) -> Result<(), Error> {
        let bound = self.resolve_bound_tokens(tokens)?;
        for condition in conditions {
            let signature = self.intern_signature(condition)?;
            for token in &bound {
                self.allocate_pair(*token, signature);
            }
        }
        Ok(())
    }

    fn resolve_bound_tokens(&self, tokens: Option<&[String]>) -> Result<Vec<TokenIdx>, Error> {
        match tokens {
            None => Ok((0..self.tokens.len())
                .map(|i| TokenIdx::try_from(i).expect("token index fits"))
                .collect()),
            Some(names) => names
                .iter()
                .map(|name| {
                    self.token_ids
                        .get(name.as_str())
                        .copied()
                        .ok_or_else(|| config_error(format!("unknown tenant token '{name}'")))
                })
                .collect(),
        }
    }

    fn intern_signature(&mut self, condition: &Condition) -> Result<SignatureId, Error> {
        if condition.entries.is_empty() {
            return Err(config_error("tenant condition declares no entries"));
        }
        let mut fixed: Vec<KeyId> = Vec::new();
        let mut wildcard: Vec<KeyId> = Vec::new();
        for entry in &condition.entries {
            let key = self.intern_key(&entry.key);
            if entry.value.is_some() {
                fixed.push(key);
            } else {
                wildcard.push(key);
            }
        }
        fixed.sort_unstable();
        wildcard.sort_unstable();

        if let Some(id) = self.signature_ids.get(&(fixed.clone(), wildcard.clone())) {
            return Ok(*id);
        }
        let id = SignatureId::try_from(self.signatures.len())
            .map_err(|_| config_error("too many distinct tenant condition signatures"))?;
        let mut required: Vec<KeyId> = fixed.iter().chain(wildcard.iter()).copied().collect();
        required.sort_unstable();
        required.dedup();
        self.signatures.push(Signature {
            fixed_keys: fixed.clone().into_boxed_slice(),
            wildcard_keys: wildcard.clone().into_boxed_slice(),
            required_keys: required.into_boxed_slice(),
        });
        let _ = self.signature_ids.insert((fixed, wildcard), id);
        Ok(id)
    }

    fn allocate_pair(&mut self, token: TokenIdx, signature: SignatureId) {
        if self.pair_index.contains_key(&(token, signature)) {
            return;
        }
        let compiled = &self.tokens[usize::from(token)];
        let required = &self.signatures[usize::from(signature)].required_keys;
        // A signature applies to a token only when the token's schema covers
        // every key the condition tests.
        if !required.iter().all(|k| compiled.keys.contains(k)) {
            return;
        }
        let slot = PairSlot::try_from(self.pairs.len()).expect("pair slot fits");
        self.pairs.push((token, signature));
        let _ = self.pair_index.insert((token, signature), slot);
    }

    /// Freeze the builder into an immutable registry.
    ///
    /// `generation` is the deployment generation. The stored epoch mixes it
    /// with a digest of the value-slot layout, because a packed context is
    /// only readable by a registry that agrees on what each slot means. Two
    /// registries built from the same token definitions agree and get the same
    /// epoch; two that do not, disagree loudly rather than misreading slots.
    #[must_use]
    pub fn build(self, generation: u16) -> TenantTokenRegistry {
        let n_keys = self.key_names.len();
        let mut value_slot = vec![NO_VALUE_SLOT; n_keys];
        for (idx, key) in self.retained.iter().enumerate() {
            value_slot[usize::from(*key)] = u16::try_from(idx).expect("value slot fits");
        }
        let mut layout = Vec::new();
        for key in &self.retained {
            layout.extend_from_slice(self.key_names[usize::from(*key)].as_bytes());
            layout.push(0);
        }
        let epoch = generation ^ (xxh3_64(&layout) as u16);
        let retain_mask: Vec<u64> = self
            .retained
            .iter()
            .map(|key| self.key_retain_mask[usize::from(*key)])
            .collect();
        let mut import_by_key = vec![SmallVec::new(); n_keys];
        for (key, slots) in self.import_by_key {
            import_by_key[usize::from(key)] = slots;
        }
        TenantTokenRegistry {
            epoch,
            key_names: self.key_names,
            tokens: self.tokens,
            header_index: self.header_index,
            static_extractors: self.static_extractors,
            signatures: self.signatures,
            pairs: self.pairs,
            pair_index: self.pair_index,
            import_by_key,
            retained: self.retained,
            bagged: self.bagged,
            value_slot,
            retain_mask,
        }
    }
}

// -- Packed per-request value ------------------------------------------------

/// Words 0 and 1 are the fixed header of the packed representation.
const HEADER_WORDS: usize = 2;

/// Offset marking a value slot that is empty for this request.
const EMPTY_OFFSET: u32 = u32::MAX;

/// Compiled boundary allowlist, matched against the inline key names carried
/// in a cross-boundary context.
#[derive(Debug, Default, Clone)]
pub struct BoundaryFilter {
    /// One flag per key id; keys outside the registry cannot be named at all.
    allow: Box<[bool]>,
    any: bool,
}

impl BoundaryFilter {
    /// True when the policy admits nothing, so the boundary can be skipped.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.any
    }

    /// Whether a key crosses the boundary.
    #[must_use]
    pub fn admits(&self, key: KeyId) -> bool {
        self.allow.get(usize::from(key)).copied().unwrap_or(false)
    }
}

/// Reusable, receiver-owned scratch space for token resolution.
///
/// Resolution touches only these buffers, which are cleared and reused across
/// requests, so the steady-state cost of building a request context is the
/// single allocation that carries the packed result.
#[derive(Debug, Default)]
pub struct TokenScratch {
    /// One word per token; a token resolves when its word reaches zero.
    unsatisfied: Vec<u64>,
    /// Per key id: byte range of the extracted value inside `arena`.
    values: Vec<ValueRef>,
    /// Staging bytes for carried values and legacy header names.
    arena: Vec<u8>,
    fingerprint_buf: Vec<u8>,
    fingerprints: Vec<u64>,
    /// One entry per registry value slot, in slot order.
    slots: Vec<StagedSlot>,
    /// OTLP bytes assembled at pack time.
    blob: Vec<u8>,
    /// Blob offset of each value slot, parallel to `slots`.
    offsets: Vec<u32>,
    out: Vec<u64>,
}

/// One value slot staged for packing.
///
/// `bagged` decides which region of the blob it lands in, and therefore
/// whether its name is encoded at all.
#[derive(Debug, Clone, Copy, Default)]
struct StagedSlot {
    off: u32,
    len: u32,
    kind: ValueKind,
    present: bool,
    bagged: bool,
}

/// Whether a carried value is text or opaque bytes.
///
/// This chooses the `AnyValue` field the value is encoded into, which is the
/// only place the distinction is observable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValueKind {
    /// UTF-8 text; encoded as `AnyValue.string_value`.
    #[default]
    Text,
    /// Opaque bytes; encoded as `AnyValue.bytes_value`.
    Binary,
}

/// A value staged in the scratch arena, addressed by key id.
#[derive(Debug, Clone, Copy, Default)]
struct ValueRef {
    off: u32,
    len: u32,
    kind: ValueKind,
    present: bool,
}

/// Marks a key that no token retains, so it has no value slot.
const NO_VALUE_SLOT: u16 = u16::MAX;

/// OTLP protobuf field numbers used by the packed blob. Duplicated here
/// because `otap-df-pdata` depends on this crate, so the dependency cannot run
/// the other way. They are fixed by the OTLP wire format.
mod otlp {
    /// `KeyValue.key`, wire type 2.
    pub const KEY_VALUE_KEY_TAG: u8 = (1 << 3) | 2;
    /// `KeyValue.value`, wire type 2.
    pub const KEY_VALUE_VALUE_TAG: u8 = (2 << 3) | 2;
    /// `AnyValue.string_value`, wire type 2.
    pub const ANY_VALUE_STRING_TAG: u8 = (1 << 3) | 2;
    /// `AnyValue.bytes_value`, wire type 2.
    pub const ANY_VALUE_BYTES_TAG: u8 = (7 << 3) | 2;
}

/// Append a base-128 varint.
fn put_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push((value as u8) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

/// Byte length of a value encoded as a varint.
const fn varint_len(mut value: u64) -> usize {
    let mut n = 1;
    while value >= 0x80 {
        value >>= 7;
        n += 1;
    }
    n
}

/// Read a varint, returning its value and the offset just past it.
fn get_varint(buf: &[u8], mut at: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *buf.get(at)?;
        at += 1;
        value |= u64::from(byte & 0x7F) << shift;
        if byte < 0x80 {
            return Some((value, at));
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
}

/// Encode `value` as a length-delimited `AnyValue`, returning the offset of
/// its length prefix.
///
/// A slot points at this offset, so the encoding carries its own size and
/// `AnyValue` carries its own type. This is what replaces a separate length
/// and kind in the slot word.
fn put_any_value(blob: &mut Vec<u8>, value: &[u8], kind: ValueKind) -> u32 {
    let at = blob.len();
    let tag = match kind {
        ValueKind::Text => otlp::ANY_VALUE_STRING_TAG,
        ValueKind::Binary => otlp::ANY_VALUE_BYTES_TAG,
    };
    let inner = 1 + varint_len(value.len() as u64) + value.len();
    put_varint(blob, inner as u64);
    blob.push(tag);
    put_varint(blob, value.len() as u64);
    blob.extend_from_slice(value);
    u32::try_from(at).unwrap_or(EMPTY_OFFSET)
}

/// Decode the `AnyValue` whose length prefix starts at `at`, returning the
/// raw payload bytes.
fn any_value_bytes(blob: &[u8], at: usize) -> Option<&[u8]> {
    let (len, body) = get_varint(blob, at)?;
    let end = body.checked_add(usize::try_from(len).ok()?)?;
    let inner = blob.get(body..end)?;
    // One field: a tag byte then a length-delimited payload.
    let (payload_len, payload) = get_varint(inner, 1)?;
    inner.get(payload..payload.checked_add(usize::try_from(payload_len).ok()?)?)
}

/// Write the staged values of `scratch` into a single packed allocation.
///
/// Values are OTLP: each is a length-delimited `AnyValue`, and a key whose
/// name is demanded is written as a full `KeyValue` in the bag region so the
/// run can be appended to telemetry without re-encoding. Nothing is
/// self-describing otherwise -- values are addressed by the registry value
/// slot of their key, so a context is meaningful only to a registry that
/// agrees on the layout, which `epoch` states.
///
/// Layout, all words little-endian:
///
/// ```text
/// word 0     : n_fp:16 | n_slots:16 | epoch:16 | bag_len:16
/// word 1     : resolved token bitmask
/// words 2..  : n_fp fingerprints, indexed by PairSlot
/// then       : ceil(n_slots/2) words holding n_slots u32 offsets, indexed by
///              registry value slot; each points at the length prefix of a
///              `KeyValue.value`, or is EMPTY_OFFSET
/// then       : the byte blob, zero padded to a word boundary
/// ```
///
/// The blob opens with the bag: `bag_len` bytes holding a run of
/// `<len> <KeyValue>` entries carrying no field tag, so a consumer chooses the
/// destination field. Value-only entries follow as bare `<len> <AnyValue>`.
fn pack_words<'n>(
    scratch: &mut TokenScratch,
    resolved: u64,
    epoch: u16,
    slot_name: impl Fn(usize) -> &'n str,
) -> Arc<[u64]> {
    let TokenScratch {
        arena,
        slots,
        blob,
        offsets,
        fingerprints,
        out,
        ..
    } = scratch;

    blob.clear();
    offsets.clear();
    offsets.resize(slots.len(), EMPTY_OFFSET);

    // Bag keys first and contiguous, so the run can be copied in one pass.
    for (slot, staged) in slots.iter().enumerate() {
        if !staged.present || !staged.bagged {
            continue;
        }
        let name = slot_name(slot);
        let value = &arena[staged.off as usize..(staged.off + staged.len) as usize];
        let inner = 1
            + varint_len(name.len() as u64)
            + name.len()
            + 1
            + varint_len(any_value_len(value.len()) as u64)
            + any_value_len(value.len());
        put_varint(blob, inner as u64);
        blob.push(otlp::KEY_VALUE_KEY_TAG);
        put_varint(blob, name.len() as u64);
        blob.extend_from_slice(name.as_bytes());
        blob.push(otlp::KEY_VALUE_VALUE_TAG);
        offsets[slot] = put_any_value(blob, value, staged.kind);
    }
    let bag_len = blob.len();

    for (slot, staged) in slots.iter().enumerate() {
        if !staged.present || staged.bagged {
            continue;
        }
        let value = &arena[staged.off as usize..(staged.off + staged.len) as usize];
        offsets[slot] = put_any_value(blob, value, staged.kind);
    }

    debug_assert!(
        blob.len() <= MAX_VALUE_BYTES,
        "request context blob overflow"
    );

    let n_fp = fingerprints.len();
    let n_slots = slots.len();
    let slot_words = n_slots.div_ceil(2);
    let total = HEADER_WORDS + n_fp + slot_words + blob.len().div_ceil(8);

    out.clear();
    out.reserve(total);
    out.push(
        (n_fp as u64)
            | ((n_slots as u64) << 16)
            | (u64::from(epoch) << 32)
            | ((bag_len as u64) << 48),
    );
    out.push(resolved);
    out.extend_from_slice(fingerprints);

    for pair in offsets.chunks(2) {
        let lo = u64::from(pair[0]);
        let hi = pair
            .get(1)
            .map_or(u64::from(EMPTY_OFFSET), |v| u64::from(*v));
        out.push(lo | (hi << 32));
    }

    for chunk in blob.chunks(8) {
        let mut word = [0u8; 8];
        word[..chunk.len()].copy_from_slice(chunk);
        out.push(u64::from_le_bytes(word));
    }

    Arc::from(out.as_slice())
}

/// Encoded size of an `AnyValue` holding `n` payload bytes.
const fn any_value_len(n: usize) -> usize {
    1 + varint_len(n as u64) + n
}

impl TokenScratch {
    /// Create an empty scratch buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self, registry: &TenantTokenRegistry) {
        self.unsatisfied.clear();
        self.unsatisfied
            .extend(registry.tokens.iter().map(|t| t.key_mask));
        self.values.clear();
        self.values
            .resize(registry.key_names.len(), ValueRef::default());
        self.arena.clear();
        self.fingerprint_buf.clear();
        self.fingerprints.clear();
        self.fingerprints.resize(registry.pairs.len(), 0);
        self.slots.clear();
        self.slots
            .resize(registry.retained.len(), StagedSlot::default());
        self.out.clear();
    }

    /// Clear everything that packing consumes, then size the value slots for
    /// `registry`. Used when a context is rebuilt rather than resolved.
    fn restage(&mut self, n_slots: usize) {
        self.arena.clear();
        self.fingerprints.clear();
        self.slots.clear();
        self.slots.resize(n_slots, StagedSlot::default());
        self.out.clear();
    }

    fn stage(&mut self, bytes: &[u8]) -> (u32, u32) {
        let off = u32::try_from(self.arena.len()).unwrap_or(u32::MAX);
        self.arena.extend_from_slice(bytes);
        (off, u32::try_from(bytes.len()).unwrap_or(u32::MAX))
    }

    fn store(&mut self, key: KeyId, value: &[u8]) {
        let off = u32::try_from(self.arena.len()).unwrap_or(u32::MAX);
        self.arena.extend_from_slice(value);
        self.values[usize::from(key)] = ValueRef {
            off,
            len: u32::try_from(value.len()).unwrap_or(u32::MAX),
            kind: ValueKind::Text,
            present: true,
        };
    }
}

/// Borrowed inputs a receiver hands to token resolution.
///
/// `headers` is an iterator over the receiver's own header representation, so
/// no intermediate header collection is ever materialized. New sources of
/// tenant material (authorization data, for example) are added here rather
/// than at every receiver call site.
pub struct TokenInputs<I> {
    /// Header name and raw value pairs, in arrival order.
    pub headers: I,
    /// Peer socket address, when the receiver has a real socket.
    pub peer_addr: Option<SocketAddr>,
}

impl<I> TokenInputs<I> {
    /// Create inputs from a header iterator.
    pub fn new(headers: I) -> Self {
        Self {
            headers,
            peer_addr: None,
        }
    }

    /// Attach a peer address.
    #[must_use]
    pub fn with_peer_addr(mut self, peer_addr: Option<SocketAddr>) -> Self {
        self.peer_addr = peer_addr;
        self
    }
}

impl TenantTokenRegistry {
    /// Registry epoch, carried in every packed value so a stale context can be
    /// detected after a reconfiguration.
    #[must_use]
    pub fn epoch(&self) -> u16 {
        self.epoch
    }

    /// Returns true when nothing is configured, letting receivers skip the
    /// resolution call entirely.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Name of an interned key, for diagnostics.
    #[must_use]
    pub fn key_name(&self, key: KeyId) -> &str {
        &self.key_names[usize::from(key)]
    }

    /// Key occupying a value slot.
    #[must_use]
    pub fn slot_key(&self, value_slot: u16) -> KeyId {
        self.retained[usize::from(value_slot)]
    }

    /// Value slot a key occupies, or `None` when the key is match-only and its
    /// value is therefore never copied into a request context.
    #[must_use]
    pub fn value_slot(&self, key: KeyId) -> Option<u16> {
        match self.value_slot.get(usize::from(key)).copied() {
            Some(NO_VALUE_SLOT) | None => None,
            Some(slot) => Some(slot),
        }
    }

    /// Interned id of a key name, if the key is known to this registry.
    #[must_use]
    pub fn key_id(&self, name: &str) -> Option<KeyId> {
        self.key_names
            .iter()
            .position(|k| k.as_ref() == name)
            .map(|i| KeyId::try_from(i).expect("key id fits"))
    }

    /// Compile a boundary allowlist into a key-indexed filter.
    #[must_use]
    pub fn compile_filter(&self, policy: &TenantBoundaryPolicy) -> BoundaryFilter {
        let mut allow = vec![false; self.key_names.len()];
        let mut any = false;
        for key in self.compile_policy(policy).iter() {
            allow[usize::from(*key)] = true;
            any = true;
        }
        BoundaryFilter {
            allow: allow.into_boxed_slice(),
            any,
        }
    }

    /// Compile a boundary allowlist into interned key ids.
    ///
    /// Unknown key names are silently dropped: a policy may name keys that
    /// only the other side of the boundary knows about.
    #[must_use]
    pub fn compile_policy(&self, policy: &TenantBoundaryPolicy) -> Box<[KeyId]> {
        policy
            .allow_keys
            .iter()
            .filter_map(|name| self.key_id(name))
            .collect()
    }

    /// Resolve the tenant tokens for one request and pack them into a single
    /// allocation.
    ///
    /// Returns `None` when no token resolved, in which case the request
    /// carries no tenant context at all.
    pub fn resolve<'a, I>(
        &self,
        scratch: &mut TokenScratch,
        inputs: TokenInputs<I>,
    ) -> Option<Arc<[u64]>>
    where
        I: IntoIterator<Item = (&'a str, &'a [u8])>,
    {
        scratch.reset(self);

        // Static extractors first: they never depend on the request headers.
        for extractor in &self.static_extractors {
            match &extractor.value {
                Some(value) => scratch.store(extractor.slot.key, value),
                None => match inputs.peer_addr {
                    Some(addr) => {
                        let rendered = addr.to_string();
                        scratch.store(extractor.slot.key, rendered.as_bytes());
                    }
                    None => continue,
                },
            }
            scratch.unsatisfied[usize::from(extractor.slot.token)] &= !(1u64 << extractor.slot.bit);
        }

        // One pass over the request headers.
        let mut lower: SmallVec<[u8; 64]> = SmallVec::new();
        for (name, value) in inputs.headers {
            lower.clear();
            lower.extend(name.as_bytes().iter().map(u8::to_ascii_lowercase));
            let Ok(lower_str) = std::str::from_utf8(&lower) else {
                continue;
            };
            let Some(header_slot) = self.header_index.get(lower_str) else {
                continue;
            };
            for slot in &header_slot.any_value {
                scratch.store(slot.key, value);
                scratch.unsatisfied[usize::from(slot.token)] &= !(1u64 << slot.bit);
            }
        }

        self.finish_resolve(scratch)
    }

    /// Resolve this pipeline's tokens over a context handed across a boundary.
    ///
    /// The upstream context is never adopted: only keys this receiver's import
    /// policy names are visible, and the tokens rebuilt here are the ones the
    /// downstream pipeline declared. Static extractors still run, so a
    /// dedicated pipeline can mint a `generic_key` identity of its own and
    /// combine it with imported values in a single token.
    pub fn resolve_imported(
        &self,
        scratch: &mut TokenScratch,
        view: &TenantView<'_>,
        allow: &BoundaryFilter,
    ) -> Option<Arc<[u64]>> {
        // A boundary can join two registries -- an engine-scoped topic
        // connects pipeline groups, and each group builds its own. Slot
        // numbers are only meaningful between registries that agree on the
        // layout, which is exactly what the epoch digest states. Disagreement
        // drops the upstream context rather than reading slots as something
        // they are not; the receiver's own static extractors still run.
        if view.epoch() != self.epoch {
            return None;
        }
        scratch.reset(self);

        for extractor in &self.static_extractors {
            let Some(value) = &extractor.value else {
                continue;
            };
            scratch.store(extractor.slot.key, value);
            scratch.unsatisfied[usize::from(extractor.slot.token)] &= !(1u64 << extractor.slot.bit);
        }

        // Value slots are registry positions, and both sides of a boundary
        // compile against the same registry, so an import is a slot read.
        for (slot, key) in self.retained.iter().enumerate() {
            if !allow.admits(*key) {
                continue;
            }
            let slot = u16::try_from(slot).expect("value slot fits");
            let Some(value) = view.slot_value(slot) else {
                continue;
            };
            for target in &self.import_by_key[usize::from(*key)] {
                scratch.store(target.key, value);
                scratch.unsatisfied[usize::from(target.token)] &= !(1u64 << target.bit);
            }
        }

        self.finish_resolve(scratch)
    }

    fn finish_resolve(&self, scratch: &mut TokenScratch) -> Option<Arc<[u64]>> {
        // Tokens whose bits all cleared are resolved.
        let mut resolved: u64 = 0;
        for (idx, word) in scratch.unsatisfied.iter().enumerate() {
            if *word == 0 {
                resolved |= 1u64 << idx;
            }
        }
        if resolved == 0 {
            return None;
        }

        // Probe side of the hash join: one fingerprint per allocated pair.
        let TokenScratch {
            values,
            arena,
            fingerprint_buf,
            fingerprints,
            ..
        } = scratch;
        for (slot, (token, signature)) in self.pairs.iter().enumerate() {
            if resolved & (1u64 << *token) == 0 {
                continue;
            }
            let sig = &self.signatures[usize::from(*signature)];
            fingerprint_buf.clear();
            for key in sig.fixed_keys.iter() {
                let r = values[usize::from(*key)];
                if !r.present {
                    continue;
                }
                let start = r.off as usize;
                let end = start + r.len as usize;
                fingerprint_term(fingerprint_buf, *key, &arena[start..end]);
            }
            fingerprints[slot] = xxh3_64(fingerprint_buf);
        }

        Some(self.pack(scratch, resolved))
    }

    /// Key name of each value slot, in slot order. Only bagged slots encode
    /// it, but the mapping is total so packing can index it directly.
    fn slot_names<'r>(&'r self) -> impl Fn(usize) -> &'r str + 'r {
        move |slot| &self.key_names[usize::from(self.retained[slot])]
    }

    fn pack(&self, scratch: &mut TokenScratch, resolved: u64) -> Arc<[u64]> {
        // Only keys with a value slot contribute bytes. A match-only key is
        // decided entirely by its fingerprint contribution, so its value is
        // dropped here and never travels.
        for (slot, key) in self.retained.iter().enumerate() {
            // A value travels only when a token that asked to retain it
            // resolved. Otherwise the evidence for carrying it is absent, and
            // an unrelated token resolving must not release it.
            scratch.slots[slot] = if resolved & self.retain_mask[slot] == 0 {
                StagedSlot::default()
            } else {
                let v = scratch.values[usize::from(*key)];
                StagedSlot {
                    off: v.off,
                    len: v.len,
                    kind: v.kind,
                    present: v.present,
                    bagged: self.bagged[slot],
                }
            };
        }
        pack_words(scratch, resolved, self.epoch, self.slot_names())
    }

    /// Value carried for a token key, if the key has a value slot and the
    /// request populated it.
    ///
    /// This is an array index, not a search: the slot is fixed at build time.
    /// Exporters drive it from their own `key -> header` map, which is why
    /// token definitions never name a wire header.
    #[must_use]
    pub fn retained_value<'a>(&self, view: &TenantView<'a>, key: KeyId) -> Option<&'a [u8]> {
        view.slot_value(self.value_slot(key)?)
    }

    /// Repack the carried values a boundary policy admits into a fresh context
    /// to hand to another pipeline.
    ///
    /// Nothing else survives: fingerprints, the resolved-token mask, the legacy
    /// captured headers, and every key the policy does not name are dropped, so
    /// a downstream pipeline can neither observe nor re-emit tenant material it
    /// was not granted. The blob is rebuilt rather than masked, because the
    /// packed buffer is shared and any byte left in it stays readable.
    #[must_use]
    pub fn export_boundary(
        &self,
        scratch: &mut TokenScratch,
        view: &TenantView<'_>,
        allow: &[KeyId],
    ) -> Option<Arc<[u64]>> {
        if allow.is_empty() {
            return None;
        }
        scratch.restage(self.retained.len());
        let mut any = false;
        for key in allow {
            let Some(slot) = self.value_slot(*key) else {
                continue;
            };
            let Some(value) = view.slot_value(slot) else {
                continue;
            };
            let range = scratch.stage(value);
            scratch.slots[usize::from(slot)] = StagedSlot {
                off: range.0,
                len: range.1,
                kind: ValueKind::Text,
                present: true,
                bagged: self.bagged[usize::from(slot)],
            };
            any = true;
        }
        if !any {
            return None;
        }
        Some(pack_words(scratch, 0, self.epoch, self.slot_names()))
    }

    /// Build a consumer's probe tables for its ordered conditions.
    ///
    /// Called once at node construction, never on the hot path. `tokens` names
    /// the bound tenant tokens, or `None` to bind every declared token.
    pub fn condition_set(
        &self,
        tokens: Option<&[String]>,
        conditions: &[Condition],
    ) -> Result<ConditionSet, Error> {
        let bound: Vec<TokenIdx> = match tokens {
            None => (0..self.tokens.len())
                .map(|i| TokenIdx::try_from(i).expect("token index fits"))
                .collect(),
            Some(names) => names
                .iter()
                .map(|name| {
                    self.tokens
                        .iter()
                        .position(|t| t.name.as_ref() == name.as_str())
                        .map(|i| TokenIdx::try_from(i).expect("token index fits"))
                        .ok_or_else(|| config_error(format!("unknown tenant token '{name}'")))
                })
                .collect::<Result<_, Error>>()?,
        };

        let mut groups: Vec<ProbeGroup> = Vec::new();
        for (condition_idx, condition) in conditions.iter().enumerate() {
            let condition_idx = ConditionIdx::try_from(condition_idx)
                .map_err(|_| config_error("too many tenant conditions"))?;
            let signature = self.lookup_signature(condition)?;
            let fingerprint = self.literal_fingerprint(condition, signature);

            let position = groups.iter().position(|g| g.signature == signature);
            let group = match position {
                Some(i) => &mut groups[i],
                None => {
                    let pair_slots: Vec<PairSlot> = bound
                        .iter()
                        .filter_map(|t| self.pair_index.get(&(*t, signature)).copied())
                        .collect();
                    let group_tokens: Vec<TokenIdx> = bound
                        .iter()
                        .copied()
                        .filter(|t| self.pair_index.contains_key(&(*t, signature)))
                        .collect();
                    groups.push(ProbeGroup {
                        signature,
                        pair_slots: pair_slots.into_boxed_slice(),
                        tokens: group_tokens.into_boxed_slice(),
                        table: AHashMap::new(),
                    });
                    groups.last_mut().expect("just pushed")
                }
            };
            // First match wins: keep the lowest condition index per key.
            let entry = group.table.entry(fingerprint).or_insert(condition_idx);
            if condition_idx < *entry {
                *entry = condition_idx;
            }
        }

        Ok(ConditionSet {
            groups: groups.into_boxed_slice(),
        })
    }

    fn lookup_signature(&self, condition: &Condition) -> Result<SignatureId, Error> {
        let mut fixed: Vec<KeyId> = Vec::new();
        let mut wildcard: Vec<KeyId> = Vec::new();
        for entry in &condition.entries {
            let key = self
                .key_names
                .iter()
                .position(|k| k.as_ref() == entry.key.as_str())
                .map(|i| KeyId::try_from(i).expect("key id fits"))
                .ok_or_else(|| config_error(format!("unknown tenant token key '{}'", entry.key)))?;
            if entry.value.is_some() {
                fixed.push(key);
            } else {
                wildcard.push(key);
            }
        }
        fixed.sort_unstable();
        wildcard.sort_unstable();
        self.signatures
            .iter()
            .position(|s| {
                s.fixed_keys.as_ref() == fixed.as_slice()
                    && s.wildcard_keys.as_ref() == wildcard.as_slice()
            })
            .map(|i| SignatureId::try_from(i).expect("signature id fits"))
            .ok_or_else(|| config_error("tenant condition was not declared at build time"))
    }

    /// Build-side fingerprint: hash the condition's literal values in the
    /// signature's key order, matching the probe-side layout exactly.
    fn literal_fingerprint(&self, condition: &Condition, signature: SignatureId) -> u64 {
        let sig = &self.signatures[usize::from(signature)];
        let mut buf: Vec<u8> = Vec::new();
        for key in sig.fixed_keys.iter() {
            let name = self.key_name(*key);
            let Some(entry) = condition
                .entries
                .iter()
                .find(|e| e.key.as_str() == name && e.value.is_some())
            else {
                continue;
            };
            let value = entry.value.as_deref().unwrap_or_default();
            fingerprint_term(&mut buf, *key, value.as_bytes());
        }
        xxh3_64(&buf)
    }
}

/// One signature's probe table within a consumer's condition set.
#[derive(Debug)]
struct ProbeGroup {
    signature: SignatureId,
    pair_slots: Box<[PairSlot]>,
    tokens: Box<[TokenIdx]>,
    table: AHashMap<u64, ConditionIdx>,
}

/// A consumer's compiled conditions: one hash probe per signature at runtime.
#[derive(Debug)]
pub struct ConditionSet {
    groups: Box<[ProbeGroup]>,
}

impl ConditionSet {
    /// Evaluate the conditions against a resolved request context and return
    /// the index of the first matching condition.
    ///
    /// Cost is one bit test plus one hash lookup per bound (token, signature)
    /// pair, independent of the number of conditions and of the number of
    /// entries per condition.
    #[must_use]
    pub fn first_match(&self, view: &TenantView<'_>) -> Option<ConditionIdx> {
        let mut best: Option<ConditionIdx> = None;
        for group in self.groups.iter() {
            for (slot, token) in group.pair_slots.iter().zip(group.tokens.iter()) {
                if !view.token_resolved(*token) {
                    continue;
                }
                if let Some(idx) = group.table.get(&view.fingerprint(*slot)) {
                    best = Some(match best {
                        Some(current) if current <= *idx => current,
                        _ => *idx,
                    });
                }
            }
        }
        best
    }
}

// -- Read side ---------------------------------------------------------------

/// Borrowed, zero-copy view over the packed per-request context.
///
/// The same buffer carries the tenant token fingerprints and the request's
/// carried header values, which is what lets a single pointer in the pipeline
/// context replace the former transport header map.
#[derive(Debug, Clone, Copy)]
pub struct TenantView<'a> {
    words: &'a [u64],
}

impl<'a> TenantView<'a> {
    /// Wrap a packed representation.
    #[must_use]
    pub fn new(words: &'a [u64]) -> Self {
        Self { words }
    }

    fn n_fp(&self) -> usize {
        (self.words[0] & 0xFFFF) as usize
    }

    fn n_slots(&self) -> usize {
        ((self.words[0] >> 16) & 0xFFFF) as usize
    }

    /// Registry epoch this context was built against.
    #[must_use]
    pub fn epoch(&self) -> u16 {
        ((self.words[0] >> 32) & 0xFFFF) as u16
    }

    fn bag_len(&self) -> usize {
        ((self.words[0] >> 48) & 0xFFFF) as usize
    }

    /// True when the given token resolved for this request.
    #[must_use]
    pub fn token_resolved(&self, token: TokenIdx) -> bool {
        self.words[1] & (1u64 << token) != 0
    }

    /// True when at least one token resolved.
    #[must_use]
    pub fn any_token_resolved(&self) -> bool {
        self.words[1] != 0
    }

    /// Precomputed fingerprint for one applicable (token, signature) pair.
    #[must_use]
    pub fn fingerprint(&self, slot: PairSlot) -> u64 {
        self.words[HEADER_WORDS + usize::from(slot)]
    }

    /// True when the request carried no values at all.
    ///
    /// Entries are self-delimiting, so the blob needs no stored length: an
    /// empty context is simply one with no blob words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blob().is_empty()
    }

    /// The value region, including up to seven bytes of zero padding.
    ///
    /// Padding is never read: a slot offset addresses a length-delimited
    /// `AnyValue`, and the bag is bounded by `bag_len`, so no reader depends
    /// on the region ending exactly where the last entry does.
    fn blob(&self) -> &'a [u8] {
        let start = HEADER_WORDS + self.n_fp() + self.n_slots().div_ceil(2);
        bytemuck::cast_slice(&self.words[start..])
    }

    fn offset(&self, slot: usize) -> Option<usize> {
        if slot >= self.n_slots() {
            return None;
        }
        let word = self.words[HEADER_WORDS + self.n_fp() + slot / 2];
        let off = if slot.is_multiple_of(2) {
            word as u32
        } else {
            (word >> 32) as u32
        };
        (off != EMPTY_OFFSET).then_some(off as usize)
    }

    /// Value carried in one registry value slot, if the request populated it.
    ///
    /// The offset addresses an encoded `AnyValue`, so this decodes rather than
    /// slices. Callers matching on the value should prefer the fingerprints,
    /// which never look at bytes at all.
    #[must_use]
    pub fn slot_value(&self, slot: u16) -> Option<&'a [u8]> {
        let at = self.offset(usize::from(slot))?;
        any_value_bytes(self.blob(), at)
    }

    /// Append the bagged keys to `dst` as repeated `KeyValue` under `field`.
    ///
    /// The bag is stored without a field tag precisely so the consumer picks
    /// the destination -- resource, scope, log record, span, or exemplar
    /// attributes all differ in field number and in nothing else. Each entry
    /// is copied verbatim; nothing is re-encoded.
    ///
    /// Returns the number of attributes appended.
    pub fn append_attributes(&self, dst: &mut Vec<u8>, field: u32) -> usize {
        let bag = &self.blob()[..self.bag_len()];
        let tag = (u64::from(field) << 3) | 2;
        let mut at = 0usize;
        let mut count = 0usize;
        while at < bag.len() {
            let Some((len, body)) = get_varint(bag, at) else {
                break;
            };
            let Ok(len) = usize::try_from(len) else { break };
            let Some(entry) = bag.get(body..body + len) else {
                break;
            };
            put_varint(dst, tag);
            put_varint(dst, len as u64);
            dst.extend_from_slice(entry);
            at = body + len;
            count += 1;
        }
        count
    }

    /// True when any key was captured into the bag.
    #[must_use]
    pub fn has_attributes(&self) -> bool {
        self.bag_len() > 0
    }
}
