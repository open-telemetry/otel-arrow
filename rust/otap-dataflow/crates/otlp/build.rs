// SPDX-License-Identifier: Apache-2.0

//! A build script to generate the gRPC OTLP receiver API (client and server stubs.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // To regenerate the gRPC API from the proto file:
    // - Uncomment the following lines.
    // - Run `cargo build` to regenerate the API.
    // - Comment the following lines.
    // - Commit the changes.
    // tonic_build::configure()
    //     .out_dir("src/grpc/grpc_stubs")
    //     .compile_protos(
    //         &[
    //             "../../../../proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
    //             "../../../../proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
    //             "../../../../proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
    //         ],
    //         &["../../../../proto"],
    //     )?;
    // Ok(())
}