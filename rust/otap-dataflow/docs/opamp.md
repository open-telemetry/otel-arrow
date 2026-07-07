# OpAMP Controller Extension

This document describes the mechanism through which dataflow engine can be
configured via an embedded [OpAMP](https://opentelemetry.io/docs/specs/opamp/) 
Agent, implemented as a controller extension.

## Problem

The OpAMP protocol is used in the OpenTelemetry ecosystem for fleet management.
As members of the OpenTelemetry ecosystem, it is sensible to add the capability
to Otel Arrow Dataflow Engine to be configured using this management protocol.

## Background

### OpAMP

See [Full OpAMP documentation](https://opentelemetry.io/docs/specs/opamp/) for
more information.

OpAMP (Open Agent Management Protocol) is a vendor-agnostic network protocol
existing as part of the OpenTelemetry ecosystem. It specifies a mechanism for:

- _Servers_ to configure remote _Agents_
- Agents to report their status to the Server
- Management of Agent downloadable _packages_.
- Connection credential management (including client-side TLS)

Agent to Server communication typically happens over a network where the Agent
acts as a client, using either
[WebSocket](https://opentelemetry.io/docs/specs/opamp/#websocket-transport) or
[plain HTTP](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)
(plain meaning standard HTTP request-response model, not HTTP over plaintext).

The messages exchanged are protobuf serialized `AgentToServer` and
`ServerToAgent` messages.
Find [proto message definitions](https://github.com/open-telemetry/opamp-spec/blob/main/proto/opamp/v1/opamp.proto).
Note: When constructing/handling messages, language bindings generated from
these official OpAMP protos should be used when possible.

### Controller extension

OTel Controller Extensions allow custom implementation of logic that interacts
with the engine control API to reconcile configuration and report status. They
are configurable at runtime and can be linked at compile time (similar to
processors, exporters, receivers, etc.).

See [#3263](https://github.com/open-telemetry/otel-arrow/issues/3263) and
[#3281](https://github.com/open-telemetry/otel-arrow/issues/3281).

## Scope

This design document covers the implementation of an OpAMP Agent that receives
configuration from a remote server in `ServerToAgent` messages, and also sends
reports its health and status to the remote server in `AgentToServer` messages.

OpAMP has additional capabilities such as server directed management of
connection/credential settings and package management (which could be used
for dynamic loading of WASM components, for example). These considerations
are left as future design work.

## Design Principles

### Control Plane Implementation Agnostic

The goal of this implementation is simply to expose a mechanism through which
remote control plane authors can configure an instance of Dataflow Engine via
OpAMP, however the implementation of the server (how it resolves configuration,
identifies, DFE instances, manages state, etc.) is outside the scope of the
design and is left to the whim of its creator.

The goal is intentionally to solve the use case of being the interaction point
for some abstract, theoretical control plane. To wit, design points should
be agnostic to the control plane implementation while providing requisite
flexibility.

## Configuration

The URN used to identify the controller extension will be
`urn:otel:controller_extension:opamp` and it can be configured as follows:

```yaml
# example DFE config
engine:
  controller:
    extensions:
      opamp:
        type: urn:otel:controller_extension:opamp
        config:
          # The URL of the OpAMP server, including the scheme and path.
          #
          # The scheme will be used to determine whether communication
          # is done over WebSocket or HTTP. E.g. use "ws://..." for websocket
          # and http:// for http.
          #
          # Required
          endpoint: ws://127.0.0.1:4320/v1/opamp

          # mTLS configuration
          #
          # Optional: if not provided, messages will be sent as plaintext
          tls:
            # Trust settings for verifying the server certificate
            ca_file: "./my-certs/ca.crt"
            include_system_ca_certs_pool: false

            # Client identity for mTLS (these make it mTLS instead of just TLS)
            cert_file: "./my-certs/client.crt"
            key_file: "./my-certs/client.key"

          # Flags for enabling agent capabilities
          capabilities:
            # whether the agent should report its status
            #
            # Optional (default = true)
            reports_status: true

            # whether the agent should report its current effective config
            #
            # Optional (default = true)
            reports_effective_config: true

            # whether the agent should report its health
            #
            # Optional (default = true)
            reports_health: true

            # whether the agent should send periodic heartbeats
            #
            # Optional (default = true)
            reports_heartbeat: true

            # whether the agent should accept a restart command from the server
            #
            # Optional (default = false)
            accepts_restart_command: true

            # whether the agent should accept new configurations from the server
            #
            # Optional (default = false)
            accepts_remote_config: true

          # Configuration for heartbeat timing.
          #
          # Optional (default = 30s)
          heartbeat_interval: "10s"

          # Options for configuring exponential backoff for initial WebSocket
          # connection.
          #
          # Each exponential backoff computed as
          # min(last_backoff * factor, max) + jitter
          #
          # Optional
          connect_retry:
            # initial backoff
            #
            # Optional (default = 250ms)
            initial: 1s

            # maximum backoff
            #
            # Optional (default = 30s)
            max: 30s

            # exponential backoff increase factor
            #
            # Optional (default = 2.0)
            factor: 2.0

          # Options for configuring exponential backoff retry for requests.
          #
          # Used in the event that the initial request to the server fails or
          # when the server responds with an error response type Unavailable but
          # does not specify retry_info in the response details
          #
          # Optional
          request_retry: {
            #  ... same exponential retry options as connect_retry above
          }

          # Options for engine reconciliation
          reconcile:
            # timeout for reconcile - Optional (default = 30)
            step_timeout_secs: 10

            # timeout for pipeline drain - Optional (default = 30)
            drain_timeout_secs: 10

            # timeout for pipeline delete - Optional (default = 30)
            delete_timeout_secs: 10

            # Whether to delete any pipelines that are missing from the config
            # received from the server. When `true`, any currently running 
            # pipeline not in the received config is drained/deleted. When 
            # `false` received remote configs are treated as additive/partial
            # updates.
            delete_missing: true

          
          # Configuration of instance UID
          #
          # Optional
          instance_uid: 
            # Manually specify an instance_uid.
            #
            # Optional - if not provided, a UUID v7 will be generated
            initial_value: "..."

            # Specify that the current instance UID should be persisted at some
            # location. If this is configured, the instance will attempt to 
            # read the initial value ofr the instance_uid from the location,
            # providing a stable instance_uid across controller extension 
            # restarts
            persist:
              # A file location in which to store the current instance_uid
              file_path: "/tmp/instance_uid.txt"

          # Attributes describing agent
          #
          # Optional
          agent_description:
            identifying_attributes:
              "attr.key": "attr value"
              # ...

            non_identifying_attributes:
              "attr.key2": "attr value"
              # ...
```

### TLS/mTLS behaviour

The configuration in the `tls` block will use the structure defined in [`otap_df_config::tls::TlsClientConfig`](
https://github.com/open-telemetry/otel-arrow/blob/f45dc00642f0187020a4a46de796a07bf368d443/rust/otap-dataflow/crates/config/src/tls.rs#L49-L92)

If the key is supplied without the certificate, or the certificate is
configured without the key, configuration will be considered invalid at
startup.

If the `tls` section of the config is not supplied, messages will be exchanged
in plaintext.

## Protocol Implementation

### Capability Configuration

Capabilities which control including what the agent sends and what fields it
accepts from the remote server, can be toggled via configuration.

### Instance UID resolution

The initial value `instance_uid` field will be resolved as follows:

Step 1: If the config value `instance_uid.persist` is configured, the 
implementation will attempt to read the previously persisted value from the
configured store. (e.g., from a file). If the config value cannot be read (e.g. 
if the file does not exist), proceed to the next step.

Step 2: If an initial value for the `instance_uid` is configured at (using the
`instance_uid.initial_value`) option, an attempt will be made to parse a UUID
from this value and use this as the instance_uid. If the config value cannot be
parser as a valid UUID, it is a configuration error.

Step 3: If the previous steps were not able to resolve the instance_uid, a new
instance UID will be generated from a generated UUIDv7.

### Client Behaviour and Message Flow

Note - this section is written in such a way as to assume that all capabilities
are enabled In each section, if certain capabilities controlling which fields
a message should contain, the field should be omitted/ignored in the sent/
received message unless otherwise stated.

#### 1. Initial Message

The client begins by sending an initial `AgentToServer` message sending its
full state, including the following fields:

- `instance_uid` - from config if present, otherwise generated
- `sequence_num` - zero if first message in sequence
- `agent_description` - from configuration if present, otherwise omit
- `capabilities` - list of supported, enabled capabilities including:
  - Reports status
  - Reports Effective Config
  - Reports Health
  - Reports Remote Config
  - Reports Heartbeat
  - Accepts Remote Config
  - Accepts Restart Command
- `health` - see section below on server health
- `flags` - `Unset`
- `custom_capabilities` - list capability to send full state as custom message
  (see section below on custom capabilities).
- `remote_config_status` - status field = `Unset`
- `connection_settings_status` - status field = `Unset`
- `custom_message` - full pipeline status from status snapshot

#### 2. Exchange Messages

The client and server exchange a series of messages.

##### Handle `ServerToAgent` message

When the client receives a message from the server (a `ServerToAgent` message),
it should handle:

- If the message is in any way invalid (invalid protobuf, unexpected type of
  websocket message, etc.) it should ignore the message
- If the `instance_uid` field does not match the agent's `instance_uid`,
  then ignore the message
- If the message contained an error in the `error_response` field:
  - If the type is `Unavailable`, the client should disconnect the TCP
    connection, backoff exponentially, and retry the last request
  - Otherwise, ignore the message
- If the field `agent_identification` is set, update the `instance_uid`
- If the field `remote_config` is set:
  - If a valid config can be found in the config map and the hash does not
    match the last applied hash, reconcile the new config while sending
    appropriate replies (see section below on Engine Config Reconciliation)
- If the flags `ReportFullState` and/or `ReportAvailableComponents` are set,
  the next reply to the server should contain full state or available
  components. If a new config is being reconciled, these can be included on
  the message that is sent to the server letting it know config is being
  applied. Otherwise, send an ad-hoc message immediately reporting these
  fields.

In any case where a message is ignored (especially in error cases), appropriate
telemetry should be emitted.

##### Error Handling

When errors arise during the exchange of messages, there are three categories 
of errors and ways in which they should be handled
- ignorable errors
- notifiable errors
- session fatal errors

When the error is ignorable, the agent may emit appropriate telemetry but will
continue receiving messages on the same connection.

Notifiable errors generally occur when the server sends some invalid/
unprocessable configuration. The agent should send an `AgentToServer` message
to the server with `remote_config_status.status` of `Failed` with an 
`error_message` explaining why the message could not be handled as well as the
config hash.

For session fatal errors, the client should disconnect backoff, retry connecting,
and commence exchanging messages by reporting its full state (see section above
on Initial Message for what to include). In these cases, the client will not reset
the sequence_num to 0.

The following errors are considered ignorable:
- The response contains an `instance_uid` that does not match the agent's 
  instance uid.
- `ServerToAgent` messages containing `error_response.type` of anything other
  than `Unavailable`.

Notifiable errors include:
- The client receives a valid `ServerToAgent` message, but the remote_config
  did not contain the config at the expected key, the config is not encoded
  in a supported format (e.g. `application/json`), or the config could not be
  decoded as the format identified in the config file content type.

All other errors are considered session fatal including:
- TCP/http errors, including non-200 HTTP responses and unexpected closure of
  TCP connection
- The client has received a message, but it has invalid protobuf encoding
- The client has received a WebSocket message, but the message type is not
  Binary (Opcode 0x2).

##### Retry Behaviour

All backoff/retry behaviour shall be configurable and have the following 
configuration options:

- The initial backoff duration (default = 250ms)
- The maximum backoff duration (default = 15s)
- The exponential factor (default = 2.0)

The first backoff shall be the initial backoff and each subsequent shall be
computed using the  formula: `min(last_backoff * factor, max) + jitter`
where `jitter` is a random factor between 0.8 and 1.2.

The establishing of the connection between the client and the server will be
retried with an exponential backoff.

Once the connection is established, if the a `ServerToAgent` message is 
received with an `error_response.type` of `Unavailable`, the client will
disconnect, then use an exponential backoff before retrying the request
until it no-longer receives responses with this error type (e.g. until
server availability appears restored), unless a different error occurs in
which case the behaviour should follow that which is specified in the section
above related to error handling.

See the section on
[WebSocket Throttling](https://opentelemetry.io/docs/specs/opamp/#throttling)
in the OpAMP documentation for more details.

In the case where there is some error sending or receive the requests, the
client may disconnect and following this disconnection it will resume the
exchange of messages by sending an initial message with its full state (See
the note in the section above about how session fatal errors are handled).
If only a single request or response was attempted before this disconnect/
resumption, the client should also back off using the configured connection
retry backoff. This ensures that, if every request causes a session fatal
error that there is some backoff between requests, avoiding a flood of
retried initial messages.

##### Send periodic heartbeats

The client should send periodic heartbeats to the server at regular intervals.

These should always contain:

- `instance_uid`
- `sequence_num`
- `capabilities`
- `health` - computed from current pipeline status (see section below on
  Health Resolution).
- `remote_config_status` - computed from current pipeline status (see section
  below on Status Resolution), including the last applied config hash.
- `custom_message` - full pipeline status from status snapshot

Note: the heartbeat message should carry the `remote_config_status` and the
`custom_message` with the status snapshot even when there is no change to the
applied configuration. This gives the server periodic confirmation that the
agent is still running some configuration version alongside simple
confirmation that the server is alive.



### Agent Identity

The `AgentToServer` message contains an `instance_uid` and an
`AgentDescription` with identifying attributes. These are used by the server
to identify the agent.

Different control plane implementations may have different mechanisms for
identifying the client depending on their unique deployment scenario. As such,
it is not specified how agents should identify themselves nor how to compute
agent description - these parameters will be configurable.

```yaml
engine:
  controller:
    extensions:
      opamp:
        type: "urn:otel:controller_extension:opamp"
        config:
          # ...
          instance_uuid: "<uuid>"
          agent_description:
            identifying_attributes:
              attr.key: "attr_value"
              # ...
            non_identifying_attributes:
              attr.key2: "attr_value"
              # ...
```

If `instance_uuid` is not specified, a uuid v7 will be created.
Note that this logic only applies to bootstrap resolution of the
`instance_uid` - the server may respond with a new agent identification to
override the [initial value](https://opentelemetry.io/docs/specs/opamp/#servertoagentagent_identification).

If `agent_description` is not configured, the `agent_description` field will
not be set on the `AgentToServer` messages.

### Engine Config Representation

The server should supply the engine config as JSON, embedded within the
`ServerToAgent.remote_config.config.config_map["desired_state"]` which contains
an `AgentConfigFile` message.

The `body` of the message should contain the JSON serialized
`otap_df_config::engine::OtelDataflowSpec` and the `content_type` field should
identify that the body is JSON serialized.

```rs
ServerToAgent {
    remote_config: AgentRemoteConfig {
        config_hash: <hash>
        config: AgentConfigMap {
            config_map: HashMap {
                "desired_state": AgentConfigFile {
                    content_type: "application/json",
                    body: ...
                }
            }
        }
    }
}
```

The `config_hash` computation is performed by the server. The client does not
depend on any particular implementation or algorithm, but the server should
endeavour to choose the hash such that each new config has a unique hash value
without collisions. The agent does not calculate hashes, it only stores and
compares them. This is
[recommended by the OpAMP spec as well](https://opentelemetry.io/docs/specs/opamp/#calculating-hashes).

### Engine Config Reconciliation

When the OpAMP agent controller extension receives a `ServerToAgent` message,
if it contains a remote configuration whose `config_hash` does not match the
last applied config hash, the new config will be applied to the engine.

The agent will try to deserialize the remote config as an `OtelDataflowSpec`.
If deserialization fails, it will respond to the server with an `AgentToServer`
message with a `remote_config_status` containing a `FAILED` status.

If deserialization of the new remote_config succeeds, the agent will immediately
reply to the server with an `AgentToServer` message with a `remote_config_status`
containing an `APPLYING` status. This message should be sent before using the DFE 
`ControlPlane` to reconcile the engine config.

The agent will then call `ControlPlane::reconcile_engine_config` which takes as 
arguments various timeouts and options which will be exposed as configuration
on the controller extension.

When this function returns, the agent will use the
`Result<EngineConfigReconcileStatus>` to generate another `AgentToServer` message
with status indicating the result of the reconciliation of the remote config.

### Status Resolution

#### `RemoteConfigStatus`

The `AgentToServer` message will contain a
[`RemoteConfigStatus` message](https://opentelemetry.io/docs/specs/opamp/#remoteconfigstatus-message)
for this purpose which contains high-level information about the applied remote
config.

Before receiving any configuration from a remote server, the agent will always
set the status to `UNSET`.

Otherwise, the status will be derived from the pipeline status snapshot
returned by the control plane. Note, this snapshot contains a `PipelineStatus`
object, which contains `PipelineRuntimeStatus` for all instances of a pipeline,
which contains a `PipelinePhase`.

The remote config status will be derived from the snapshot using the following
rules:

- If any instance of a pipeline has phase `Pending`, `Starting`,
  `Draining`, `Updating`, `RollingBack` or `Deleting` the remote config status
  will be `Applying`
- Otherwise, if any instance of any pipeline has phase `Failed` or `Rejected`,
  the remote config status will be `Failed`
- Otherwise, the remote config status will be `Applied`

### Health Resolution

The `AgentToServer` message contains a `health` field which is a
`ComponentHealth` message. The implementation will use this to report on the
health status of each group and pipeline.

Example:

```rs
AgentToServer {
    health: ComponentHealth {
        healthy: true,
        status: "running",
        component_health_map: HashMap {
            "<group_key>": ComponentHealth {
                healthy: true,
                status: "running",
                component_health_map: HashMap {
                    "<pipeline_key>": ComponentHealth {
                        healthy: true,
                        status: "running",
                    }
                }
            }
        }
    }
}
```

The health status (`status` field) for each component can take on the following
values: `starting`, `running`, `stopping`, `stopped`, `failed` and `degraded`.

When resolving the status for the pipelines, the following logic will be used:

- If any instance has phase `Deleting` or `Draining` the status will be
  `stopping`
- Otherwise if any instance has phase `Pending`, `Updating`, `Starting` or
  `RollingBack` the status will be `starting`
- Otherwise if any instance has phase `Failed` or `Rejected` the status will be
  `failed`
- Otherwise if all the instances have phase `Running` the status will be
  `running`
- Otherwise if all the instances have phase `Stopped` or `Deleted` the status
  will be `stopped`
- Otherwise the status will be `degraded`

When resolving the status for engine's pipeline groups, the resolution logic
will use the following rules:

- If any pipeline has status `stopping`, the group status will be `stopping`
- Otherwise if any pipeline has status `failed`, the group status will be
  `failed`
- Otherwise if all pipeline have status `stopped`, the group status will be
  `stopped`
- Otherwise if all pipelines have status `running`, the group status will be
  `running`
- Otherwise the group status will be `degraded`.

### Custom Messages & Capabilities: 

#### Full Pipeline Status

The component health module presents an aggregate view of the health of each
group and pipeline, but the detailed pipeline status contains additional
information including specifically in which phase each instance of each
pipeline is in.

For servers that need this level of insight into the status of the pipelines
the implementation will advertise a custom capability representing its ability
to provide the full configuration status via a custom message.

Proposed capability FQDN: `io.open-telemetry.arrow-dfe.pipeline-status/v1`.

Custom message example:

```rs
AgentToServer {
  capabilities: ["io.open-telemetry.arrow-dfe.pipeline-status/v1"],
  custom_message: CustomMessage {
    capability: "io.open-telemetry.arrow-dfe.pipeline-status/v1",
    r#type: "report",
    data: [ ... ]
  }
}
```

The payload here is the bytes from a JSON serialized status snapshot (e.g., the
`HashMap<PipelineKey, PipelineStatus>` returned from
`ObservedStateHandle::status_snapshot`).

#### Imperative Configuration Commands

The primary mechanism for the server pass configuration to the agent will be as
described above which is to pass the entire configuration declaratively in the
`remote_config` config map.

However, there may be instances where the server wishes to change the engine
configuration using imperative commands such as starting, stopping, updating
or reconfiguring specific pipelines.

For this, a custom message will be supported that can contain multiple
operations to perform on certain pipelines.

Proposed capability FQDN: `io.open-telemetry.arrow-dfe.operations/v1`

Custom message structure

```rs
ServerToAgent {
  custom_message: CustomMessage {
    capability: "io.open-telemetry.arrow-dfe.operations/v1",
    r#type: "operations",
    data: [...]
  }
}
```

The data will will be a serialized message with the following structure:

```rs
struct Operations {
  /// Set of operations to apply
  operations: Vec<Operation>

  /// timeout for set of operations
  timeout: Option<Duration>
}

struct Operation {
  /// Unique identifier of the request. Can be used for observability purposes
  /// and the reply to the server will contain the request ID.
  request_id: Vec<u8>,

  /// The command to perform
  command: Command,

  /// ID of pipeline on which to perform the command
  pipeline_id: String,

  /// ID of group to which the pipeline being acted upon 
  pipeline_group_id: String,

  /// timeout for this operation
  timeout: Option<Duration>
}

/// The command to perform
enum Command {
  /// Start an existing pipeline
  Start,

  /// Create (and possibly start) a new pipeline
  Create {
    config: PipelineConfig,

    /// whether to start the pipeline after it has been created
    start: bool,
  },

  /// Update configuration of a pipeline
  Update {
    config: PipelineConfig
  },

  /// Drain & Shutdown a pipeline
  Shutdown,
  
  /// Delete a pipeline. It will be drained and shutdown before it is deleted.
  Delete,
}
```

After attempting agent should reply with a message containing the request ID
as well as the information about the result of the operation application,
including whether the operation was able to be successfully applied, and
if not, a description of the error that prevented the operation from being
successful.

## Telemetry

### Metrics

The controller extension implementation should produce metrics:

- Count successful `AgentToServer` messages transmitted
  - There should be a dimension on this attribute of te remote config status.
- Count failed `AgentToServer` messages transmitted (due to TCP or HTTP errors)
- Count `ServerToAgent` successfully handled
  - there should be attributes for whether the remote config was actually 
    reconciled or whether it was handled by skipping it due to unchanged
    config hash.
- Count `ServerToAgent` messages unsuccessfully handled (due to instance_uid
  mismatch, invalid proto encoding, or errors etc.). There should be an 
  attribute for the error kind in cases where a error_response is contained in
  the message.
- Count remote configs successfully reconciled
  - There should also be a counter of total time spent reconciling.
- Count remote configs unsuccessfully reconciled
- Count of times WebSocket connects/reconnects
- Count of instances where WebSocket was unexpectedly closed by the server.

## Future Work

Future design work - potential areas worth exploring to extend the initial
implementation described in this document

### Client Settings

OpAMP has the capability to configure connection settings both between the
agent & server, and between agent and some external OTLP receiver to which the
agent should export its own internal telemetry.

In the case of DFE, the settings between agent & OTLP receiver are not useful
because this can be configured via the internal telemetry pipeline, the
configuration for which is contained in the `ServerToAgent`'s `remote_config`
message.

The connection settings _could_ be used to control the connection/
authentication between the agent and server, using inspiration from the OpAMP
spec (
[this section](https://opentelemetry.io/docs/specs/opamp/#connection-settings-management)
). A followup design can be created for this.

### Packages

In [#2973](https://github.com/open-telemetry/otel-arrow/issues/2973) a proposal
was added to have WASM based plugins which implement pipeline components.

OpAMP has a capability for managing Packages that could be used to manage the
availability and provisioning of these plugins.

See [documentation](https://opentelemetry.io/docs/specs/opamp/#packages).

### Server SDK

This design completely abdicates the DFE of the responsibility for creating/
maintaining an OpAMP server. This is the concern of some system designer
looking to manage their own fleet of DFEs. It's not the intention of this
design to prescribe a solution for this, especially given that its unknown
how such a theoretical system will manage/generate DFE config.

However, we can still imagine a world where there are some cross-cutting
primitives that _most_ OpAMP server implementers will need to build. This
includes an HTTP server, possibly with websocket enabled, cert management,
instance_uid and other state management, etc.
