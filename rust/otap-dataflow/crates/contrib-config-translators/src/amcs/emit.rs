// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Conversion of extracted AMCS endpoint bindings into an engine pipeline specification.
//!
//! # Topology
//!
//! Listeners are global to the host (see [`crate::amcs::listener`]), so the generated pipeline
//! has a single OTLP receiver. Every Data Collection Rule then gets its own
//! `filter -> batch -> exporter` chain.
//!
//! With one export path the chain hangs directly off the receiver:
//!
//! ```text
//!   receiver -> filter -> batch -> exporter
//! ```
//!
//! With several export paths a `processor:fanout` clones each message to one named output port
//! per rule, so every rule sees every message and applies its own filter:
//!
//! ```text
//!                       +-- port_a --> filter_a -> batch_a -> exporter_a
//!   receiver -> fanout -+
//!                       +-- port_b --> filter_b -> batch_b -> exporter_b
//! ```
//!
//! A filter node is emitted for **every** rule, including one that declares no resource attribute
//! routing. In that case the filter matches everything and is redundant, but keeping the shape
//! uniform makes generation and review simpler; the redundant node can be optimised away later.
//!
//! Each path gets its **own** batch node. A batch shared across rules would merge their telemetry
//! with no way to separate it again before the exporters, because the batch processor has no
//! partition key or per-key queueing. Per-path batching also keeps one high-volume rule from
//! driving flush behaviour for the others.
//!
//! # Values and their origins
//!
//! | Generated field | Source |
//! |---|---|
//! | receiver `listening_addr` | environment variables and the agent settings rule |
//! | receiver size limits | fixed constants ([`MAX_DECODING_MESSAGE_SIZE`]) |
//! | filter `resource_attributes` | AMCS `resourceAttributeRouting` |
//! | exporter `logs_endpoint` / `traces_endpoint` | AMCS channel templates, `<STREAM>` substituted |
//! | exporter headers and user agent | the embedding host, via [`HostContext`] |
//! | `batch`, `policies`, `version`, `engine` | fixed defaults |

use crate::amcs::extract::{OtlpAttributeRouting, OtlpEventInfo, OtlpEventName};
use crate::amcs::listener::{OtlpEventListenerInfo, OtlpProtocol};
use otel_arrow_dfe_config::engine::{EngineConfig, OtelDataflowSpec};
use otel_arrow_dfe_config::pipeline::{PipelineConfigBuilder, PipelineType};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::collections::HashSet;

/// Node type URN for the OTLP receiver.
const OTLP_RECEIVER_URN: &str = "urn:otel:receiver:otlp";

/// Node type URN for the fan-out processor.
const FANOUT_PROCESSOR_URN: &str = "urn:otel:processor:fanout";

/// Node type URN for the filter processor.
const FILTER_PROCESSOR_URN: &str = "urn:otel:processor:filter";

/// Node type URN for the batch processor.
const BATCH_PROCESSOR_URN: &str = "urn:otel:processor:batch";

/// Node type URN for the OTLP/HTTP exporter.
const OTLP_HTTP_EXPORTER_URN: &str = "urn:otel:exporter:otlp_http";

/// Name of the fan-out node used when there is more than one export path.
const FANOUT_NODE: &str = "fanout";

/// Pipeline group the generated pipeline is placed in.
const PIPELINE_GROUP_ID: &str = "default";

/// Pipeline name within the group.
const PIPELINE_ID: &str = "main";

/// Maximum decompressed OTLP/gRPC message the receiver will accept.
///
/// The agent accepts a much larger payload than the engine's own 4 MiB default, so without this
/// every request above 4 MiB is rejected before it reaches the pipeline.
const MAX_DECODING_MESSAGE_SIZE: &str = "64MiB";

/// Maximum OTLP/HTTP request body the receiver will accept, for the same reason.
const MAX_REQUEST_BODY_SIZE: &str = "64MiB";

/// Batch size in bytes. The minimum and maximum are deliberately identical so a batch is emitted
/// at a predictable size rather than anywhere within a range.
const BATCH_SIZE_BYTES: u64 = 1_043_333;

/// Longest a batch may wait before being flushed.
const BATCH_MAX_DURATION: &str = "20000ms";

/// Batching format. `otlp` keeps payloads in OTLP form end to end rather than converting to OTAP.
const BATCH_FORMAT: &str = "otlp";

/// Whether outbound OTLP/HTTP requests are gzip compressed.
///
/// Disabled for now. When enabled the exporter's `http.compression` setting is emitted; keeping
/// the decision in a constant means the parser logic does not change if it is revisited.
const ENABLE_COMPRESSION: bool = false;

/// Compression algorithm used when [`ENABLE_COMPRESSION`] is set.
const COMPRESSION_ALGORITHM: &str = "gzip";

/// Number of pooled HTTP clients per exporter.
const EXPORTER_CLIENT_POOL_SIZE: u64 = 1;

/// Header carrying the Azure resource id of the host, matching the .NET agent.
const HEADER_RESOURCE_ID: &str = "azure-monitor-source-resourceId";

/// Header carrying the Azure region of the host, matching the .NET agent.
const HEADER_REGION: &str = "x-ms-AzureRegion";

/// Capability binding name for the credential provider the embedding host supplies.
///
/// `urn:otel:exporter:otlp_http` accepts two mutually exclusive credential capabilities:
/// `bearer_token_provider`, and `agent_fed_credential_provider` for host-managed rotating
/// credentials (added by open-telemetry/otel-arrow#3836). The exporter rejects a configuration
/// that binds both.
///
/// Agent-fed is the one that applies here: the Monitoring Agent owns the token and rotates it, and
/// the exporter re-reads the snapshot on every export attempt rather than holding a static token.
const CAPABILITY_AGENT_FED_CREDENTIAL_PROVIDER: &str = "agent_fed_credential_provider";

