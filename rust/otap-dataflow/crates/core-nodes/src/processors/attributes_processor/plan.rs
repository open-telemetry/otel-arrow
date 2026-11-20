// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use enrichment::{SourceReader, invalid_config};
use otel_arrow_dfe_engine::pdata_context::{Metadata, NodeBindings};
use otel_arrow_dfe_pdata::otap::transform::{
    TransformStats, apply_attribute_transform, apply_attribute_transform_with_values,
};

/// One action with an optional telemetry-signal filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedAction {
    /// The attribute operation.
    #[serde(flatten)]
    pub action: Action,
    /// Selected signals; omission selects all signals.
    #[serde(
        default,
        deserialize_with = "enrichment::non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub signals: Option<Vec<ActionSignal>>,
}

/// A telemetry signal, independent of the selected attribute domains.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionSignal {
    /// Log records.
    Logs,
    /// Metric data.
    Metrics,
    /// Spans.
    Traces,
}

impl ScopedAction {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        if self.signals.as_ref().is_some_and(Vec::is_empty) {
            return Err(invalid_config("signals cannot be empty"));
        }
        match &self.action {
            Action::Insert(AttributeValueConfig {
                value,
                literal_type,
                context_value,
                ..
            })
            | Action::Upsert(AttributeValueConfig {
                value,
                literal_type,
                context_value,
                ..
            })
            | Action::Update(AttributeValueConfig {
                value,
                literal_type,
                context_value,
                ..
            }) => match (value, context_value) {
                (Some(value), None) => value.to_literal(*literal_type).map(|_| ()),
                (None, Some(source)) if literal_type.is_none() => {
                    enrichment::validate_source(source)
                }
                (None, Some(_)) => Err(invalid_config(
                    "attribute type is only valid with a literal value",
                )),
                _ => Err(invalid_config(
                    "attribute action requires exactly one of value or context_value",
                )),
            },
            _ => Ok(()),
        }
    }

    fn applies(&self, signal: SignalType) -> bool {
        self.signals.as_ref().is_none_or(|signals| {
            signals.iter().any(|selected| {
                matches!(
                    (selected, signal),
                    (ActionSignal::Logs, SignalType::Logs)
                        | (ActionSignal::Metrics, SignalType::Metrics)
                        | (ActionSignal::Traces, SignalType::Traces)
                )
            })
        })
    }
}

pub(super) fn signal_index(signal: SignalType) -> usize {
    match signal {
        SignalType::Logs => 0,
        SignalType::Metrics => 1,
        SignalType::Traces => 2,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Operation {
    Insert,
    Upsert,
    Update,
}

pub(super) fn effective_value_actions(
    actions: &[ScopedAction],
    signal: SignalType,
) -> impl Iterator<Item = (Operation, &AttributeValueConfig)> {
    let mut seen = BTreeSet::new();
    actions
        .iter()
        .rev()
        .filter(move |action| action.applies(signal))
        .filter_map(move |scoped| {
            let (operation, action) = match &scoped.action {
                Action::Insert(action) => (Operation::Insert, action),
                Action::Upsert(action) => (Operation::Upsert, action),
                Action::Update(action) => (Operation::Update, action),
                _ => return None,
            };
            seen.insert((operation, action.key.as_str()))
                .then_some((operation, action))
        })
}

struct DynamicValue {
    key: String,
    source: SourceReader,
}

#[derive(Clone, Copy)]
pub(super) enum SourceBindings<'a> {
    /// Validate destinations without retaining executable dynamic reads.
    ValidateOnly,
    /// Bind effective sources during construction or reconfiguration.
    Node(&'a NodeBindings),
}

pub(super) struct CompiledTransform {
    pub(super) transform: AttributesTransform,
    values: Box<[DynamicValue]>,
    has_literals: bool,
}

pub(super) struct ResolvedTransform<'a> {
    pub(super) transform: &'a AttributesTransform,
    reads: &'a [DynamicValue],
    values: SmallVec<[Option<LiteralValue>; 4]>,
    has_literals: bool,
}

impl ResolvedTransform<'_> {
    pub(super) fn is_noop(&self) -> bool {
        !self.has_literals
            && self.values.iter().all(Option::is_none)
            && self.transform.rename.is_none()
            && self.transform.delete.is_none()
            && self.transform.hash.is_none()
    }

    fn value<'a>(&'a self, key: &str, configured: &'a LiteralValue) -> Option<&'a LiteralValue> {
        match self
            .reads
            .binary_search_by(|read| read.key.as_str().cmp(key))
        {
            Ok(index) => self.values[index].as_ref(),
            Err(_) => Some(configured),
        }
    }

    pub(super) fn apply(
        &self,
        records: &mut OtapArrowRecords,
        payload: ArrowPayloadType,
        compute_stats: bool,
    ) -> Result<Option<TransformStats>, EngineError> {
        if self.reads.is_empty() {
            return apply_attribute_transform(records, payload, self.transform, compute_stats)
                .map_err(Into::into);
        }
        apply_attribute_transform_with_values(
            records,
            payload,
            self.transform,
            |key, configured| self.value(key, configured),
            compute_stats,
        )
        .map_err(Into::into)
    }
}

