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

//! Tests for OTAP message types and builders

#[cfg(test)]
mod tests {
    use crate::proto::opentelemetry_proto_experimental_arrow_v1::{
        BatchArrowRecords, ArrowPayloadType
    };
    use crate::proto::pdata::otap::{BatchArrowRecordsBuilder, ArrowPayloadBuilder};
    
    #[test]
    fn test_batch_arrow_records_builder() {
        // Create a BatchArrowRecords using the builder pattern
        let batch = BatchArrowRecordsBuilder::new()
            .batch_id(123)
            .add_payload(
                ArrowPayloadBuilder::new()
                    .schema_id("test-schema")
                    .payload_type(ArrowPayloadType::ResourceAttrs)
                    .record(vec![1, 2, 3, 4])
                    .build()
            )
            .headers(vec![5, 6, 7, 8])
            .build();
        
        // Verify the built object
        assert_eq!(batch.batch_id, 123);
        assert_eq!(batch.arrow_payloads.len(), 1);
        assert_eq!(batch.arrow_payloads[0].schema_id, "test-schema");
        assert_eq!(batch.arrow_payloads[0].r#type, ArrowPayloadType::ResourceAttrs as i32);
        assert_eq!(batch.arrow_payloads[0].record, vec![1, 2, 3, 4]);
        assert_eq!(batch.headers, vec![5, 6, 7, 8]);
    }
    
    #[test]
    fn test_batch_builder_from_struct() {
        // Demonstrate creating a builder from an existing struct
        let batch = BatchArrowRecords::builder()
            .batch_id(456)
            .build();
        
        assert_eq!(batch.batch_id, 456);
        assert_eq!(batch.arrow_payloads.len(), 0);
    }
    
    #[cfg(feature = "derive")]
    #[test]
    fn test_message_trait_implementation() {
        use crate::proto::pdata::otap::Message;
        use crate::proto::pdata::otap::utils;
        
        // This test will only compile if the Message trait is properly derived
        assert_eq!(BatchArrowRecords::message_type(), "BatchArrowRecords");
        
        // Test utils functions
        let batch = BatchArrowRecords::builder()
            .batch_id(789)
            .build();
        
        let bytes = utils::to_bytes(&batch);
        let decoded = utils::from_bytes::<BatchArrowRecords>(&bytes).unwrap();
        
        assert_eq!(decoded.batch_id, 789);
    }
}