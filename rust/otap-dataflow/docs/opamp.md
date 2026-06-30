# OpAMP Controller Extension

This document describes the mechanism through which dataflow engine can be
configured via an embedded OpAMP Agent, implemented as a controller extension.

## Problem

As the machinations of those desiring to control telemetry flow the become more
sophisticated, the concept of building some fleet management becomes
increasingly appealing.

System designers, probably, will wish to implement their own remote control
plane without having to design, from scratch, a new protocol for sending
configuration or receiving status.

OpAMP provides such a protocol, and the solution to this problem will be to
add the capability for dataflow engine to act as a OpAMP agent, receiving
configuration from some remote server while emitting its state & health.

## Background

### OpAMP

See [Full OpAMP documentation](https://opentelemetry.io/docs/specs/opamp/) for
more information.

OpAMP (Open Agent Management Protocol) is a vendor-agnostic network protocol
existing as part of the Open-Telemetry ecosystem. It specifies a mechanism for:

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
Find proto message definitions
[here](https://github.com/open-telemetry/opamp-spec/blob/main/proto/opamp/v1/opamp.proto).

### Controller extension

OTel Controller Extensions allow custom implementation of logic that interacts
with the engine control API to reconcile configuration and report status. They
are configurable at runtime and can be linked at compile time (similar to
processors, exporters, receivers, etc.).

See [#3263](https://github.com/open-telemetry/otel-arrow/issues/3263) and
[#3281](https://github.com/open-telemetry/otel-arrow/issues/3281).

## Scope

This design document only covers the implementation of OpAMP for exchanging
engine configuration and pipeline status.

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


        # Configuration for heart-beat timing. 
        #
        # Optional (default = 30s)
        heart_beat_interval: "10s"

        # Options for configuring exponential backoff for initial Websocket
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

        # Manually specify an instance_uuid.
        #
        # Optional - if not provided, a UUID v7 will be generated
        instance_uuid: "..."

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

## Protocol Implementation

### Client Behaviour and Message Flow

#### 1. Initial Message

The client begins by sending an initial `AgentToServer` message sending its 
full state, including the following fields:
- `instance_uid` - from config if present, otherwise generated
- `sequence_num` - zero if first message in sequence
- `agent_description` - from configuration if present, otherwise omit
- `capabilities` - list of supported capabilities:
  - Reports status
  - Reports Effective Config
  - Reports Health
  - Reports Remote Config
  - Reports Heart Beat
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

##### Send periodic heart beats:

The client should send periodic heartbeats to the server at regular intervals.

These should contain:
- `instance_uid`
- `sequence_num`
- `capabilities`
- `health` - computed from current pipeline status (see section below on 
  Health Resolution).
- `remote_config_status` - computed from current pipeline status (see section
  below on Status Resolution)
- `custom_message` - full pipeline status from status snapshot


#### Error Handling

On TCP Errors or HTTP errors, the client should backoff, retry connecting, and
commence exchanging messages by reporting its full state (see section above on
Initial Message for what to include).

### Agent Identity

The `AgentToServer` message contains an `instance_uid` and an 
`AgentDescription` with identifying attributes. These are used by the server
to identify the agent.

Different control plane implementation may have different mechanisms for 
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
override the initial value (see 
[here](https://opentelemetry.io/docs/specs/opamp/#servertoagentagent_identification)
).

If `agent_description` is not configured, the `agent_description` field will
not be set on the `AgentToServer` messages.

### Engine Config Representation

The server should supply the engine config as JSON, embedded within the 
`ServerToAgent.remote_config.config.config_map["desired_state"]` which contains
an  `AgentConfigFile` message. 

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
compares them (see 
[here](https://opentelemetry.io/docs/specs/opamp/#calculating-hashes)).

### Engine Config Reconciliation

When the OpAMP agent controller extension receives a `ServerToAgent` message,
if it contains a remote configuration whose `config_hash` does not match the
last applied config hash, the new config will be applied to the engine.

The agent will try to deserialize the remote config as an `OtelDataflowSpec`. 
If deserialization fails, it will respond to the server with an `AgentToServer`
message with a `remote_config_status` containing a `FAILED` status.

If deserialization of the new remote_config succeeds, the agent will immediately
reply to the server with an `AgentToServer` message with a `remote_config_status`
containing an `APPLYING` status, and it will then use the DFE `ControlPlane` to
reconcile the engine config.

`ControlPlane::reconcile_engine_config` also takes as arguments various timeouts,
which will be exposed as configuration on the controller extension.

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
set the status to `APPLYING`.

Otherwise, the status will be derived from the pipeline status snapshot
returned by the control plane. Note, this snapshot contains a `PipelineStatus`
object, which contains `PipelineRuntimeStatus` for all instances of a pipeline,
which contains a `PipelinePhase`.

The remote config status will be derived from the snapshot using the following
rules:
- If any instance of any pipeline has phase is `Pending`, `Starting`, 
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
- Otherwise if all pipeline has status `stopped`, the group status will be 
  `stopped`
- Otherwise if all pipelines have status `running`, the group status will be
  `running`
- Otherwise the group status will be `degraded`.

### Custom Messages & Capabilities: Full Pipeline Status

The implementation will advertise a custom capability representing its ability
to provide the full configuration status via a custom message.

Proposed capability FQDN: `io.open-telemetry.otap-dfe.pipeline-status/v1`.

Custom message example:
```rs
AgentToServer {
  capabilities: ["io.open-telemetry.otap-dfe.pipeline-status/v1"],
  custom_message: CustomMessage {
    capability: "io.open-telemetry.otap-dfe.pipeline-status/v1",
    r#type: "report",
    data: [ ... ]
  }
}
```

The payload here is the bytes from a JSON serialized status snapshot (e.g., the
`HashMap<PipelineKey, PipelineStatus>` returned from
`ObservedStateHandle::status_snapshot`).

## Telemetry

### Metrics

The controller extension implementation should produce metrics:
- Count successful `AgentToServer` messages transmitted
- Count failed `AgentToServer` messages transmitted (due to TCP or HTTP errors)
- Count `ServerToAgent` successfully handled
- Count `ServerToAgent` messages unsuccessfully handled (due to instance_uid
  mismatch, invalid proto encoding, etc.)
- Count remote configs successfully reconciled
- Count remote configs unsuccessfully reconciled

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

### Imperative Command Capability

This design assumes the server will specify the remote configuration in full.
E.g. the agent expects to receive the config declaratively.

However, in the future we may want to expose a mechanism for servers to trigger
the ad-hoc creation/reconfiguration/deletion of some pipeline. This could be 
supported through a custom agent capability / custom server message.

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

We could consider making this easier by providing an SDK in rust that abstracts
away all these details and exposes a higher level API at the level of agent 
identity, DFE config.