pub(super) fn is_noop(transform: &AttributesTransform) -> bool {
    transform.rename.is_none()
        && transform.delete.is_none()
        && transform.insert.is_none()
        && transform.upsert.is_none()
        && transform.update.is_none()
        && transform.hash.is_none()
}

fn values_transform(
    inserts: BTreeMap<String, LiteralValue>,
    upserts: BTreeMap<String, LiteralValue>,
    updates: BTreeMap<String, LiteralValue>,
) -> AttributesTransform {
    AttributesTransform {
        insert: (!inserts.is_empty()).then(|| InsertTransform::new(inserts)),
        upsert: (!upserts.is_empty()).then(|| UpsertTransform::new(upserts)),
        update: (!updates.is_empty()).then(|| UpdateTransform::new(updates)),
        ..Default::default()
    }
}

impl CompiledTransform {
    pub(super) fn has_dynamic_values(&self) -> bool {
        !self.values.is_empty()
    }

    pub(super) fn is_noop(&self) -> bool {
        is_noop(&self.transform) && !self.has_dynamic_values()
    }

    pub(super) fn new(
        actions: &[ScopedAction],
        signal: SignalType,
        bindings: SourceBindings<'_>,
    ) -> Result<Self, ConfigError> {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();
        let mut inserts = BTreeMap::new();
        let mut upserts = BTreeMap::new();
        let mut updates = BTreeMap::new();
        let mut hashes = BTreeMap::new();
        for (operation, action) in effective_value_actions(actions, signal) {
            let destination = match operation {
                Operation::Insert => &mut inserts,
                Operation::Upsert => &mut upserts,
                Operation::Update => &mut updates,
            };
            _ = destination.insert(action.key.as_str(), action);
        }
        for scoped in actions.iter().filter(|action| action.applies(signal)) {
            match &scoped.action {
                Action::Rename {
                    source_key,
                    destination_key,
                } => {
                    _ = renames.insert(source_key.clone(), destination_key.clone());
                }
                Action::Delete { key } => {
                    _ = deletes.insert(key.clone());
                }
                Action::Insert(_) | Action::Upsert(_) | Action::Update(_) => {}
                Action::Hash {
                    key,
                    algorithm: HashAlgorithm::Sha256,
                    salt,
                } => {
                    _ = hashes.insert(key.clone(), HashSpec::sha256(salt.clone().into_inner()));
                }
                Action::Unsupported => {}
            }
        }
        // Validate all configured destinations even when their sources are unavailable.
        let placeholder = |sources: &BTreeMap<&str, &AttributeValueConfig>| {
            sources
                .keys()
                .map(|key| ((*key).to_owned(), LiteralValue::Int(0)))
                .collect()
        };
        let mut transform = values_transform(
            placeholder(&inserts),
            placeholder(&upserts),
            placeholder(&updates),
        );
        transform.rename = (!renames.is_empty()).then(|| RenameTransform::new(renames));
        transform.delete = (!deletes.is_empty()).then(|| DeleteTransform::new(deletes));
        transform.hash = (!hashes.is_empty()).then(|| HashTransform::new(hashes));
        transform.validate().map_err(|error| {
            invalid_config(format!(
                "Invalid attribute transform configuration: {error}"
            ))
        })?;

        let mut values = Vec::new();
        let mut has_literals = false;
        let mut split = |sources: BTreeMap<&str, &AttributeValueConfig>|
         -> Result<BTreeMap<String, LiteralValue>, ConfigError> {
            let mut literals = BTreeMap::new();
            for (key, action) in sources {
                let value = match (&action.value, &action.context_value) {
                    (Some(value), None) => {
                        has_literals = true;
                        value.to_literal(action.literal_type)?
                    }
                    (None, Some(source)) if action.literal_type.is_none() => {
                        let SourceBindings::Node(bindings) = bindings else {
                            continue;
                        };
                        values.push(DynamicValue {
                            key: key.to_owned(),
                            source: SourceReader::bind(source, bindings)?,
                        });
                        // Preserve every configured key for validation; resolved values replace this.
                        LiteralValue::Int(0)
                    }
                    _ => {
                        return Err(invalid_config(
                            "attribute action requires exactly one of value or context_value",
                        ));
                    }
                };
                _ = literals.insert(key.to_owned(), value);
            }
            Ok(literals)
        };
        let literals = values_transform(split(inserts)?, split(upserts)?, split(updates)?);
        transform.insert = literals.insert;
        transform.upsert = literals.upsert;
        transform.update = literals.update;
        values.sort_unstable_by(|left, right| left.key.cmp(&right.key));
        Ok(Self {
            transform,
            values: values.into_boxed_slice(),
            has_literals,
        })
    }

    pub(super) fn resolve(
        &self,
        metadata: &Metadata,
    ) -> Result<ResolvedTransform<'_>, EngineError> {
        Ok(ResolvedTransform {
            transform: &self.transform,
            reads: &self.values,
            values: self
                .values
                .iter()
                .map(|value| value.source.resolve(metadata))
                .collect::<Result<_, _>>()?,
            has_literals: self.has_literals,
        })
    }
}
