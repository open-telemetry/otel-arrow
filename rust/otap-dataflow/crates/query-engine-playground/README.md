# OPL Playground

Interactive web playground for executing OPL (OTel Processing Language) programs
against OpenTelemetry data.

## Running

```bash
cargo run -p otap-df-query-engine-playground
```

Then open <http://localhost:3000> in a browser.

## How it works

The playground is a single Rust binary that serves a web UI and exposes an HTTP
API for pipeline execution:

1. **Browser** — loads an embedded HTML page with inline JavaScript.
   [protobuf.js](https://github.com/protobufjs/protobuf.js) (from CDN) loads
   the OTel `.proto` definitions (served by the backend) and uses them to encode
   user-provided OTLP JSON into protobuf bytes and decode results back to JSON
   for display.

2. **Server** — an [Axum](https://github.com/tokio-rs/axum) HTTP server that
   embeds the `.proto` files from the repo, serves them to the browser, receives
   protobuf-encoded OTLP data plus an OPL query string, runs the query-engine
   pipeline, and returns both the OTLP result (as protobuf) and pretty-printed
   Arrow tables.

### Data flow

```text
Browser                              Rust Backend
──────                               ────────────
protobuf.load() <--------- GET /proto/  (serves embedded .proto files)

User edits OTLP JSON --->
  protobuf.js encodes to binary --->
    POST /api/execute --------------> prost::Message::decode()
                                      otlp_to_otap()
                                      OplParser::parse()
                                      Pipeline::execute()
                                      otap_to_otlp()
                                      prost::Message::encode()
    JSON response <------------------
  protobuf.js decodes to JSON <--
Display result + Arrow tables <--
```
