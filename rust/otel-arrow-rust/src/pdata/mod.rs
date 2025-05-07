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

//! This module contains data structures for OTLP and OTAP pipeline data

pub mod otlp;

// TODO use the OTel-Rust API definitions, if such a dependency.

// TODO: otherwise, specialize TraceID and SpanID for [u8; 8 or 16].

#[derive(Clone, Copy, Debug)]
pub struct TraceID([u8; 16]);

impl TraceID {
    pub fn new(value: &[u8; 16]) -> TraceID {
        TraceID(*value)
    }
}

impl From<[u8; 16]> for TraceID {
    fn from(tid: [u8; 16]) -> Self {
        TraceID(tid)
    }
}

impl From<TraceID> for Vec<u8> {
    fn from(tid: TraceID) -> Self {
        tid.0.to_vec()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpanID([u8; 8]);

impl SpanID {
    pub fn new(value: &[u8; 8]) -> SpanID {
        SpanID(*value)
    }
}

impl From<SpanID> for Vec<u8> {
    fn from(sid: SpanID) -> Self {
        sid.0.to_vec()
    }
}

impl From<[u8; 8]> for SpanID {
    fn from(sid: [u8; 8]) -> Self {
        SpanID(sid)
    }
}
