// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use enrichment::invalid_config;

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
            Action::Insert { value, .. }
            | Action::Upsert { value, .. }
            | Action::Update { value, .. } => value.validate(),
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

#[derive(Clone, Copy)]
enum Operation {
    Insert,
    Upsert,
    Update,
}

struct DynamicAction {
    operation: Operation,
    key: String,
    value: ConfiguredValue,
}

pub(super) struct CompiledTransform {
    pub(super) transform: AttributesTransform,
    dynamic: Vec<DynamicAction>,
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
    pub(super) fn new(actions: &[ScopedAction], signal: SignalType) -> Result<Self, ConfigError> {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();
        let mut inserts = BTreeMap::new();
        let mut upserts = BTreeMap::new();
        let mut updates = BTreeMap::new();
        let mut hashes = BTreeMap::new();
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
                Action::Insert { key, value } => {
                    _ = inserts.insert(key.clone(), value.clone());
                }
                Action::Upsert { key, value } => {
                    _ = upserts.insert(key.clone(), value.clone());
                }
                Action::Update { key, value } => {
                    _ = updates.insert(key.clone(), value.clone());
                }
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
        let placeholder = |sources: &BTreeMap<String, ConfiguredValue>| {
            sources
                .keys()
                .map(|key| (key.clone(), LiteralValue::Int(0)))
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

        let mut dynamic = Vec::new();
        let mut split = |sources: BTreeMap<String, ConfiguredValue>, operation| {
            sources
                .into_iter()
                .filter_map(|(key, value)| {
                    if let Some(value) = value.literal() {
                        Some((key, value))
                    } else {
                        dynamic.push(DynamicAction {
                            operation,
                            key,
                            value,
                        });
                        None
                    }
                })
                .collect()
        };
        let values = values_transform(
            split(inserts, Operation::Insert),
            split(upserts, Operation::Upsert),
            split(updates, Operation::Update),
        );
        transform.insert = values.insert;
        transform.upsert = values.upsert;
        transform.update = values.update;
        Ok(Self { transform, dynamic })
    }

    pub(super) fn resolve(&self, context: &Context) -> Result<AttributesTransform, EngineError> {
        let mut inserts = BTreeMap::new();
        let mut upserts = BTreeMap::new();
        let mut updates = BTreeMap::new();
        for action in &self.dynamic {
            if let Some(value) = action.value.resolve(context)? {
                let destination = match action.operation {
                    Operation::Insert => &mut inserts,
                    Operation::Upsert => &mut upserts,
                    Operation::Update => &mut updates,
                };
                _ = destination.insert(action.key.clone(), value);
            }
        }
        Ok(values_transform(inserts, upserts, updates))
    }
}
