// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::grpc::OtapArrowBytes;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::receiver::ReceiverWrapper;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;

/// The URN for the fake data generator receiver
pub const OTAP_FAKE_DATA_GENERATOR_URN: &str = "urn:otel:otap:fake_data_generator";

/// Configuration for the Fake Data Generator Receiver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Number of batches to generate
    pub batch_count: usize,
}

/// A Receiver that generates fake OTAP data for testing purposes.
pub struct FakeGeneratorReceiver {
    /// Configuration for the fake data generator
    config: Config,
}

/// Declares the fake data generator as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_FAKE_DATA_GENERATOR: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_FAKE_DATA_GENERATOR_URN,
    create: |node_config: Rc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            FakeGeneratorReceiver::from_config(&node_config.config)?,
            node_config,
            receiver_config,
        ))
    },
};

impl Default for FakeGeneratorReceiver {
    fn default() -> Self {
        Self {
            config: Config {
                batch_count: 10, // Default batch count
            },
        }
    }
}

impl FakeGeneratorReceiver {
    /// creates a new fake data generator
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new fake data generator from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(FakeGeneratorReceiver { config })
    }
}

/// Implement the Receiver trait for the FakeGeneratorReceiver
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for FakeGeneratorReceiver {
    async fn start(
        self: Box<Self>,
        _ctrl_msg_recv: local::ControlChannel,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        for _ in 0..self.config.batch_count {
            let msg = OtapArrowBytes::ArrowLogs(BatchArrowRecords::default());
            // Send the fake data message to the effect handler
            effect_handler.send_message(msg.into()).await?;
        }

        // Exit the receiver gracefully
        Ok(())
    }
}
