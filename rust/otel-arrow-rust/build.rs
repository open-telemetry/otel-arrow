// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;

fn main() {
    let out_dir = Path::new("src/proto");
    let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    // Generate OTAP (Arrow) protos
    generate_otap_protos(out_dir, &base);

    // Generate OTLP protos
    generate_otlp_protos(out_dir, &base);
}

fn prost_cfg() -> prost_build::Config {
    // Documentation comments are a problem because they look like
    // doctests and clippy wants to checks them.
    let mut cfg = prost_build::Config::default();
    cfg.disable_comments(["."]);
    cfg
}

fn generate_otap_protos(out_dir: &Path, base: &str) {
    // Create a tonic-build configuration with our custom settings
    let builder = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", r#"#[cfg(feature = "server")]"#)
        .client_mod_attribute(".", r#"#[cfg(feature = "client")]"#)
        .disable_comments(".")
        .out_dir(out_dir);

    // Compile the protobuf definitions
    builder
        .compile_protos_with_config(
            prost_cfg(),
            &["experimental/arrow/v1/arrow_service.proto"],
            &[format!("{}/../../proto/opentelemetry/proto/", base)],
        )
        .expect("Failed to compile OTAP protos.");
}

fn generate_otlp_protos(out_dir: &Path, base: &str) {
    // Configure the builder for OTLP protos
    let builder = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", r#"#[cfg(feature = "server")]"#)
        .client_mod_attribute(".", r#"#[cfg(feature = "client")]"#)
        .disable_comments(".")
        .out_dir(out_dir);

    // Note: this adds derive expressions for each OTLP message type.
    let builder = otlp_model::add_type_attributes(builder);

    builder
        .compile_protos_with_config(
            prost_cfg(),
            &[
                "opentelemetry/proto/common/v1/common.proto",
                "opentelemetry/proto/resource/v1/resource.proto",
                "opentelemetry/proto/trace/v1/trace.proto",
                "opentelemetry/proto/metrics/v1/metrics.proto",
                "opentelemetry/proto/logs/v1/logs.proto",
                "opentelemetry/proto/collector/logs/v1/logs_service.proto",
                "opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
            ],
            &[format!("{}/../../proto/opentelemetry-proto", base)],
        )
        .expect("Failed to compile OTLP protos.");
}