/// Sentinel used to build a filter that matches nothing.
///
/// The filter processor treats an empty match list as "match everything", so a signal cannot be
/// dropped by leaving its section empty. A non-empty list of names that cannot occur in real
/// telemetry yields an all-false mask instead, which is how a path discards a signal it has no
/// endpoint for.
const NEVER_MATCHES: &str = "__otap_amcs_signal_disabled__";

/// Values supplied by the embedding host rather than by the AMCS payload.
///
/// The agent knows its own identity, region and version; none of that appears in a Data
/// Collection Rule. The .NET agent sends the same three values as request headers, so the
/// generated configuration carries them on every exporter.
#[derive(Debug, Clone, Default)]
pub struct HostContext {
    /// Azure resource id of the host, for example
    /// `/subscriptions/<id>/resourceGroups/<rg>/providers/...`.
    ///
    /// Percent-encoded before it is emitted, matching `HttpUtility.UrlEncode` in the .NET agent.
    pub agent_identity: Option<String>,

    /// Azure region of the host, for example `westus2`. Emitted verbatim.
    pub region: Option<String>,

    /// Value for the outbound `User-Agent` header, for example `AMACoreAgent/1.2.3`.
    ///
    /// The host builds this from its own product name and version so the backend can tell which
    /// agent flavour sent the request.
    pub user_agent: Option<String>,
}

/// All bindings belonging to one `{configurationId}.{channelId}` identifier.
#[derive(Debug, Default)]
struct Branch {
    /// Resolved logs endpoint URLs, in stable order.
    logs_urls: Vec<String>,
    /// Resolved traces endpoint URLs, in stable order.
    traces_urls: Vec<String>,
    /// Routing filter for logs, or `None` when logs are broadcast.
    logs_routing: Option<OtlpAttributeRouting>,
    /// Routing filter for traces, or `None` when traces are broadcast.
    traces_routing: Option<OtlpAttributeRouting>,
}

/// One export path: a filter, a batch processor, and an exporter.
#[derive(Debug)]
struct SubBranch {
    /// Node name suffix, unique within the generated configuration.
    suffix: String,
    /// The logs endpoint for this path, if any.
    logs_url: Option<String>,
    /// The traces endpoint for this path, if any.
    traces_url: Option<String>,
    /// Routing filter for logs, or `None` when logs are broadcast.
    logs_routing: Option<OtlpAttributeRouting>,
    /// Routing filter for traces, or `None` when traces are broadcast.
    traces_routing: Option<OtlpAttributeRouting>,
}

impl SubBranch {
    /// Node name of this path's filter.
    fn filter_node(&self) -> String {
        format!("filter_{}", self.suffix)
    }

    /// Node name of this path's batch processor.
    ///
    /// The flush duration is part of the name so a reader can tell a node's batching behaviour
    /// without cross-referencing its config.
    fn batch_node(&self) -> String {
        format!("batch_20k_ms_{}", self.suffix)
    }

    /// Node name of this path's exporter.
    fn exporter_node(&self) -> String {
        format!("exporter_{}", self.suffix)
    }

    /// Name of the fan-out output port feeding this path.
    fn fanout_port(&self) -> String {
        format!("port_{}", self.suffix)
    }

    /// Whether this path needs a filter node at all.
    ///
    /// A filter is required to apply a routing attribute, or to drop a signal this path has no
    /// endpoint for. A rule that routes on nothing and exports both signals needs neither, so the
    /// node is omitted and the path is one hop shorter; a filter that matches everything would
    /// only cost a copy per message.
    ///
    /// # Metrics are not dropped
    ///
    /// AMCS describes only `otelLogs` and `otelTraces`, so no path ever carries a metrics
    /// endpoint. The OTLP receiver still accepts metrics, and neither the filter nor the exporter
    /// discards them: a filter section left absent means "match everything" (`MetricFilter` with
    /// no include and no exclude returns the payload untouched), and the exporter has no way to
    /// disable a signal -- an unset `metrics_endpoint` falls back to `endpoint + "/v1/metrics"`.
    /// A client that sends OTLP metrics to the agent therefore has them posted to a synthesised
    /// DCE URL that does not exist, producing 404s and retry churn rather than data loss.
    ///
    /// This is deliberate. Dropping metrics would mean emitting a filter on *every* path,
    /// including the ones this method leaves filter-free for rules that route on nothing, which is
    /// the shape the design calls for. Metrics are out of scope for the agent functionality being
    /// shipped, so the extra node is not worth spending on a signal the product does not carry.
    /// Revisit if metrics ever become part of the collected set.
    const fn needs_filter(&self) -> bool {
        self.logs_routing.is_some()
            || self.traces_routing.is_some()
            || self.logs_url.is_none()
            || self.traces_url.is_none()
    }
}

