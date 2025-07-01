// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::grpc::OTAPData;
use crate::proto::opentelemetry::experimental::arrow::v1::BatchArrowRecords;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::error::Error;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::local::receiver as local;
use serde_json::Value;
use std::rc::Rc;
use otap_df_config::node::NodeUserConfig;

/// The URN for the fake data generator receiver
pub const OTAP_FAKE_DATA_GENERATOR_URN: &str = "urn:otel:otap:fake_data_generator";

/// A Receiver that generates fake OTAP data for testing purposes.
pub struct FakeGeneratorReceiver {
}

/// Declares the fake data generator as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_FAKE_DATA_GENERATOR: ReceiverFactory<OTAPData> = ReceiverFactory {
    name: OTAP_FAKE_DATA_GENERATOR_URN,
    create: |node_config: Rc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
        ReceiverWrapper::local(FakeGeneratorReceiver::from_config(&node_config.config), node_config, receiver_config)
    },
};

impl FakeGeneratorReceiver {
    /// creates a new fake data generator
    #[must_use]
    pub fn new(
    ) -> Self {
        FakeGeneratorReceiver {}
    }

    /// Creates a new fake data generator from a configuration object
    #[must_use]
    pub fn from_config(_config: &Value) -> Self {
        // ToDo: implement config parsing
        FakeGeneratorReceiver {
        }
    }
}

// Use the async_trait due to the need for thread safety because of tonic requiring Send and Sync traits
// The Shared version of the receiver allows us to implement a Receiver that requires the effect handler to be Send and Sync
//
#[async_trait(?Send)]
impl local::Receiver<OTAPData> for FakeGeneratorReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel,
        mut effect_handler: local::EffectHandler<OTAPData>,
    ) -> Result<(), Error<OTAPData>> {
        for i in 0..10 {
            let msg = OTAPData::ArrowLogs(BatchArrowRecords::default());
            // Send the fake data message to the effect handler
            effect_handler.send_message(msg).await?;
        }

        //Exit event loop
        Ok(())
    }
}