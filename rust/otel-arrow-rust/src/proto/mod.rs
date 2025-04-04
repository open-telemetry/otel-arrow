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

//! Protocol buffer definitions for OpenTelemetry Arrow Protocol

pub mod opentelemetry {
    pub mod proto {
        pub mod experimental {
            pub mod arrow {
                pub mod v1 {
                    pub use crate::proto::opentelemetry_proto_experimental_arrow_v1::*;
                }
            }
        }
    }
}

/// Generated protocol buffer code for OpenTelemetry Arrow v1
#[path = "opentelemetry.proto.experimental.arrow.v1.rs"]
pub mod opentelemetry_proto_experimental_arrow_v1;

/// OTAP data model modules
pub mod pdata;