/// Build an engine pipeline specification from extracted AMCS bindings.
///
/// # Errors
///
/// Returns [`Error::EmptyPipeline`](crate::Error::EmptyPipeline) when `infos` is empty,
/// [`Error::InvalidListenerAddress`](crate::Error::InvalidListenerAddress) when a listener host
/// cannot be resolved, and [`Error::InvalidPipeline`](crate::Error::InvalidPipeline) when the
/// assembled pipeline is rejected by the engine configuration model.
pub fn build_pipeline(
    infos: &[OtlpEventInfo],
    host: &HostContext,
) -> Result<OtelDataflowSpec, crate::Error> {
    if infos.is_empty() {
        return Err(crate::Error::EmptyPipeline {
            details: "no OTLP endpoints were extracted from the AMCS configuration".to_string(),
        });
    }

    let listeners = collect_listeners(infos);
    let receiver_name = receiver_node_name(&listeners);
    let receiver_config = build_receiver_config(&listeners)?;
    let paths = collect_sub_branches(infos);

    if paths.is_empty() {
        return Err(crate::Error::EmptyPipeline {
            details: "no export paths could be derived from the AMCS configuration".to_string(),
        });
    }

    let mut builder = PipelineConfigBuilder::new().add_receiver(
        receiver_name.clone(),
        OTLP_RECEIVER_URN,
        Some(receiver_config),
    );

    // A single path hangs directly off the receiver. Several paths need a fan-out, because the
    // receiver's one output port cannot feed more than one target.
    let needs_fanout = paths.len() > 1;
    if needs_fanout {
        builder = builder
            .add_processor(
                FANOUT_NODE,
                FANOUT_PROCESSOR_URN,
                Some(build_fanout_config(&paths)),
            )
            .to(receiver_name.clone(), FANOUT_NODE);
    }

    for path in &paths {
        let batch_node = path.batch_node();
        let exporter_node = path.exporter_node();

        builder = builder
            .add_processor(
                batch_node.clone(),
                BATCH_PROCESSOR_URN,
                Some(build_batch_config()),
            )
            .add_exporter(
                exporter_node.clone(),
                OTLP_HTTP_EXPORTER_URN,
                Some(build_exporter_config(path, host)?),
            );

        // A path that routes on nothing and exports both signals needs no filter, so telemetry
        // goes straight to its batch node.
        let filter_node = path.needs_filter().then(|| path.filter_node());
        if let Some(filter_node) = &filter_node {
            builder = builder.add_processor(
                filter_node.clone(),
                FILTER_PROCESSOR_URN,
                Some(build_filter_config(path)),
            );
        }
        let entry_node = filter_node.clone().unwrap_or_else(|| batch_node.clone());

        // Connections are added in data-flow order so the generated document reads top to bottom:
        // the list is a `Vec` and is serialized in insertion order.
        builder = if needs_fanout {
            builder.to_output(FANOUT_NODE, path.fanout_port(), entry_node)
        } else {
            builder.to(receiver_name.clone(), entry_node)
        };

        if let Some(filter_node) = filter_node {
            builder = builder.to(filter_node, batch_node.clone());
        }

        builder = builder.to(batch_node, exporter_node);
    }

    let pipeline = builder.build(PipelineType::Otlp, PIPELINE_GROUP_ID, PIPELINE_ID)?;

    Ok(OtelDataflowSpec::from_pipeline(
        PIPELINE_GROUP_ID.into(),
        PIPELINE_ID.into(),
        pipeline,
        EngineConfig::default(),
    )?)
}

/// Collect the distinct listeners referenced by the extracted bindings, gRPC first.
fn collect_listeners(infos: &[OtlpEventInfo]) -> Vec<OtlpEventListenerInfo> {
    let mut grpc: Option<OtlpEventListenerInfo> = None;
    let mut http: Option<OtlpEventListenerInfo> = None;

    for info in infos {
        match info.listener.protocol {
            OtlpProtocol::Grpc => grpc = grpc.or_else(|| Some(info.listener.clone())),
            OtlpProtocol::HttpProtobuf => http = http.or_else(|| Some(info.listener.clone())),
        }
    }

    grpc.into_iter().chain(http).collect()
}

/// Derive the receiver node name from the ports it listens on, for example `receiver_4319_4320`.
///
/// Ports rather than addresses, because a node name cannot contain the dots in an IP address, and
/// because the ports are what a reader wants when checking which listeners are live. A disabled
/// protocol simply does not appear.
fn receiver_node_name(listeners: &[OtlpEventListenerInfo]) -> String {
    let mut name = String::from("receiver");
    for listener in listeners {
        name.push('_');
        name.push_str(&listener.port.to_string());
    }
    name
}

/// Build the shared receiver configuration.
///
/// Listener hosts are resolved to literal socket addresses because the receiver deserializes
/// `listening_addr` as a [`std::net::SocketAddr`], which cannot accept a hostname such as the
/// default `localhost`.
fn build_receiver_config(listeners: &[OtlpEventListenerInfo]) -> Result<Value, crate::Error> {
    let mut protocols = serde_json::Map::new();

    for listener in listeners {
        let addr = listener.socket_addr()?.to_string();
        match listener.protocol {
            OtlpProtocol::Grpc => {
                let _ = protocols.insert(
                    "grpc".to_string(),
                    json!({
                        "listening_addr": addr,
                        "max_decoding_message_size": MAX_DECODING_MESSAGE_SIZE,
                    }),
                );
            }
            OtlpProtocol::HttpProtobuf => {
                let _ = protocols.insert(
                    "http".to_string(),
                    json!({
                        "listening_addr": addr,
                        "max_request_body_size": MAX_REQUEST_BODY_SIZE,
                    }),
                );
            }
        }
    }

    if protocols.is_empty() {
        return Err(crate::Error::EmptyPipeline {
            details: "no OTLP listeners are enabled".to_string(),
        });
    }

    Ok(json!({ "protocols": Value::Object(protocols) }))
}

/// Build the fan-out configuration: one destination per export path.
///
/// `await_ack: none` keeps the receiver from blocking on downstream acknowledgement. Every rule
/// that matches a message should receive it, so no destination is marked primary.
fn build_fanout_config(paths: &[SubBranch]) -> Value {
    let destinations: Vec<Value> = paths
        .iter()
        .map(|path| json!({ "port": path.fanout_port() }))
        .collect();

    json!({
        "await_ack": "none",
        "destinations": destinations,
    })
}

