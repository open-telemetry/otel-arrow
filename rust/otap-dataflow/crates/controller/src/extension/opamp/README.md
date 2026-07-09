# OpAMP Controller Extension

This extension lets the dataflow engine act as an OpAMP agent: it connects to a
remote OpAMP server, receives engine configuration from it, and reports health
and pipeline status back. See [`docs/opamp.md`](../../../../docs/opamp.md) for
the design document.

## Testing locally against the opamp-go reference server

The [opamp-go](https://github.com/open-telemetry/opamp-go) repository ships an
example OpAMP server with a web UI, which makes a convenient interop test bed:
the engine connects to it as an agent, and you can inspect its status and push
engine configuration to it from a browser.

### 1. Run the opamp-go example server

```sh
git clone https://github.com/open-telemetry/opamp-go
cd opamp-go/internal/examples
go run ./server -no-tls
```

- OpAMP endpoint: `ws://127.0.0.1:4320/v1/opamp` - web UI: <http://localhost:4321>
- `-no-tls` is required because this extension currently supports `ws://` only.

### 2. Run the engine

```sh
cargo build --bin df_engine
./target/debug/df_engine --config configs/opamp-controller-extension.yaml --num-cores 1
```

The agent appears on <http://localhost:4321> within a heartbeat interval:
instance `8be4df61-...`, health `Up`, and its effective config visible on the
agent page.

### 3. Push an engine config from the server

Open the agent page, paste the following into the *Additional Configuration*
box, and hit Save:

```json
{
  "version": "otel_dataflow/v1",
  "engine": {},
  "groups": {
    "default": {
      "pipelines": {
        "remote_pipeline": {
          "nodes": {
            "otlp_receiver": {
              "type": "receiver:otlp",
              "config": {
                "protocols": {
                  "grpc": { "listening_addr": "127.0.0.1:4317" }
                }
              }
            },
            "sink": { "type": "exporter:noop", "config": {} }
          },
          "connections": [{ "from": "otlp_receiver", "to": "sink" }]
        }
      }
    }
  }
}
```

Expected sequence, visible in the server log and the engine log:

1. The engine replies with `RemoteConfigStatus=APPLYING`.
2. The engine reconciles the config and starts the pipeline (an OTLP gRPC
   receiver comes up on `127.0.0.1:4317`).
3. The next reply reports `RemoteConfigStatus=APPLIED` and `Healthy=true`.

Pushing an invalid config (for example malformed JSON) is reported back as
`RemoteConfigStatus=FAILED` with an error message, and the previously running
pipelines are left untouched.

### 4. Clean up

Stop either side with Ctrl-C. The engine keeps retrying the connection with
exponential backoff while the server is down and re-registers when it returns;
note that the example server keeps agent state (including pushed config) in
memory only, so it forgets the agent on restart.
