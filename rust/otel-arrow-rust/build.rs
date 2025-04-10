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
    let builder = tonic_build::configure()
        .build_server(true)
        .build_client(true);

    let builder = builder
        .server_mod_attribute(".", r#"#[cfg(feature = "server")]"#)
        .client_mod_attribute(".", r#"#[cfg(feature = "client")]"#);

    builder
        .out_dir(out_dir)
        .compile_protos(
            &["experimental/arrow/v1/arrow_service.proto"],
            &[format!("{}/../../proto/opentelemetry/proto/", base)],
        )
        .expect("Failed to compile protos.");
}
