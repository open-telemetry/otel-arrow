use crate::grpc::OTLPRequest;
use crate::grpc::grpc_stubs::proto::{metrics::v1::{ResourceMetrics, ScopeMetrics, metric}, logs::v1::{ResourceLogs, ScopeLogs}, trace::v1::{ResourceSpans, ScopeSpan}};
use otap_df_engine::processor::{EffectHandler, Receiver, ControlMsgChannel, SendableMode};
use regex::Regex;

// otlp processor should have impl the processMetrics, processTraces, and processLogs function
#[async_trait(?Send)]
trait OTLPProcessor: Processor {
    type PData = OTLPRequest;
    type Mode = LocalMode;
    
    fn processMetrics(&self, data: PData);
    fn processTraces(&self, data: PData);
    fn processLogs(&self, data: PData);

    async fn process(
        &mut self,
        msg: Message<Self::PData>,
        effect_handler: &mut EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>> {
        match msg {
            Message::Control(control) => match control {
                TimerTick {} => {
     
                }
                Config { .. } => {
                
                }
                Shutdown { .. } => {
                    break;
                }
                _ => {}
            },
            Message::PData(data) => {
                // process message here
                let processed_data = match data {
                    OTLPRequest::Metrics(mut data) => {
                        self.processMetrics(&mut data);
                    }
                    OTLPRequest::Traces(mut data) => { 
                        self.processTraces(&mut data);
                    }
                    OTLPRequest::Logs(mut data) => {
                        self.processLogs(&mut data);
                    }

                };
                effect_handler
                    .send_message(processed_data)
                    .await?;
            }
        }
        Ok(())
    }
}