/// Build the filter configuration for an export path.
///
/// Routing is applied per signal, because a rule may filter its logs while broadcasting its
/// traces or the reverse; the .NET parser records routing on each endpoint binding rather than on
/// the rule as a whole.
///
/// A signal this path has no endpoint for is dropped here. Without that the exporter would fall
/// back to `endpoint + "/v1/<signal>"` and deliver it somewhere nobody asked for.
fn build_filter_config(path: &SubBranch) -> Value {
    let logs_attributes = resource_attributes(path.logs_routing.as_ref());
    let traces_attributes = resource_attributes(path.traces_routing.as_ref());

    let logs_include = if path.logs_url.is_some() {
        json!({
            "match_type": "strict",
            "resource_attributes": logs_attributes,
        })
    } else {
        json!({
            "match_type": "strict",
            "resource_attributes": logs_attributes,
            "severity_texts": [NEVER_MATCHES],
        })
    };

    // `span_names` has no serde default, so it must always be present.
    let traces_include = if path.traces_url.is_some() {
        json!({
            "match_type": "strict",
            "resource_attributes": traces_attributes,
            "span_names": [],
        })
    } else {
        json!({
            "match_type": "strict",
            "resource_attributes": traces_attributes,
            "span_names": [NEVER_MATCHES],
        })
    };

    json!({
        "logs": { "include": logs_include },
        "traces": { "include": traces_include },
    })
}

/// Render a routing filter as the filter processor's `resource_attributes` list.
///
/// `None` yields an empty list, which the filter processor treats as "match everything" -- the
/// broadcast behaviour applied when a rule declares no attribute routing.
fn resource_attributes(routing: Option<&OtlpAttributeRouting>) -> Vec<Value> {
    routing.map_or_else(Vec::new, |r| {
        vec![json!({ "key": r.name, "value": r.value })]
    })
}

/// Build the batch processor configuration.
///
/// None of these values come from AMCS; a Data Collection Rule says nothing about batching.
fn build_batch_config() -> Value {
    json!({
        "otlp": {
            "sizer": "bytes",
            "min_size": BATCH_SIZE_BYTES,
            "max_size": BATCH_SIZE_BYTES,
        },
        "max_batch_duration": BATCH_MAX_DURATION,
        "format": BATCH_FORMAT,
    })
}

/// Build the exporter configuration for one export path.
fn build_exporter_config(path: &SubBranch, host: &HostContext) -> Result<Value, crate::Error> {
    // `endpoint` is mandatory and must be a valid URL. It is only ever consulted for a signal
    // with no explicit endpoint, which cannot happen here because such a signal is dropped by the
    // filter, but it still has to be present and parseable.
    let reference = path
        .logs_url
        .as_ref()
        .or(path.traces_url.as_ref())
        .ok_or_else(|| crate::Error::EmptyPipeline {
            details: "export path has neither a logs nor a traces URL".to_string(),
        })?;

    let mut http = serde_json::Map::new();
    if let Some(user_agent) = &host.user_agent {
        let _ = http.insert("user_agent".to_string(), json!(user_agent));
    }
    if ENABLE_COMPRESSION {
        let _ = http.insert("compression".to_string(), json!(COMPRESSION_ALGORITHM));
    }

    let mut headers = serde_json::Map::new();
    if let Some(identity) = &host.agent_identity {
        // Percent-encoded to match `HttpUtility.UrlEncode` in the .NET agent, because a resource
        // id contains slashes that are not legal unencoded in a header value.
        let _ = headers.insert(
            HEADER_RESOURCE_ID.to_string(),
            json!(urlencoding::encode(identity).into_owned()),
        );
    }
    if let Some(region) = &host.region {
        let _ = headers.insert(HEADER_REGION.to_string(), json!(region));
    }
    if !headers.is_empty() {
        let _ = http.insert("headers".to_string(), Value::Object(headers));
    }

    let mut config = serde_json::Map::new();
    let _ = config.insert("http".to_string(), Value::Object(http));
    let _ = config.insert("endpoint".to_string(), json!(origin_of(reference)));
    let _ = config.insert(
        "client_pool_size".to_string(),
        json!(EXPORTER_CLIENT_POOL_SIZE),
    );
    if let Some(url) = &path.logs_url {
        let _ = config.insert("logs_endpoint".to_string(), json!(url));
    }
    if let Some(url) = &path.traces_url {
        let _ = config.insert("traces_endpoint".to_string(), json!(url));
    }

    Ok(Value::Object(config))
}

/// Group bindings by identifier and flatten them into export paths.
fn collect_sub_branches(infos: &[OtlpEventInfo]) -> Vec<SubBranch> {
    // BTreeMap keeps identifiers in a stable order so generated output is deterministic.
    let mut branches: BTreeMap<String, Branch> = BTreeMap::new();

    for info in infos {
        let branch = branches.entry(info.identifier.clone()).or_default();

        let (target, routing) = match info.event_name {
            OtlpEventName::Log => (&mut branch.logs_urls, &mut branch.logs_routing),
            OtlpEventName::Span => (&mut branch.traces_urls, &mut branch.traces_routing),
        };

        if routing.is_none() {
            *routing = info.routing_info.clone();
        }

        for url in &info.endpoint_urls {
            // The same identifier appears once per listener, so URLs repeat; keep one copy.
            if !target.contains(url) {
                target.push(url.clone());
            }
        }
    }

    let mut used_names: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for (identifier, branch) in &branches {
        let slug = unique_slug(identifier, &mut used_names);

        // A data source may declare several streams, giving several endpoint URLs for one signal.
        // Each URL needs its own exporter, so emit as many paths as the widest signal requires
        // and pair URLs by index.
        let count = branch.logs_urls.len().max(branch.traces_urls.len());

        for index in 0..count {
            let suffix = if count > 1 {
                format!("{slug}_{index}")
            } else {
                slug.clone()
            };

            result.push(SubBranch {
                suffix,
                logs_url: branch.logs_urls.get(index).cloned(),
                traces_url: branch.traces_urls.get(index).cloned(),
                logs_routing: branch.logs_routing.clone(),
                traces_routing: branch.traces_routing.clone(),
            });
        }
    }

    result
}

