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
/// Symbol meaning "the request did not carry this key at all".
const SYMBOL_ABSENT: u16 = 0;

/// Symbol meaning "the request carried a value no condition declares".
///
/// Every unrecognized value collapses to this one symbol, which is safe
/// because no condition can ever ask for it: conditions are built only from
/// declared literals, whose symbols start at [`FIRST_SYMBOL`].
const SYMBOL_UNKNOWN: u16 = 1;

/// First symbol assigned to a declared literal.
const FIRST_SYMBOL: u16 = 2;

/// Pack a signature's symbols into one word using its build-time layout.
///
/// Wildcard keys contribute a presence bit; fixed keys contribute their
/// symbol. The layout reserves enough bits for every symbol a key can take,
/// so distinct tuples always produce distinct words.
fn pack_symbols(layout: &SignatureLayout, symbol: impl Fn(KeyId) -> u16) -> u64 {
    let mut word = 0u64;
    for (key, shift, mask) in layout.fixed.iter() {
        word |= (u64::from(symbol(*key)) & mask) << shift;
    }
    for (key, shift) in layout.wildcard.iter() {
        if symbol(*key) != SYMBOL_ABSENT {
            word |= 1u64 << shift;
        }
    }
    word
}

/// Bits needed to hold every symbol up to and including `max`.
const fn symbol_width(max: u16) -> u32 {
    let bits = u16::BITS - max.leading_zeros();
    if bits == 0 { 1 } else { bits }
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

/// How one signature's symbols pack into a single word.
///
/// Widths come from the number of literals each key actually declares, so the
/// packing is injective: two different symbol tuples cannot produce the same
/// word. That is what makes an equality test on the word an equality test on
/// the values.
#[derive(Debug, Clone, Default)]
struct SignatureLayout {
    /// `(key, shift, mask)` per fixed key.
    fixed: Box<[(KeyId, u32, u64)]>,
    /// `(key, shift)` per wildcard key; one bit recording presence.
    wildcard: Box<[(KeyId, u32)]>,
}

/// Layout shared by every condition that tests the same set of keys.
///
/// The key order here defines the symbol layout, so the build side and the
/// probe side agree bit for bit.
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
    /// Bloom filter over registered header names, probed before hashing.
    header_probe: u64,
    static_extractors: Vec<StaticExtractor>,
    signatures: Vec<Signature>,
    /// Per key id: declared literal -> symbol. The map owns the literal, so a
    /// lookup verifies byte equality against the operator's configured value
    /// and a hash collision cannot produce a match.
    key_literals: Vec<AHashMap<Box<[u8]>, u16>>,
    /// Per signature: the bit layout its symbols pack into.
    layouts: Vec<SignatureLayout>,
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
    /// OTLP field number the bagged run is tagged with, fixed at build time
    /// by the consumer that initialized the registry.
    bag_field: u32,
    /// Reverse of `retained`, indexed by key id; [`NO_VALUE_SLOT`] for keys
    /// that are match-only.
    value_slot: Vec<u16>,
    /// Indexed by value slot: the tokens that declared this key with
    /// `retain: true`. A slot is populated only when one of them resolves, so
    /// a value never travels without the evidence that justified carrying it.
    retain_mask: Vec<u64>,
    /// A well-formed context in which nothing resolved, built once so that a
    /// node minting a value mid-pipeline for a request that arrived without
    /// tenant context pays an `Arc` clone instead of a resolve.
    empty: Arc<[u64]>,
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
    key_literals: Vec<AHashMap<Box<[u8]>, u16>>,
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
    bag_field: Option<u32>,
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
        self.key_literals.push(AHashMap::new());
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

    /// Declare the OTLP field number the bagged attributes are tagged with.
    ///
    /// The consumer of the bag is also what initializes the registry -- the
    /// SDK sets up the pipeline and later reads the bytes back -- so the
    /// destination is known here, and the run can be encoded already tagged.
    /// Defaults to scope attributes.
    #[must_use]
    pub fn with_bag_field(mut self, field: u32) -> Self {
        self.bag_field = Some(field);
        self
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
            // Every literal a condition can match on becomes a symbol, so
            // resolution can decide membership by exact lookup rather than by
            // trusting a hash.
            for entry in &condition.entries {
                let Some(value) = entry.value.as_deref() else {
                    continue;
                };
                let key = self.intern_key(&entry.key);
                let table = &mut self.key_literals[usize::from(key)];
                // Recycling a symbol would let two distinct literals compare
                // equal, so exhausting the space is an error rather than a wrap.
                let next = u16::try_from(table.len())
                    .ok()
                    .and_then(|n| n.checked_add(FIRST_SYMBOL))
                    .ok_or_else(|| {
                        config_error(format!(
                            "tenant key '{}' declares more distinct values than \
                             the symbol space holds ({} max)",
                            entry.key,
                            u16::MAX - FIRST_SYMBOL,
                        ))
                    })?;
                let _ = table.entry(value.as_bytes().into()).or_insert(next);
            }
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
    ///
    /// Fails when a signature's symbols do not fit one word, which would cost
    /// the guarantee that comparing words is comparing values.
    pub fn build(self, generation: u16) -> Result<TenantTokenRegistry, Error> {
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

        // Assign each signature a bit layout wide enough for every symbol its
        // keys can take. Injectivity is the whole point, so a layout that does
        // not fit is a configuration error rather than a silent narrowing.
        let mut layouts: Vec<SignatureLayout> = Vec::with_capacity(self.signatures.len());
        for sig in &self.signatures {
            let mut shift = 0u32;
            let mut fixed = Vec::with_capacity(sig.fixed_keys.len());
            for key in sig.fixed_keys.iter() {
                let declared = self.key_literals[usize::from(*key)].len();
                let max = u16::try_from(declared)
                    .ok()
                    .and_then(|n| n.checked_add(FIRST_SYMBOL))
                    .map_or(u16::MAX, |n| n.saturating_sub(1))
                    .max(SYMBOL_UNKNOWN);
                let width = symbol_width(max);
                fixed.push((*key, shift, (1u64 << width) - 1));
                shift += width;
            }
            let mut wildcard = Vec::with_capacity(sig.wildcard_keys.len());
            for key in sig.wildcard_keys.iter() {
                wildcard.push((*key, shift));
                shift += 1;
            }
            if shift > u64::BITS {
                let names: Vec<&str> = sig
                    .required_keys
                    .iter()
                    .map(|k| self.key_names[usize::from(*k)].as_ref())
                    .collect();
                return Err(config_error(format!(
                    "tenant condition over keys [{}] needs {shift} bits of symbol \
                     space but only {} are available; reduce the number of keys \
                     or the number of declared values per key",
                    names.join(", "),
                    u64::BITS,
                )));
            }
            layouts.push(SignatureLayout {
                fixed: fixed.into_boxed_slice(),
                wildcard: wildcard.into_boxed_slice(),
            });
        }

        let mut registry = TenantTokenRegistry {
            epoch,
            key_names: self.key_names,
            tokens: self.tokens,
            header_probe: self
                .header_index
                .keys()
                .map(|name| 1u64 << header_probe_bit(name.as_bytes()))
                .fold(0, |acc, bit| acc | bit),
            header_index: self.header_index,
            static_extractors: self.static_extractors,
            signatures: self.signatures,
            key_literals: self.key_literals,
            layouts,
            pairs: self.pairs,
            pair_index: self.pair_index,
            import_by_key,
            retained: self.retained,
            bagged: self.bagged,
            bag_field: self.bag_field.unwrap_or(attribute_field::SCOPE),
            value_slot,
            retain_mask,
            empty: Arc::from(Vec::new()),
        };

        // Pack with nothing resolved. Every signature word is zero, and no
        // condition can pack to zero because each one constrains at least one
        // key to a declared literal, whose symbol is never SYMBOL_ABSENT. The
        // empty context therefore matches nothing, which is the fail-closed
        // answer for a request that carried no tenant evidence.
        let mut scratch = TokenScratch::new();
        scratch.reset(&registry);
        registry.empty = registry.pack(&mut scratch, 0);
        Ok(registry)
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
    /// Per key id: the symbol its value resolved to.
    symbols: Vec<u16>,
    /// Per pair slot: the packed symbol word. Named for its role in the join.
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

/// OTLP attributes field numbers a bagged context can be tagged with.
///
/// These are the destinations a consumer chooses between when initializing a
/// registry. They differ in field number and in nothing else, which is why one
/// encoding serves all of them.
pub mod attribute_field {
    /// `Resource.attributes`.
    pub const RESOURCE: u32 = 1;

    /// `InstrumentationScope.attributes`. The default: scope attributes are
    /// shared by every record under one `ScopeLogs`/`ScopeSpans`, and batches
    /// are already partitioned by tenant conditions, so one batch means one
    /// scope means one copy.
    pub const SCOPE: u32 = 3;

    /// `LogRecord.attributes`.
    pub const LOG_RECORD: u32 = 6;

    /// `Exemplar.filtered_attributes`.
    pub const EXEMPLAR: u32 = 7;

    /// `Span.attributes`.
    pub const SPAN: u32 = 9;
}

/// Cheap discriminator over a header name, used as a one-word Bloom filter.
///
/// A request carries far more headers than any pipeline registers -- content
/// type, user agent, encoding, timeouts -- and hashing all of them to discover
/// that is the dominant cost of resolution. Length and first byte separate
/// real header names well and are available without touching the rest of the
/// string, so an unregistered name is usually rejected before it is hashed or
/// case-folded.
fn header_probe_bit(name: &[u8]) -> u32 {
    let first = name.first().copied().unwrap_or(0).to_ascii_lowercase();
    ((u32::from(first).wrapping_mul(31)) ^ (name.len() as u32).wrapping_mul(7)) & 63
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
/// `<tag> <len> <KeyValue>` entries already tagged with the consumer's
/// attributes field, so the region is a complete repeated field and is read
/// whole rather than entry by entry. Value-only entries follow as bare
/// `<len> <AnyValue>`, which no consumer reads as a run.
fn pack_words<'n>(
    scratch: &mut TokenScratch,
    resolved: u64,
    epoch: u16,
    bag_field: u32,
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
        put_varint(blob, (u64::from(bag_field) << 3) | 2);
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

    // One bulk copy into a zero-padded tail. Feeding the words eight bytes at
    // a time costs more than the memcpy it replaces: the trailing partial
    // chunk makes every iteration a variable-length copy, so the compiler
    // cannot reduce any of them to a plain load.
    let tail = out.len();
    out.resize(tail + blob.len().div_ceil(8), 0);
    bytemuck::cast_slice_mut::<u64, u8>(&mut out[tail..])[..blob.len()].copy_from_slice(blob);

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
        self.symbols.clear();
        self.symbols.resize(registry.key_names.len(), SYMBOL_ABSENT);
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

    /// A shared context in which no token resolved and no value travels.
    ///
    /// Callers that mint a value mid-pipeline use this as the source to
    /// [`rewrite`](Self::rewrite) when the request arrived without one.
    #[must_use]
    pub fn empty_context(&self) -> &Arc<[u64]> {
        &self.empty
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
            let bytes = name.as_bytes();
            if self.header_probe & (1u64 << header_probe_bit(bytes)) == 0 {
                continue;
            }
            // gRPC and HTTP/2 require lowercase names, so the copy is usually
            // avoidable; only fold a name that actually needs it.
            let lower_str = if bytes.iter().any(u8::is_ascii_uppercase) {
                lower.clear();
                lower.extend(bytes.iter().map(u8::to_ascii_lowercase));
                let Ok(folded) = std::str::from_utf8(&lower) else {
                    continue;
                };
                folded
            } else {
                name
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

        // Probe side of the join. Each key's value is first resolved to a
        // symbol by exact lookup against the declared literals -- the map owns
        // those bytes and compares them, so a hash collision cannot manufacture
        // a match. Only then are symbols packed into the per-pair word.
        let TokenScratch {
            values,
            arena,
            symbols,
            fingerprints,
            ..
        } = scratch;
        for (key, r) in values.iter().enumerate() {
            symbols[key] = if !r.present {
                SYMBOL_ABSENT
            } else if self.key_literals[key].is_empty() {
                // No condition tests this key by value, so the lookup can only
                // miss. Skipping it avoids hashing the value for keys that are
                // carried but never matched on.
                SYMBOL_UNKNOWN
            } else {
                self.key_literals[key]
                    .get(&arena[r.off as usize..(r.off + r.len) as usize])
                    .copied()
                    .unwrap_or(SYMBOL_UNKNOWN)
            };
        }

        for (slot, (token, signature)) in self.pairs.iter().enumerate() {
            if resolved & (1u64 << *token) == 0 {
                continue;
            }
            fingerprints[slot] = pack_symbols(&self.layouts[usize::from(*signature)], |key| {
                symbols[usize::from(key)]
            });
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
        pack_words(
            scratch,
            resolved,
            self.epoch,
            self.bag_field,
            self.slot_names(),
        )
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
        Some(pack_words(
            scratch,
            0,
            self.epoch,
            self.bag_field,
            self.slot_names(),
        ))
    }

    /// Rebuild a context with one key's value replaced.
    ///
    /// This is how a node mints tenant material of its own mid-pipeline: the
    /// partition processor, for example, derives one outbound context per
    /// partition from a single inbound one. The packed buffer is immutable and
    /// shared, so the value cannot be patched in place; a fresh context is
    /// built instead, at one allocation per call. That is still cheaper than
    /// what the header world charged for the same operation, where appending
    /// to a shared `Arc<Vec<TransportHeader>>` deep-cloned every header.
    ///
    /// The rewritten key's symbol is re-resolved against the declared literals
    /// and its fields in every affected signature word are rewritten with it.
    /// Leaving the old symbol in place would be the dangerous thing: routing
    /// would keep matching the value the context no longer carries.
    ///
    /// Returns `None` when `key` has no value slot -- a match-only key holds no
    /// bytes to replace, and inventing a slot for it would silently change what
    /// the configuration asked to travel.
    #[must_use]
    pub fn rewrite(
        &self,
        scratch: &mut TokenScratch,
        view: &TenantView<'_>,
        key: KeyId,
        value: &[u8],
    ) -> Option<Arc<[u64]>> {
        let target = self.value_slot(key)?;

        // Carry every other slot across unchanged. Values live only in the
        // blob, so restaging from the view reproduces them exactly; match-only
        // keys have nothing to restage, which is why their symbols have to come
        // from the existing signature words below.
        scratch.restage(self.retained.len());
        for slot in 0..self.retained.len() {
            let slot_u16 = u16::try_from(slot).expect("slot fits");
            let bytes = if slot_u16 == target {
                Some(value)
            } else {
                view.slot_value(slot_u16)
            };
            let Some(bytes) = bytes else {
                continue;
            };
            let range = scratch.stage(bytes);
            scratch.slots[slot] = StagedSlot {
                off: range.0,
                len: range.1,
                kind: ValueKind::Text,
                present: true,
                bagged: self.bagged[slot],
            };
        }

        // Re-resolve the new value to a symbol by exact lookup, exactly as
        // resolution does, so an undeclared value lands on SYMBOL_UNKNOWN
        // rather than impersonating a declared one.
        let symbol = if self.key_literals[usize::from(key)].is_empty() {
            SYMBOL_UNKNOWN
        } else {
            self.key_literals[usize::from(key)]
                .get(value)
                .copied()
                .unwrap_or(SYMBOL_UNKNOWN)
        };

        // Every other key's symbol is already encoded in the existing words, so
        // only the rewritten key's fields move.
        scratch.fingerprints.clear();
        for (pair, (_, signature)) in self.pairs.iter().enumerate() {
            let layout = &self.layouts[usize::from(*signature)];
            let mut word = view.signature_word(u16::try_from(pair).expect("pair fits"));
            for (k, shift, mask) in layout.fixed.iter() {
                if *k == key {
                    word &= !(mask << shift);
                    word |= (u64::from(symbol) & mask) << shift;
                }
            }
            for (k, shift) in layout.wildcard.iter() {
                if *k == key {
                    // The key is present by construction: it was just written.
                    word |= 1u64 << shift;
                }
            }
            scratch.fingerprints.push(word);
        }

        Some(pack_words(
            scratch,
            view.resolved_mask(),
            self.epoch,
            self.bag_field,
            self.slot_names(),
        ))
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
            let word = self.literal_word(condition, signature)?;

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
            let entry = group.table.entry(word).or_insert(condition_idx);
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
    /// The word a request must produce to satisfy `condition`.
    ///
    /// Built from the same symbols and the same layout as the request side, so
    /// the comparison in [`ConditionSet::first_match`] is an equality test on
    /// values rather than on a digest of them.
    fn literal_word(&self, condition: &Condition, signature: SignatureId) -> Result<u64, Error> {
        let layout = &self.layouts[usize::from(signature)];
        // Fail closed on an undeclared literal. It would otherwise take
        // SYMBOL_UNKNOWN, which is the symbol every unrecognized request value
        // takes, and the condition would match all of them at once.
        for (key, _, _) in layout.fixed.iter() {
            let name = self.key_name(*key);
            let literal = condition
                .entries
                .iter()
                .find(|e| e.key.as_str() == name)
                .and_then(|e| e.value.as_deref())
                .unwrap_or_default();
            if !self.key_literals[usize::from(*key)].contains_key(literal.as_bytes()) {
                return Err(config_error(format!(
                    "tenant condition tests '{name}' against a value that was \
                     never declared to the registry; every literal a condition \
                     can match must be interned at build time so that matching \
                     is an equality test rather than a hash comparison"
                )));
            }
        }
        Ok(pack_symbols(layout, |key| {
            let name = self.key_name(key);
            condition
                .entries
                .iter()
                .find(|e| e.key.as_str() == name)
                .and_then(|e| e.value.as_deref())
                .map_or(SYMBOL_UNKNOWN, |value| {
                    self.key_literals[usize::from(key)]
                        .get(value.as_bytes())
                        .copied()
                        .unwrap_or(SYMBOL_UNKNOWN)
                })
        }))
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
                if let Some(idx) = group.table.get(&view.signature_word(*slot)) {
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

    /// The full resolved-token mask, needed when deriving a context from this
    /// one so the derived context reports the same tokens as resolved.
    #[must_use]
    pub fn resolved_mask(&self) -> u64 {
        self.words[1]
    }

    /// Packed symbol word for one applicable (token, signature) pair.
    ///
    /// This is a dictionary encoding of the request's values, not a digest of
    /// them: comparing two words compares the values exactly.
    #[must_use]
    pub fn signature_word(&self, slot: PairSlot) -> u64 {
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

    /// The bagged keys, as a complete OTLP repeated `KeyValue` field.
    ///
    /// The run is already tagged with the field number the registry was
    /// initialized with, so this is the payload itself and not an input to an
    /// encoder: a consumer splices the slice in with one copy, or borrows it
    /// outright. Nothing here is re-encoded, re-tagged or walked per entry.
    #[must_use]
    pub fn attributes(&self) -> &'a [u8] {
        &self.blob()[..self.bag_len()]
    }

    /// True when any key was captured into the bag.
    #[must_use]
    pub fn has_attributes(&self) -> bool {
        self.bag_len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{Condition, Entry, TenantTokenSpec, TenantTokens};

    fn entry(key: &str, value: &str) -> Entry {
        Entry {
            key: key.to_owned(),
            value: Some(value.to_owned()),
        }
    }

    fn condition(pairs: &[(&str, &str)]) -> Condition {
        Condition {
            entries: pairs.iter().map(|(k, v)| entry(k, v)).collect(),
        }
    }

    /// Build a registry over two match-only keys read from transport headers.
    fn fixture(conditions: &[Condition]) -> TenantTokenRegistry {
        let extractors = [("x-tenant-id", "tenant_id"), ("x-env", "env")]
            .iter()
            .map(|(wire, key)| Extractor::TransportHeader {
                key: (*key).to_owned(),
                transport_header: (*wire).to_owned(),
                retain: false,
                bag: false,
            })
            .collect();
        let mut tokens = TenantTokens::default();
        let _ = tokens.insert("gateway".to_owned(), TenantTokenSpec { extractors });
        let mut builder = TenantTokenRegistryBuilder::new();
        builder.add_tokens(&tokens).expect("tokens compile");
        builder
            .declare_conditions(None, conditions)
            .expect("conditions declare");
        builder.build(1).expect("layout fits")
    }

    fn resolve(reg: &TenantTokenRegistry, headers: &[(&str, &[u8])]) -> Arc<[u64]> {
        let mut scratch = TokenScratch::new();
        reg.resolve(
            &mut scratch,
            TokenInputs::new(headers.iter().map(|(k, v)| (*k, *v))),
        )
        .expect("token resolves")
    }

    /// Build a registry whose `tenant_id` is retained, so it has a value slot
    /// that can be rewritten, and whose `env` is match-only.
    fn rewritable(conditions: &[Condition]) -> TenantTokenRegistry {
        let extractors = vec![
            Extractor::TransportHeader {
                key: "tenant_id".to_owned(),
                transport_header: "x-tenant-id".to_owned(),
                retain: true,
                bag: false,
            },
            Extractor::TransportHeader {
                key: "env".to_owned(),
                transport_header: "x-env".to_owned(),
                retain: false,
                bag: false,
            },
        ];
        let mut tokens = TenantTokens::default();
        let _ = tokens.insert("gateway".to_owned(), TenantTokenSpec { extractors });
        let mut builder = TenantTokenRegistryBuilder::new();
        builder.add_tokens(&tokens).expect("tokens compile");
        builder
            .declare_conditions(None, conditions)
            .expect("conditions declare");
        builder.build(1).expect("layout fits")
    }

    /// Scenario: a node derives a new context from an existing one by replacing
    /// one retained key's value, where both the old and the new value are
    /// declared literals of competing conditions.
    /// Guarantees: the derived context routes by its new value and never by the
    /// value it replaced, other keys (including match-only keys, whose bytes
    /// never travelled) keep their original matching behavior, and the derived
    /// value reads back byte-for-byte.
    #[test]
    fn rewrite_moves_matching_to_the_new_value() {
        let conds = [
            condition(&[("tenant_id", "acme"), ("env", "prod")]),
            condition(&[("tenant_id", "globex"), ("env", "prod")]),
            condition(&[("tenant_id", "globex"), ("env", "dev")]),
        ];
        let reg = rewritable(&conds);
        let set = reg.condition_set(None, &conds).expect("conditions bind");

        let words = resolve(&reg, &[("x-tenant-id", b"acme"), ("x-env", b"prod")]);
        assert_eq!(set.first_match(&TenantView::new(&words)), Some(0));

        let key = reg.key_id("tenant_id").expect("declared");
        let mut scratch = TokenScratch::new();
        let derived = reg
            .rewrite(&mut scratch, &TenantView::new(&words), key, b"globex")
            .expect("tenant_id is retained");
        let view = TenantView::new(&derived);

        // Routing follows the new value, and `env` still decides between the
        // two globex conditions even though its bytes never travelled.
        assert_eq!(set.first_match(&view), Some(1));
        assert_eq!(reg.retained_value(&view, key), Some(b"globex".as_slice()));
        assert_eq!(
            view.resolved_mask(),
            TenantView::new(&words).resolved_mask()
        );

        // A value no condition declares must not impersonate one that is.
        let unknown = reg
            .rewrite(&mut scratch, &TenantView::new(&words), key, b"acme-2")
            .expect("tenant_id is retained");
        let unknown = TenantView::new(&unknown);
        assert_eq!(set.first_match(&unknown), None);
        assert_eq!(
            reg.retained_value(&unknown, key),
            Some(b"acme-2".as_slice())
        );
    }

    /// Scenario: requests are probed against conditions whose declared literals
    /// share keys, including values that differ only in one byte or in length.
    /// Guarantees: a condition matches a request only when every declared
    /// `key: value` pair is byte-for-byte equal to the value the request
    /// carried, so no request can be routed to another tenant's condition.
    #[test]
    fn matching_is_exact() {
        let conditions = [
            condition(&[("tenant_id", "acme"), ("env", "prod")]),
            condition(&[("tenant_id", "globex"), ("env", "prod")]),
            condition(&[("tenant_id", "acme"), ("env", "staging")]),
        ];
        let reg = fixture(&conditions);
        let set = reg
            .condition_set(None, &conditions)
            .expect("condition set builds");

        let cases: [(&[u8], &[u8], Option<u16>); 8] = [
            // Each declared combination selects its own condition, and never
            // the one that shares a key with it.
            (b"acme", b"prod", Some(0)),
            (b"globex", b"prod", Some(1)),
            (b"acme", b"staging", Some(2)),
            // A combination no condition declares matches nothing, even though
            // both of its values are declared elsewhere.
            (b"globex", b"staging", None),
            // Near misses: one byte differs, a prefix, and a suffix.
            (b"acmf", b"prod", None),
            (b"acm", b"prod", None),
            (b"acme ", b"prod", None),
            // An undeclared value collapses to SYMBOL_UNKNOWN, which no
            // condition can name.
            (b"initech", b"prod", None),
        ];
        for (tenant, env, expected) in cases {
            let words = resolve(&reg, &[("x-tenant-id", tenant), ("x-env", env)]);
            let view = TenantView::new(&words);
            assert_eq!(
                set.first_match(&view),
                expected,
                "tenant={:?} env={:?}",
                String::from_utf8_lossy(tenant),
                String::from_utf8_lossy(env),
            );
        }
    }

    /// Scenario: the keys a condition tests are declared with `retain: false`,
    /// so the request context carries none of their bytes.
    /// Guarantees: matching stays exact without carrying the values, because
    /// verification happens at the receiver against the registry's own copy of
    /// each literal rather than against anything held in the request.
    #[test]
    fn match_only_keys_carry_no_bytes() {
        let conditions = [condition(&[("tenant_id", "acme"), ("env", "prod")])];
        let reg = fixture(&conditions);
        let set = reg
            .condition_set(None, &conditions)
            .expect("condition set builds");

        let words = resolve(&reg, &[("x-tenant-id", b"acme"), ("x-env", b"prod")]);
        let view = TenantView::new(&words);
        assert_eq!(set.first_match(&view), Some(0));
        assert!(view.is_empty(), "match-only keys must carry no bytes");
        assert!(!view.has_attributes());
    }

    /// Scenario: a consumer builds a condition set testing a literal that was
    /// never interned by `declare_conditions`.
    /// Guarantees: the build fails instead of assigning the literal the same
    /// symbol every unrecognized request value takes, which would otherwise
    /// make the condition match all of them at once.
    #[test]
    fn undeclared_literal_is_rejected() {
        let declared = [condition(&[("tenant_id", "acme"), ("env", "prod")])];
        let reg = fixture(&declared);

        let undeclared = [condition(&[("tenant_id", "initech"), ("env", "prod")])];
        let err = reg
            .condition_set(None, &undeclared)
            .expect_err("undeclared literal must fail closed");
        assert!(
            format!("{err:?}").contains("never declared"),
            "unexpected error: {err:?}"
        );
    }

    /// Scenario: a bagged key is packed into the request context and read back
    /// through `attributes()`.
    /// Guarantees: the bytes are a complete, correctly tagged OTLP repeated
    /// `KeyValue` field for the destination the registry was built for, so a
    /// consumer can copy them without inspecting or rewriting them.
    #[test]
    fn bagged_attributes_are_wire_exact() {
        let extractors = vec![Extractor::TransportHeader {
            key: "tenant_id".to_owned(),
            transport_header: "x-tenant-id".to_owned(),
            retain: false,
            bag: true,
        }];
        let mut tokens = TenantTokens::default();
        let _ = tokens.insert("gateway".to_owned(), TenantTokenSpec { extractors });
        let mut builder = TenantTokenRegistryBuilder::new().with_bag_field(attribute_field::SCOPE);
        builder.add_tokens(&tokens).expect("tokens compile");
        let reg = builder.build(1).expect("layout fits");

        let words = resolve(&reg, &[("x-tenant-id", b"tenant-a")]);
        let view = TenantView::new(&words);

        // ScopeAttributes: field 3, length-delimited.
        let mut expected = vec![(3u8 << 3) | 2, 23];
        // KeyValue.key: field 1, "tenant_id".
        expected.extend_from_slice(&[0x0a, 9]);
        expected.extend_from_slice(b"tenant_id");
        // KeyValue.value: field 2, an AnyValue holding a string.
        expected.extend_from_slice(&[0x12, 10, 0x0a, 8]);
        expected.extend_from_slice(b"tenant-a");

        assert_eq!(view.attributes(), expected.as_slice());
        assert!(view.has_attributes());
    }

    /// Scenario: every declared value tuple is packed into the one-word
    /// signature used as the probe key, with enough declared values per key
    /// that a narrowed bit field would force two tuples to share a word.
    /// Guarantees: the packing is injective over declared values, so equality
    /// of packed words is equality of the underlying values and the word can
    /// serve as a hash key without risking a cross-tenant match.
    #[test]
    fn signature_packing_is_injective() {
        const TENANTS: [&str; 3] = ["acme", "globex", "initech"];
        const ENVS: [&str; 3] = ["prod", "staging", "dev"];

        let conditions: Vec<Condition> = TENANTS
            .iter()
            .zip(ENVS.iter())
            .map(|(t, e)| condition(&[("tenant_id", t), ("env", e)]))
            .collect();
        let reg = fixture(&conditions);

        let mut seen: AHashMap<u64, (&str, &str)> = AHashMap::new();
        for tenant in TENANTS {
            for env in ENVS {
                let words = resolve(
                    &reg,
                    &[
                        ("x-tenant-id", tenant.as_bytes()),
                        ("x-env", env.as_bytes()),
                    ],
                );
                let view = TenantView::new(&words);
                let previous = seen.insert(view.signature_word(0), (tenant, env));
                assert!(
                    previous.is_none(),
                    "declared tuples must pack to distinct words: \
                     {tenant}/{env} collided with {previous:?}",
                );
            }
        }
        assert_eq!(seen.len(), TENANTS.len() * ENVS.len());
    }
}
