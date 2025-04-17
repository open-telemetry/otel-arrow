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

// TODO: also, define From<> instead of Into<> in ../proto/mod.rs

#[derive(Clone, Copy, Debug)]
pub struct TraceID([u8; 16]);

type Error = &'static str;

impl<'a> TraceID {
    pub fn try_new(value: &[u8]) -> Result<TraceID, Error> {
        if value.len() == 16 {
            Ok(TraceID(value.try_into().unwrap()))
        } else {
            Err("wrong size [u8] for TraceID")
        }
    }
}

impl Into<Vec<u8>> for TraceID {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpanID([u8; 8]);

impl Into<Vec<u8>> for SpanID {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl<'a> SpanID {
    pub fn try_new(value: &[u8]) -> Result<SpanID, Error> {
        if value.len() == 8 {
            Ok(SpanID(value.try_into().unwrap()))
        } else {
            Err("wrong size [u8] for SpanID")
        }
    }
}