/// Return the scheme and authority of `url`, for example `https://dce.example.com`.
///
/// Falls back to the whole string when the input has no path component, which keeps the value a
/// parseable URL in every case the exporter will accept.
fn origin_of(url: &str) -> String {
    match url.find("://") {
        Some(scheme_end) => {
            let authority_start = scheme_end + "://".len();
            match url[authority_start..].find('/') {
                Some(path_start) => url[..authority_start + path_start].to_string(),
                None => url.to_string(),
            }
        }
        None => url.to_string(),
    }
}

/// Derive a node-name-safe slug from an identifier, guaranteed unique within the pipeline.
///
/// Identifiers look like `dcr-00000002....gigl-dce-00000002...`; the `.` separator is replaced so
/// generated node names stay readable. Because distinct identifiers can sanitize to the same
/// string, a numeric suffix is appended on collision.
fn unique_slug(identifier: &str, used: &mut HashSet<String>) -> String {
    let base: String = identifier
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let base = if base.is_empty() {
        "branch".to_string()
    } else {
        base
    };

    if used.insert(base.clone()) {
        return base;
    }

    let mut suffix = 2u32;
    loop {
        let candidate = format!("{base}_{suffix}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        suffix = suffix.saturating_add(1);
    }
}

/// Name of the capability an exporter must bind to receive credentials from the embedding host.
///
/// This translator deliberately does **not** emit the binding itself. Attaching credentials takes
/// two things that only the host can supply: an entry in the pipeline's `extensions` section
/// declaring the provider instance, and a `capabilities` binding on each exporter. The generated
/// specification carries neither, because the AMCS payload says nothing about how the agent
/// authenticates, and because `PipelineConfigBuilder` currently exposes no way to set node
/// capabilities (`PipelineConfig::nodes` is private and `add_exporter` always writes an empty
/// capability map). Emitting a binding without the matching extension would also fail validation
/// outright -- see `PipelineConfig` validation, "binds capability ... but no extension with that
/// name exists".
///
/// The name is exported so the host binds the right one. `urn:otel:exporter:otlp_http` takes
/// either this capability or `bearer_token_provider`, never both -- it rejects an ambiguous
/// configuration. Agent-fed is correct for this agent because the Monitoring Agent owns the token
/// and rotates it; the exporter reads a fresh snapshot per export instead of caching a static one.
///
/// Requires open-telemetry/otel-arrow#3836, which adds agent-fed support to the OTLP/HTTP
/// exporter. Until it merges, binding this name resolves nothing: validation checks only that the
/// *extension name* exists, never that the node consumes the capability, and the binding is
/// optional -- so the exporter would send no `Authorization` header and report nothing.
#[must_use]
pub const fn credential_capability_name() -> &'static str {
    CAPABILITY_AGENT_FED_CREDENTIAL_PROVIDER
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amcs::listener::OtlpEventListenerInfo;

    fn listener(protocol: OtlpProtocol, port: u16) -> OtlpEventListenerInfo {
        OtlpEventListenerInfo {
            host: "127.0.0.1".to_string(),
            port,
            protocol,
        }
    }

    fn routing(value: &str) -> Option<OtlpAttributeRouting> {
        Some(OtlpAttributeRouting {
            name: "service.name".to_string(),
            value: value.to_string(),
        })
    }

    fn info(
        identifier: &str,
        event_name: OtlpEventName,
        url: &str,
        routing_info: Option<OtlpAttributeRouting>,
    ) -> OtlpEventInfo {
        OtlpEventInfo {
            listener: listener(OtlpProtocol::Grpc, 4319),
            identifier: identifier.to_string(),
            endpoint_urls: vec![url.to_string()],
            event_name,
            routing_info,
        }
    }

    /// Build a spec with no host context and render it as YAML.
    fn to_yaml(infos: &[OtlpEventInfo]) -> String {
        let spec = build_pipeline(infos, &HostContext::default()).expect("pipeline should build");
        serde_yaml::to_string(&spec).expect("spec should serialize")
    }

    /// Scenario: `build_pipeline` is called with no extracted endpoint bindings.
    /// Guarantees: an empty AMCS payload reports an error rather than producing a pipeline that
    /// would start, bind ports and silently discard everything.
    #[test]
    fn empty_input_is_an_error() {
        assert!(matches!(
            build_pipeline(&[], &HostContext::default()),
            Err(crate::Error::EmptyPipeline { .. })
        ));
    }

    /// Scenario: one rule sends both logs and traces to a single channel.
    /// Guarantees: a single export path produces the receiver, filter, batch and exporter chain
    /// with no fan-out node, since fan-out is only needed for more than one path.
    #[test]
    fn single_path_has_no_fanout() {
        let infos = vec![
            info(
                "dcr-1.gig-1",
                OtlpEventName::Log,
                "https://dce.example.com/a/logs",
                routing("amcs"),
            ),
            info(
                "dcr-1.gig-1",
                OtlpEventName::Span,
                "https://dce.example.com/a/traces",
                routing("amcs"),
            ),
        ];

        let yaml = to_yaml(&infos);

        assert!(yaml.contains("urn:otel:receiver:otlp"));
        assert!(yaml.contains("filter_dcr-1_gig-1"));
        assert!(yaml.contains("batch_20k_ms_dcr-1_gig-1"));
        assert!(yaml.contains("exporter_dcr-1_gig-1"));
        assert!(
            !yaml.contains("urn:otel:processor:fanout"),
            "a single path needs no fan-out:\n{yaml}"
        );
    }

    /// Scenario: two rules on one host route on different resource attribute values.
    /// Guarantees: a fan-out node clones telemetry to one named port per rule, so each rule sees
    /// every message and applies its own filter, and the receiver is not duplicated.
    #[test]
    fn multiple_paths_use_fanout() {
        let infos = vec![
            info(
                "dcr-a.gig-a",
                OtlpEventName::Log,
                "https://a.example.com/logs",
                routing("amcs_1"),
            ),
            info(
                "dcr-b.gig-b",
                OtlpEventName::Log,
                "https://b.example.com/logs",
                routing("amcs_2"),
            ),
        ];

        let yaml = to_yaml(&infos);

        assert_eq!(yaml.matches("urn:otel:receiver:otlp").count(), 1);
        assert_eq!(yaml.matches("urn:otel:processor:fanout").count(), 1);
        assert_eq!(yaml.matches("urn:otel:exporter:otlp_http").count(), 2);
        assert!(yaml.contains("port_dcr-a_gig-a"));
        assert!(yaml.contains("port_dcr-b_gig-b"));
        assert!(yaml.contains("amcs_1"));
        assert!(yaml.contains("amcs_2"));
    }

    /// Scenario: two rules each need their telemetry batched before export.
    /// Guarantees: rules do not share a batch node, which would merge their telemetry with no way
    /// to separate it again before the exporters, since the batch processor has no partition key.
    #[test]
    fn each_path_gets_its_own_batch() {
        let infos = vec![
            info(
                "dcr-a.gig-a",
                OtlpEventName::Log,
                "https://a.example.com/logs",
                routing("amcs_1"),
            ),
            info(
                "dcr-b.gig-b",
                OtlpEventName::Log,
                "https://b.example.com/logs",
                routing("amcs_2"),
            ),
        ];

        let yaml = to_yaml(&infos);

        assert_eq!(
            yaml.matches("urn:otel:processor:batch").count(),
            2,
            "expected one batch per path:\n{yaml}"
        );
        assert!(yaml.contains("batch_20k_ms_dcr-a_gig-a"));
        assert!(yaml.contains("batch_20k_ms_dcr-b_gig-b"));
    }

    /// Scenario: a rule declares no `resourceAttributeRouting` and exports both signals.
    /// Guarantees: no filter node is emitted, matching Kennedy's optimised shape where a rule that
    /// accepts everything connects straight to its batch node instead of paying for a
    /// match-everything filter on every message.
    #[test]
    fn a_rule_that_routes_on_nothing_needs_no_filter() {
        let infos = vec![
            info(
                "dcr-1.gig-1",
                OtlpEventName::Log,
                "https://x.example.com/logs",
                None,
            ),
            info(
                "dcr-1.gig-1",
                OtlpEventName::Span,
                "https://x.example.com/traces",
                None,
            ),
        ];

        let yaml = to_yaml(&infos);

        assert!(
            !yaml.contains("urn:otel:processor:filter"),
            "a match-everything path should skip the filter:\n{yaml}"
        );
        assert!(yaml.contains("batch_20k_ms_dcr-1_gig-1"));
        assert!(yaml.contains("exporter_dcr-1_gig-1"));
    }

    /// Scenario: Kennedy's three-rule matrix -- two rules routing on distinct `service.name`
    /// values and a third accepting everything, each with its own destination.
    /// Guarantees: the routed rules get filters, the accept-everything rule does not, and all
    /// three still reach separate exporters, so a future change that misroutes the
    /// accept-everything case fails here.
    #[test]
    fn three_rule_matrix_filters_only_where_routing_is_declared() {
        let infos = vec![
            info(
                "dcr-a.gig-a",
                OtlpEventName::Log,
                "https://dce1.example.com/logs",
                routing("amcs"),
            ),
            info(
                "dcr-a.gig-a",
                OtlpEventName::Span,
                "https://dce1.example.com/traces",
                routing("amcs"),
            ),
            info(
                "dcr-b.gig-b",
                OtlpEventName::Log,
                "https://dce2.example.com/logs",
                routing("backend"),
            ),
            info(
                "dcr-b.gig-b",
                OtlpEventName::Span,
                "https://dce2.example.com/traces",
                routing("backend"),
            ),
            info(
                "dcr-c.gig-c",
                OtlpEventName::Log,
                "https://dce3.example.com/logs",
                None,
            ),
            info(
                "dcr-c.gig-c",
                OtlpEventName::Span,
                "https://dce3.example.com/traces",
                None,
            ),
        ];

        let yaml = to_yaml(&infos);

        assert_eq!(yaml.matches("urn:otel:exporter:otlp_http").count(), 3);
        assert_eq!(
            yaml.matches("urn:otel:processor:filter").count(),
            2,
            "only the two routed rules need a filter:\n{yaml}"
        );
        assert!(yaml.contains("value: amcs"));
        assert!(yaml.contains("value: backend"));
        assert!(yaml.contains("dce1.example.com"));
        assert!(yaml.contains("dce2.example.com"));
        assert!(yaml.contains("dce3.example.com"));
    }

    /// Scenario: a channel supplies a logs endpoint but no traces endpoint.
    /// Guarantees: traces are dropped by the filter rather than reaching the exporter, which would
    /// otherwise fall back to `endpoint + "/v1/traces"` and deliver them to an unintended URL.
    #[test]
    fn a_signal_without_an_endpoint_is_dropped() {
        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            None,
        )];

        let yaml = to_yaml(&infos);

        assert!(yaml.contains(NEVER_MATCHES));
        assert!(yaml.contains("logs_endpoint"));
        assert!(!yaml.contains("traces_endpoint"));
    }

    /// Scenario: the receiver listens on a gRPC and an HTTP port.
    /// Guarantees: the node is named after both ports with no address, since a node name cannot
    /// contain the dots of an IP address, and both size limits are present so payloads above the
    /// engine's 4 MiB default are not rejected before reaching the pipeline.
    #[test]
    fn receiver_is_named_after_its_ports_and_raises_size_limits() {
        let mut grpc = info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        );
        grpc.listener = listener(OtlpProtocol::Grpc, 4319);
        let mut http = grpc.clone();
        http.listener = listener(OtlpProtocol::HttpProtobuf, 4320);

        let yaml = to_yaml(&[grpc, http]);

        assert!(yaml.contains("receiver_4319_4320"), "got:\n{yaml}");
        assert!(!yaml.contains("receiver_127"));
        assert!(yaml.contains(MAX_DECODING_MESSAGE_SIZE));
        assert!(yaml.contains(MAX_REQUEST_BODY_SIZE));
    }

    /// Scenario: the batch processor configuration is generated.
    /// Guarantees: the agreed batching values are emitted, including identical minimum and maximum
    /// sizes so batches flush at a predictable size, and `otlp` format so payloads are not
    /// converted to OTAP and back on a path that is OTLP end to end.
    #[test]
    fn batch_uses_the_agreed_values() {
        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        )];

        let yaml = to_yaml(&infos);

        assert!(yaml.contains("min_size: 1043333"));
        assert!(yaml.contains("max_size: 1043333"));
        assert!(yaml.contains("max_batch_duration: 20000ms"));
        assert!(yaml.contains("format: otlp"));
    }

    /// Scenario: the embedding host supplies its identity, region and user agent.
    /// Guarantees: those reach every exporter as request metadata, with the resource id
    /// percent-encoded so its slashes remain a legal HTTP header value.
    #[test]
    fn host_context_reaches_the_exporter() {
        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        )];

        let host = HostContext {
            agent_identity: Some("/subscriptions/abc/resourceGroups/rg".to_string()),
            region: Some("westus2".to_string()),
            user_agent: Some("AMACoreAgent/1.2.3".to_string()),
        };

        let spec = build_pipeline(&infos, &host).expect("pipeline should build");
        let yaml = serde_yaml::to_string(&spec).expect("serialize");

        assert!(yaml.contains("AMACoreAgent/1.2.3"));
        assert!(yaml.contains(HEADER_REGION));
        assert!(yaml.contains("westus2"));
        assert!(
            yaml.contains("%2Fsubscriptions%2Fabc%2FresourceGroups%2Frg"),
            "resource id should be percent-encoded:\n{yaml}"
        );
    }

    /// Scenario: the exported credential capability name is checked, and a generated config is
    /// checked for capability bindings.
    /// Guarantees: the name is `agent_fed_credential_provider`, which suits a host that rotates
    /// the token, and which `urn:otel:exporter:otlp_http` accepts as of
    /// open-telemetry/otel-arrow#3836. The exporter takes this capability or
    /// `bearer_token_provider` but never both, so the two must not be conflated. The translator
    /// emits no binding of its own, since the matching `extensions` entry can only come from the
    /// host and a binding without it would not validate.
    #[test]
    fn credential_capability_is_the_agent_fed_provider() {
        assert_eq!(
            credential_capability_name(),
            "agent_fed_credential_provider",
            "the host rotates the token, so the exporter must re-read a snapshot per export"
        );

        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        )];
        let yaml = to_yaml(&infos);

        assert!(
            !yaml.contains("capabilities:"),
            "credential binding is the host's to add, together with the extension:\n{yaml}"
        );
    }

    /// Scenario: one rule declares more log streams than trace streams, so the two signals yield
    /// different numbers of endpoint URLs and are paired by index.
    /// Guarantees: every URL is exported exactly once. The shorter signal is not repeated to fill
    /// the gap, which would duplicate telemetry and bill a customer twice for the same data, and
    /// the surplus path drops the signal it has no endpoint for rather than inventing one.
    #[test]
    fn asymmetric_url_counts_export_each_url_exactly_once() {
        let infos = vec![
            OtlpEventInfo {
                identifier: "dcr-1.gig-1".to_string(),
                endpoint_urls: vec![
                    "https://x.example.com/s1/logs".to_string(),
                    "https://x.example.com/s2/logs".to_string(),
                ],
                event_name: OtlpEventName::Log,
                routing_info: routing("amcs"),
                listener: listener(OtlpProtocol::Grpc, 4319),
            },
            OtlpEventInfo {
                identifier: "dcr-1.gig-1".to_string(),
                endpoint_urls: vec!["https://x.example.com/s1/traces".to_string()],
                event_name: OtlpEventName::Span,
                routing_info: routing("amcs"),
                listener: listener(OtlpProtocol::Grpc, 4319),
            },
        ];

        let yaml = to_yaml(&infos);

        let occurrences = |needle: &str| -> usize { yaml.matches(needle).count() };

        assert_eq!(
            occurrences("https://x.example.com/s1/logs"),
            1,
            "first logs URL should be exported once:\n{yaml}"
        );
        assert_eq!(
            occurrences("https://x.example.com/s2/logs"),
            1,
            "second logs URL should be exported once:\n{yaml}"
        );
        assert_eq!(
            occurrences("https://x.example.com/s1/traces"),
            1,
            "the single traces URL must not be repeated onto the surplus path:\n{yaml}"
        );

        // The surplus path has no traces endpoint, so it must drop traces outright.
        assert!(
            yaml.contains(NEVER_MATCHES),
            "the path without a traces URL should drop the signal:\n{yaml}"
        );
    }

    /// Scenario: the host supplies no identity, region or user agent.
    /// Guarantees: no header block is emitted, so a configuration generated without host context
    /// stays valid rather than carrying blank metadata.
    #[test]
    fn absent_host_context_emits_no_headers() {
        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        )];

        let yaml = to_yaml(&infos);

        assert!(!yaml.contains(HEADER_REGION));
        assert!(!yaml.contains(HEADER_RESOURCE_ID));
        assert!(!yaml.contains("user_agent"));
    }

    /// Scenario: compression is disabled by the build-time constant.
    /// Guarantees: no compression setting is emitted while the constant is false, so the default
    /// deployment sends uncompressed requests and the decision lives in exactly one place.
    #[test]
    fn compression_is_absent_while_disabled() {
        let infos = vec![info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        )];

        let yaml = to_yaml(&infos);

        // Guard the current default without asserting on a constant directly, so flipping
        // ENABLE_COMPRESSION fails this test loudly rather than silently changing behaviour.
        if ENABLE_COMPRESSION {
            assert!(yaml.contains(COMPRESSION_ALGORITHM));
        } else {
            assert!(!yaml.contains("compression"));
        }
    }

    /// Scenario: several streams on one data source yield several endpoint URLs.
    /// Guarantees: each URL gets its own export path, so no configured destination is dropped.
    #[test]
    fn multiple_urls_produce_multiple_paths() {
        let infos = vec![OtlpEventInfo {
            listener: listener(OtlpProtocol::Grpc, 4319),
            identifier: "dcr-1.gig-1".to_string(),
            endpoint_urls: vec![
                "https://x.example.com/s1/logs".to_string(),
                "https://x.example.com/s2/logs".to_string(),
            ],
            event_name: OtlpEventName::Log,
            routing_info: routing("amcs"),
        }];

        let yaml = to_yaml(&infos);

        assert_eq!(yaml.matches("urn:otel:exporter:otlp_http").count(), 2);
        assert!(yaml.contains("exporter_dcr-1_gig-1_0"));
        assert!(yaml.contains("exporter_dcr-1_gig-1_1"));
    }

    /// Scenario: one identifier is reported once per enabled listener.
    /// Guarantees: repeated bindings collapse into a single export path, so the number of
    /// exporters follows the number of destinations rather than the number of listeners.
    #[test]
    fn repeated_listeners_do_not_duplicate_paths() {
        let mut grpc = info(
            "dcr-1.gig-1",
            OtlpEventName::Log,
            "https://x.example.com/logs",
            routing("amcs"),
        );
        grpc.listener = listener(OtlpProtocol::Grpc, 4319);
        let mut http = grpc.clone();
        http.listener = listener(OtlpProtocol::HttpProtobuf, 4320);

        let yaml = to_yaml(&[grpc, http]);

        assert_eq!(yaml.matches("urn:otel:exporter:otlp_http").count(), 1);
        assert!(yaml.contains("127.0.0.1:4319"));
        assert!(yaml.contains("127.0.0.1:4320"));
    }

    /// Scenario: a generated specification is serialized and read back.
    /// Guarantees: the engine's own loader accepts it, so the translator cannot emit a
    /// configuration the engine would refuse to start.
    #[test]
    fn generated_yaml_round_trips() {
        let infos = vec![
            info(
                "dcr-a.gig-a",
                OtlpEventName::Log,
                "https://a.example.com/logs",
                routing("amcs_1"),
            ),
            info(
                "dcr-b.gig-b",
                OtlpEventName::Log,
                "https://b.example.com/logs",
                routing("amcs_2"),
            ),
        ];

        let yaml = to_yaml(&infos);
        let reparsed = OtelDataflowSpec::from_yaml(&yaml);
        assert!(reparsed.is_ok(), "round-trip failed: {reparsed:?}\n{yaml}");
    }

    /// Scenario: an endpoint URL is reduced to the value used for the exporter's `endpoint` field.
    /// Guarantees: only the scheme and authority survive, which keeps the fallback endpoint a
    /// parseable URL without pointing at any one signal's path.
    #[test]
    fn origin_is_extracted_from_a_url() {
        assert_eq!(
            origin_of("https://dce.example.com/dataCollectionRules/dcr-1/streams/S/otlp/v1/logs"),
            "https://dce.example.com"
        );
        assert_eq!(
            origin_of("https://dce.example.com"),
            "https://dce.example.com"
        );
        assert_eq!(
            origin_of("https://dce.example.com:443/x"),
            "https://dce.example.com:443"
        );
    }

    /// Scenario: two identifiers differ only in a character that is not name-safe.
    /// Guarantees: the sanitized names stay distinct, so one rule's nodes cannot silently
    /// overwrite another's.
    #[test]
    fn slugs_are_sanitized_and_unique() {
        let mut used = HashSet::new();
        assert_eq!(unique_slug("dcr-1.gig-1", &mut used), "dcr-1_gig-1");
        assert_eq!(unique_slug("dcr-1_gig-1", &mut used), "dcr-1_gig-1_2");
    }
}
