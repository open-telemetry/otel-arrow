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

fn generate_otap_protos(out_dir: &Path, base: &str) {
    // Create a tonic-build configuration with our custom settings
    let builder = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", r#"#[cfg(feature = "server")]"#)
        .client_mod_attribute(".", r#"#[cfg(feature = "client")]"#)
        .type_attribute(".", r#"#[derive(crate::pdata::otap::Message)]"#);

    // Compile the protobuf definitions
    builder
        .out_dir(out_dir)
        .compile_protos(
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
        .type_attribute(".", r#"#[derive(crate::pdata::otlp::Message)]"#);

    builder
        .out_dir(out_dir)
        .compile_protos(
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
